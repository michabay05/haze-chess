use crate::attack::AttackInfo;
use crate::bb::BBUtil;
use crate::board::{self, Board};
use crate::consts::{Piece, PieceColor, Sq};
use crate::eval::{self, EvalMasks};
use crate::engine::Engine;
use crate::move_gen::{self, MoveList};
use crate::moves::{self, Move, MoveFlag, MoveUtil};
use crate::threads;
use crate::tt::{HashTT, TTFlag};
use crate::uci::{self, UCIState};
use crate::zobrist::{ZobristInfo, self, ZobristAction};

use std::sync::{Arc, RwLock};

const FULL_DEPTH_MOVES: u32 = 4;
const REDUCTION_LIMIT: u32 = 3;
pub const MAX_SEARCH_PLY: usize = 64;

// Mating score bounds
// [-INFINITY, -MATE_VALUE ... -MATE_SCORE, ... SCORE ... MATE_SCORE ... MATE_VALUE, INFINITY]
const INFINITY: i32 = 50000;
const MATE_VALUE: i32 = 49000; // Upper bound
pub const MATE_SCORE: i32 = 48000; // Lower bound

const MVV_LVA: [[u32; 6]; 6] = [
    [105, 205, 305, 405, 505, 605],
    [104, 204, 304, 404, 504, 604],
    [103, 203, 303, 403, 503, 603],
    [102, 202, 302, 402, 502, 602],
    [101, 201, 301, 401, 501, 601],
    [100, 200, 300, 400, 500, 600],
];

#[derive(Clone)]
pub struct SearchInfo {
    // Half move counter
    pub ply: u32,
    pub nodes: u32,
    // PV flags
    pub follow_pv: bool,
    pub score_pv: bool,
    // 'Quiet' moves that cause a beta-cutoffs
    pub killer: [[Move; MAX_SEARCH_PLY]; 2], // [id][ply]
    pub history: [[Move; 64]; 12],    // [piece][sq]
    pub pv_len: [u32; MAX_SEARCH_PLY],
    pub pv_table: [[u32; MAX_SEARCH_PLY]; MAX_SEARCH_PLY],
    pub tt: Arc<RwLock<HashTT>>,
}

impl SearchInfo {
    pub fn new() -> Self {
        Self {
            ply: 0,
            nodes: 0,
            follow_pv: false,
            score_pv: false,
            killer: [[0; MAX_SEARCH_PLY]; 2],
            history: [[0; 64]; 12],
            pv_len: [0; MAX_SEARCH_PLY],
            pv_table: [[0; MAX_SEARCH_PLY]; MAX_SEARCH_PLY],
            tt: Arc::new(RwLock::new(HashTT::new())),
        }
    }

    pub fn reset(&mut self) {
        self.ply = 0;
        self.nodes = 0;
        self.follow_pv = false;
        self.score_pv = false;
        self.killer = [[0; MAX_SEARCH_PLY]; 2];
        self.history = [[0; 64]; 12];
        self.pv_len = [0; MAX_SEARCH_PLY];
        self.pv_table = [[0; MAX_SEARCH_PLY]; MAX_SEARCH_PLY];
    }
}

#[derive(Clone)]
pub struct SearchData {
    pub attack_info: AttackInfo,
    pub board: Board,
    pub eval_mask: EvalMasks,
    pub search_info: SearchInfo,
    pub zobrist_info: ZobristInfo,
    pub uci_state: Arc<RwLock<UCIState>>,
}

impl SearchData {
    pub fn from_engine(engine: &Engine) -> Self {
        Self {
            attack_info: engine.attack_info.clone(),
            board: engine.board.clone(),
            eval_mask: engine.eval_mask.clone(),
            search_info: engine.search_info.clone(),
            zobrist_info: engine.zobrist_info.clone(),
            uci_state: Arc::clone(&engine.uci_state),
        }
    }
}

pub fn search_pos(data: &SearchData, depth: u32, thread_count: usize) {
    // 1. Create search worker threads
    let workers = threads::create_search_workers(data, depth, thread_count);
    // 2. Join the search worker threads
    for mut worker in workers {
        if let Some(th) = worker.handle.take() {
            let _ = th.join();
        }
    }
}

pub fn worker_search_pos(mut data: SearchData, depth: u32, worker_id: usize) {
    // data.search_info = SearchInfo::new();
    data.search_info.reset();
    {
        let mut info_state = data.uci_state.write().unwrap();
        info_state.stop = false;
    }
    let mut alpha = -INFINITY;
    let mut beta = INFINITY;
    let start_time = uci::get_curr_time();

    let mut score;
    'deepen: for current_depth in 1..=depth {
        {
            if data.uci_state.read().unwrap().stop {
                break 'deepen;
            }
        }
        data.search_info.follow_pv = true;
        // Find the best move in the current position
        score = negamax(
            &mut data.search_info,
            &mut data.board,
            &data.attack_info,
            &data.eval_mask,
            &data.uci_state,
            &data.zobrist_info,
            alpha,
            beta,
            current_depth,
        );
        // Aspiration window
        if (score <= alpha) || (score >= beta) {
            alpha = -INFINITY;
            beta = INFINITY;
            continue;
        }
        // Set up window for next iteration
        alpha = score - 50;
        beta = score + 50;

        let time_diff = uci::get_curr_time() - start_time;

        if worker_id != 0 {
            continue;
        }
        // Only the first worker thread should print these information
        if data.search_info.pv_len[0] != 0 {
            let (cp_str, cp_score) = if score > -MATE_VALUE && score < -MATE_SCORE {
                ("mate", (-(score + MATE_VALUE) / 2) - 1)
            } else if score > MATE_SCORE && score < MATE_VALUE {
                ("mate", ((MATE_VALUE - score) / 2) + 1)
            } else {
                ("cp", score)
            };
            print!(
                "info score {} {} depth {} nodes {} time {} pv ",
                cp_str, cp_score, current_depth, data.search_info.nodes, time_diff
            );
            // Print principal variation
            for i in 0..(data.search_info.pv_len[0] as usize) {
                print!("{} ", data.search_info.pv_table[0][i].to_str().trim());
            }
            println!();
        }
    }
    if worker_id == 0 {
        println!("bestmove {}", data.search_info.pv_table[0][0].to_str().trim());
    }
}

const CHECK_UP_NODES: u32 = 2047;

fn negamax(
    info: &mut SearchInfo,
    board: &mut Board,
    attack_info: &AttackInfo,
    mask: &EvalMasks,
    uci_state: &Arc<RwLock<UCIState>>,
    zobrist_info: &ZobristInfo,
    mut alpha: i32,
    beta: i32,
    mut depth: u32,
) -> i32 {
    info.pv_len[info.ply as usize] = info.ply;
    // Store the current move's score
    let mut score;
    let mut tt_flag = TTFlag::Exact;
    // Repetition stuff
    let is_pv_node = (beta - alpha) > 1;

    // If score of current position exists, return score instead of searching
    // Read hash entry (if not root ply) score for current position and isn't PV node
    #[allow(unused_assignments)]
    let mut hash_score = None;
    {
        let mut info_tt = info.tt.write().unwrap();
        hash_score = info_tt.read_entry(board, alpha, beta, depth, info.ply);
    }
    if info.ply != 0 && hash_score.is_some() && !is_pv_node {
        return hash_score.unwrap();
    }

    // Communicate with UCI every so often
    if (info.nodes & CHECK_UP_NODES) == 0 {
        let mut info_state = uci_state.write().unwrap();
        info_state.check_up();
    }

    // Escape condition or Base case
    if depth == 0 {
        return quiescence(info, board, attack_info, mask, uci_state, zobrist_info, alpha, beta);
    }
    // Exit if ply > max ply; ply should be <= 63
    if info.ply > (MAX_SEARCH_PLY - 1) as u32 {
        return eval::evaluate(&board.pos, board.state.side, attack_info, mask);
    }
    // Increment nodes
    info.nodes += 1;

    // Check extension
    let in_check =  board::in_check(board, attack_info, board.state.xside);
    if in_check {
        depth += 1;
    }
    let mut legal_move_count = 0;

    // NULL move pruning
    if depth >= 3 && !in_check && info.ply != 0 {
        let clone = board.clone();
        info.ply += 1;
        // Repetition stuff
        if board.state.enpassant != Sq::NoSq {
            zobrist::update(zobrist_info, ZobristAction::SetEnpassant(board.state.enpassant), board);
        }
        board.state.enpassant = Sq::NoSq;
        board.state.change_side();
        zobrist::update(zobrist_info, ZobristAction::ChangeColor, board);
        // Search move with reduced depth to find beta-cutoffs
        score = -negamax(
            info,
            board,
            attack_info,
            mask,
            uci_state,
            zobrist_info,
            -beta,
            -beta + 1,
            depth - 1 - 2,
        );
        info.ply -= 1;
        *board = clone;
        // When timer runs out, return 0
        {
            if uci_state.read().unwrap().stop {
                return 0;
            }
        }
        // Fail hard; beta-cutoffs
        if score >= beta {
            return beta;
        }
    }
    let mut ml = MoveList::new();
    move_gen::generate(board, attack_info, &mut ml);
    if info.follow_pv {
        enable_pv_scoring(info, &mut ml);
    }
    sort_moves(info, board, &mut ml);

    let mut clone;
    let mut move_searched = 0;
    for mv in &ml.moves {
        // Preserve board state by copying it
        clone = board.clone();
        info.ply += 1;
        // Repetition stuff
        // Make sure that every move from this point on is legal
        if !moves::make(board, attack_info, zobrist_info, *mv, MoveFlag::AllMoves) {
            info.ply -= 1;
            // Repetition stuff
            continue;
        }
        legal_move_count += 1;

        // Full depth search
        if move_searched == 0 {
            score = -negamax(
                info,
                board,
                attack_info,
                mask,
                uci_state,
                zobrist_info,
                -beta,
                -alpha,
                depth - 1,
            );
        } else {
            // Late move reduction (LMR)
            if move_searched >= FULL_DEPTH_MOVES
                && depth >= REDUCTION_LIMIT
                && !in_check
                && mv.promoted().is_none()
                && !mv.is_capture()
            {
                score = -negamax(
                    info,
                    board,
                    attack_info,
                    mask,
                    uci_state,
                    zobrist_info,
                    -alpha - 1,
                    -alpha,
                    depth - 2,
                );
            } else {
                // Hack to ensure that full depth search is done
                score = alpha + 1;
            }
            // Principal Variation Search (PVS)
            if score > alpha {
                // Re-search at full depth but with a narrowed score bandwidth
                score = -negamax(
                    info,
                    board,
                    attack_info,
                    mask,
                    uci_state,
                    zobrist_info,
                    -alpha - 1,
                    -alpha,
                    depth - 1,
                );
                // If LMR fails, re-search at full depth and full score bandwidth
                if (score > alpha) && (score < beta) {
                    score = -negamax(
                        info,
                        board,
                        attack_info,
                        mask,
                        uci_state,
                        zobrist_info,
                        -beta,
                        -alpha,
                        depth - 1,
                    );
                }
            }
        }
        info.ply -= 1;
        // Repetition stuff
        *board = clone;
        // When timer runs out, return 0
        {
            if uci_state.read().unwrap().stop {
                return 0;
            }
        }
        move_searched += 1;

        // If a better move is found
        if score > alpha {
            // Switch flag to EXACT(PV node) from ALPHA (fail-low node)
            tt_flag = TTFlag::Exact;
            if !mv.is_capture() {
                info.history[mv.piece() as usize][mv.target() as usize] += depth;
            }

            // PV node
            alpha = score;

            // Write PV move
            info.pv_table[info.ply as usize][info.ply as usize] = *mv;

            // Copy PV from following plies
            for next_ply in (info.ply + 1)..info.pv_len[info.ply as usize + 1] {
                info.pv_table[info.ply as usize][next_ply as usize] =
                    info.pv_table[info.ply as usize + 1][next_ply as usize];
            }
            // Adjust PV length
            info.pv_len[info.ply as usize] = info.pv_len[info.ply as usize + 1];

            // Fail hard; beta-cutoff
            if score >= beta {
                {
                    let mut info_tt = info.tt.write().unwrap();
                    info_tt.write_entry(board, depth, beta, TTFlag::Beta, info.ply);
                }

                if !mv.is_capture() {
                    info.killer[1][info.ply as usize] = info.killer[0][info.ply as usize];
                    info.killer[0][info.ply as usize] = *mv;
                }
                // Node (move) fails high
                return beta;
            }
        }
    }
    if legal_move_count == 0 {
        // Possible checkmate or stalemate
        if in_check {
            // Mating score
            // if 49000 is returned, mate is on the board
            // if not, there are ply number of moves before mate is on the board
            return -MATE_VALUE + info.ply as i32;
        } else {
            // Stalemate
            return 0;
        }
    }

    {
        let mut info_tt = info.tt.write().unwrap();
        info_tt.write_entry(board, depth, alpha, tt_flag, info.ply);
    }
    // Node (move) that fails low
    alpha
}

pub fn quiescence(
    info: &mut SearchInfo,
    board: &mut Board,
    attack_info: &AttackInfo,
    mask: &EvalMasks,
    uci_state: &Arc<RwLock<UCIState>>,
    zobrist_info: &ZobristInfo,
    mut alpha: i32,
    beta: i32,
) -> i32 {
    // Communicate with UCI every so often
    if (info.nodes & CHECK_UP_NODES) == 0 {
        let mut info_state = uci_state.write().unwrap();
        info_state.check_up();
    }

    info.nodes += 1;
    // Escape condition
    let eval = eval::evaluate(&board.pos, board.state.side, attack_info, mask);
    // Exit if ply > max ply; ply should be <= 63
    if info.ply > MAX_SEARCH_PLY as u32 - 1 {
        return eval;
    }

    if eval >= beta {
        // Node (move) fails high
        return beta;
    }

    // If found a better move
    if eval > alpha {
        alpha = eval;
    }

    let mut ml = MoveList::new();
    move_gen::generate(board, attack_info, &mut ml);
    sort_moves(info, board, &mut ml);

    let mut clone;
    for mv in &ml.moves {
        clone = board.clone();
        info.ply += 1;
        // Repetition stuff
        // Make sure that every move from this point on is legal
        if !moves::make(board, attack_info, zobrist_info, *mv, MoveFlag::CapturesOnly) {
            info.ply -= 1;
            // Repetition stuff
            continue;
        }
        let score = -quiescence(info, board, attack_info, mask, uci_state, zobrist_info, -beta, -alpha);
        info.ply -= 1;
        // Repetition stuff
        *board = clone;
        // When timer runs out, return 0

        {
            if uci_state.read().unwrap().stop {
                return 0;
            }
        }
        if score > alpha {
            // PV node
            alpha = score;
            if score >= beta {
                // Node (move) fails high
                return beta;
            }
        }
    }
    alpha
}

fn score_move(info: &mut SearchInfo, board: &mut Board, mv: Move) -> u32 {
    if info.score_pv {
        // Check if move on current ply is a PV move
        if info.pv_table[0][info.ply as usize] == mv {
            info.score_pv = false;
            // Give PV move the highest score so as to search it first
            return 20_000;
        }
    }

    if mv.is_capture() {
        // Set to pawn by default; for enpassant
        let mut captured = Piece::LP as usize;
        // Get opponent piece start and end range
        let (start, end) = if board.state.side == PieceColor::Light {
            (Piece::DP as usize, Piece::DK as usize)
        } else {
            (Piece::LP as usize, Piece::LK as usize)
        };
        for i in start..=end {
            if board.pos.piece[i].get(mv.target() as usize) {
                captured = i;
                break;
            }
        }
        // Add 10,000 to ensure captures are evaluated before killer moves
        MVV_LVA[(mv.piece() as usize) % 6][captured % 6] + 10_000
    } else {
        // Score the best killer move
        if info.killer[0][info.ply as usize] == mv {
            9000
        } else if info.killer[1][info.ply as usize] == mv {
            return 8000;
        } else {
            return info.history[mv.piece() as usize][mv.target() as usize];
        }
    }
}

fn sort_moves(info: &mut SearchInfo, board: &mut Board, ml: &mut MoveList) {
    let mut move_score_list: Vec<u32> = vec![];
    for mv in &ml.moves {
        move_score_list.push(score_move(info, board, *mv));
    }

    // Sort moves and their scores in 'descending' order
    for curr in 0..ml.moves.len() {
        for next in (curr + 1)..ml.moves.len() {
            if move_score_list[curr] < move_score_list[next] {
                // Swap scores
                move_score_list.swap(curr, next);
                /* let temp = move_score_list[curr];
                move_score_list[curr] = move_score_list[next];
                move_score_list[next] = temp; */
                // Swap moves
                ml.moves.swap(curr, next);
                /* let temp = ml.moves[curr];
                ml.moves[curr] = ml.moves[next];
                ml.moves[next] = temp; */
            }
        }
    }
}

fn enable_pv_scoring(info: &mut SearchInfo, ml: &mut MoveList) {
    info.follow_pv = false;
    for i in 0..ml.moves.len() {
        if info.pv_table[0][info.ply as usize] == ml.moves[i] {
            info.score_pv = true;
            info.follow_pv = true;
        }
    }
}
