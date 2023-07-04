use crate::attack::AttackInfo;
use crate::bb::BBUtil;
use crate::board::{self, Board};
use crate::consts::{Piece, PieceColor, Sq};
use crate::eval::{self, EvalMasks};
use crate::move_gen::{self, MoveList};
use crate::moves::{self, Move, MoveFlag, MoveUtil};
use crate::tt::{HashTT, TTFlag};
use crate::uci::UCIState;
use crate::zobrist::{ZobristInfo, self, ZobristAction};

const FULL_DEPTH_MOVES: u32 = 4;
const REDUCTION_LIMIT: u32 = 3;
pub const MAX_PLY: usize = 64;

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

pub struct SearchInfo {
    // Half move counter
    pub ply: u32,
    pub nodes: u32,
    // PV flags
    pub follow_pv: bool,
    pub score_pv: bool,
    // 'Quiet' moves that cause a beta-cutoffs
    pub killer: [[Move; MAX_PLY]; 2], // [id][ply]
    pub history: [[Move; 64]; 12],    // [piece][sq]
    pub pv_len: [u32; MAX_PLY],
    pub pv_table: [[u32; MAX_PLY]; MAX_PLY],
    pub tt: HashTT,
}

impl SearchInfo {
    pub fn new() -> Self {
        Self {
            ply: 0,
            nodes: 0,
            follow_pv: false,
            score_pv: false,
            killer: [[0; MAX_PLY]; 2],
            history: [[0; 64]; 12],
            pv_len: [0; MAX_PLY],
            pv_table: [[0; MAX_PLY]; MAX_PLY],
            tt: HashTT::new(),
        }
    }
}

pub fn search(
    info: &mut SearchInfo,
    board: &mut Board,
    attack_info: &AttackInfo,
    mask: &EvalMasks,
    uci_state: &mut UCIState,
    zobrist_info: &ZobristInfo,
    depth: u32,
) {
    *info = SearchInfo::new();
    uci_state.stop = false;
    let mut alpha = -INFINITY;
    let mut beta = INFINITY;

    let mut score;
    for current_depth in 1..=depth {
        if uci_state.stop {
            break;
        }
        info.follow_pv = true;
        // Find the best move in the current position
        score = negamax(
            info,
            board,
            attack_info,
            mask,
            uci_state,
            zobrist_info,
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

        if info.pv_len[0] != 0 {
            let (cp_score, cp_str) = if score > -MATE_VALUE && score < -MATE_SCORE {
                (-(score + MATE_VALUE) / 2 - 1, "mate")
            } else if score > MATE_VALUE && score < MATE_SCORE {
                (((MATE_VALUE - score) / 2 + 1) as i32, "mate")
            } else {
                (score as i32, "cp")
            };
            print!(
                "info score {} {} depth {} nodes {} pv ",
                cp_str, cp_score, current_depth, info.nodes
            );
            // print!("info score {} {} depth {} nodes {} time {}  pv ", cp_str, cp_score, current_depth, info.nodes, info.time);
            // Print principal variation
            for i in 0..(info.pv_len[0] as usize) {
                print!("{} ", info.pv_table[0][i].to_str().trim());
            }
            println!();
        }
    }
    println!("bestmove {}", info.pv_table[0][0].to_str().trim());
}

fn negamax(
    info: &mut SearchInfo,
    board: &mut Board,
    attack_info: &AttackInfo,
    mask: &EvalMasks,
    uci_state: &mut UCIState,
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
    let hash_score = info.tt.read_entry(board, alpha, beta, depth, info.ply);
    if info.ply != 0 && !hash_score.is_none() && !is_pv_node {
        return hash_score.unwrap();
    }

    // Every 2047 nodes, communicate with UCI
    if (info.nodes & 2047) == 0 {
        uci_state.check_up();
    }

    // Escape condition or Base case
    if depth == 0 {
        return quiescence(info, board, attack_info, mask, uci_state, zobrist_info, alpha, beta);
    }
    // Exit if ply > max ply; ply should be <= 63
    if info.ply > (MAX_PLY - 1) as u32 {
        return eval::evaluate(&board.pos, board.state.side, attack_info, mask);
    }
    // Increment nodes
    info.nodes += 1;

    // Check extension
    let king_type = if board.state.side == PieceColor::Light {
        Piece::LK
    } else {
        Piece::DK
    } as usize;
    let is_in_check = board::sq_attacked(
        &board.pos,
        attack_info,
        Sq::from_num(board.pos.piece[king_type].lsb()),
        board.state.xside,
    );

    if is_in_check {
        depth += 1
    }
    let mut legal_move_count = 0;

    // NULL move pruning
    if depth >= 3 && !is_in_check && info.ply != 0 {
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
        if uci_state.stop {
            return 0;
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
                && !is_in_check
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
        if uci_state.stop {
            return 0;
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
                info.tt.write_entry(&board, depth, beta, TTFlag::Beta, info.ply);
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
        if is_in_check {
            // Mating score
            // if 49000 is returned, mate is on the board
            // if not, there are ply number of moves before mate is on the board
            return -MATE_VALUE + info.ply as i32;
        } else {
            // Stalemate
            return 0;
        }
    }
    info.tt.write_entry(&board, depth, alpha, tt_flag, info.ply);
    // Node (move) that fails low
    alpha
}

pub fn quiescence(
    info: &mut SearchInfo,
    board: &mut Board,
    attack_info: &AttackInfo,
    mask: &EvalMasks,
    uci_state: &mut UCIState,
    zobrist_info: &ZobristInfo,
    mut alpha: i32,
    beta: i32,
) -> i32 {
    // Every 2047 nodes, communicate with UCI
    if (info.nodes & 2047) == 0 {
        uci_state.check_up();
    }
    info.nodes += 1;
    // Escape condition
    let eval = eval::evaluate(&board.pos, board.state.side, attack_info, mask);
    // Exit if ply > max ply; ply should be <= 63
    if info.ply > MAX_PLY as u32 - 1 {
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
        if uci_state.stop {
            return 0;
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
        return MVV_LVA[(mv.piece() as usize) % 6][captured % 6] + 10_000;
    } else {
        // Score the best killer move
        if info.killer[0][info.ply as usize] == mv {
            return 9000;
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
                let temp = move_score_list[curr];
                move_score_list[curr] = move_score_list[next];
                move_score_list[next] = temp;
                // Swap moves
                let temp = ml.moves[curr];
                ml.moves[curr] = ml.moves[next];
                ml.moves[next] = temp;
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
