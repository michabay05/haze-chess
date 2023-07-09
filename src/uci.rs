<<<<<<< HEAD
use crate::board::Board;
use crate::consts::{Piece, PieceColor, Sq};
use crate::engine::Engine;
use crate::fen::FEN_POSITIONS;
use crate::{move_gen, moves, perft, search::{self, MAX_PLY}};
=======
use crate::board::{self, Board};
use crate::consts::{Piece, PieceColor, Sq};
use crate::engine::Engine;
use crate::fen::FEN_POSITIONS;
use crate::move_gen::MoveList;
use crate::{eval, VERSION};
use crate::{
    move_gen,
    moves::{self, Move},
    perft,
    search::{self, MAX_SEARCH_PLY},
};
>>>>>>> eb57ea2 (Completed version 1.0 of the engine)

use std::time::{SystemTime, UNIX_EPOCH};

pub struct UCIState {
    pub stop: bool,
    pub quit: bool,
    pub is_infinite: bool,
    pub time_controlled: bool,

    time_left: Option<u32>,
    increment: u32,
    moves_to_go: u32,
    move_time: Option<u32>,
    start_time: u128,
    stop_time: u128,
}

<<<<<<< HEAD
fn get_curr_time() -> u128 {
=======
pub fn get_curr_time() -> u128 {
>>>>>>> eb57ea2 (Completed version 1.0 of the engine)
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    since_the_epoch.as_millis()
}

impl UCIState {
    pub fn new() -> Self {
        Self {
            stop: false,
            quit: false,
            is_infinite: false,
            time_controlled: false,

            time_left: None,
            increment: 0,
            moves_to_go: 40,
            move_time: None,
            start_time: 0,
            stop_time: 0,
        }
    }

    pub fn check_up(&mut self) {
        if self.time_controlled && get_curr_time() >= self.stop_time {
            self.stop = true;
        }
    }

    fn reset_time_control(&mut self) {
        self.moves_to_go = 40;
        self.time_left = None;
        self.move_time = None;
        self.increment = 0;
        self.start_time = 0;
        self.stop_time = 0;

        self.time_controlled = false;
        self.quit = false;
        self.stop = false;
    }
}

pub fn parse(engine: &mut Engine, input_str: &str) {
    let (first_arg, rest) = first_and_rest(input_str);

    if first_arg == "quit" {
        engine.uci_state.quit = true;
    } else if first_arg == "stop" {
        engine.uci_state.stop = true;
    } else if first_arg == "ucinewgame" {
        parse_position(engine, "startpos");
<<<<<<< HEAD
        engine.search_info.tt.clear_table();
    } else if first_arg == "uci" {
        print_author_info();
        engine.search_info.tt.clear_table();
=======
    } else if first_arg == "uci" {
        print_author_info();
>>>>>>> eb57ea2 (Completed version 1.0 of the engine)
    } else if first_arg == "isready" {
        println!("readyok");
    } else if first_arg == "position" {
        parse_position(engine, &rest);
    } else if first_arg == "go" {
        parse_go(engine, &rest);
<<<<<<< HEAD
=======
    } else if first_arg == "evalpos" {
        let eval = eval::evaluate(
            &engine.board.pos,
            engine.board.state.side,
            &engine.attack_info,
            &engine.eval_mask,
        );
        println!("Current eval: {eval}");
    } else if first_arg == "genmoves" {
        let mut ml = MoveList::new();
        move_gen::generate(&engine.board, &engine.attack_info, &mut ml);
        ml.print();
>>>>>>> eb57ea2 (Completed version 1.0 of the engine)
    } else if (first_arg == "display") || (first_arg == "d") {
        engine.board.display();
    } else if first_arg == "help" {
        print_help();
<<<<<<< HEAD
    } else {
        eprintln!("ERROR: Incorrect command.\nType 'help' to get the list of commands");
=======
>>>>>>> eb57ea2 (Completed version 1.0 of the engine)
    }
}

fn parse_position(engine: &mut Engine, args: &str) {
    let (first_arg, rest) = first_and_rest(args);

    if first_arg == "startpos" {
        engine.board = Board::from_fen(FEN_POSITIONS[1], &engine.zobrist_info);
    } else if first_arg == "fen" {
<<<<<<< HEAD
        engine.board = Board::from_fen(args, &engine.zobrist_info);
    } else {
        eprintln!("ERROR: Incorrect command.");
        print_help();
    }

    if !rest.is_empty() {
        parse_moves(engine, &rest);
    }
}

fn parse_moves(engine: &mut Engine, args: &str) {
    let (first_arg, rest) = first_and_rest(args);
    if first_arg != "moves" {
        eprintln!("ERROR: Incorrect command.");
        print_help();
        return;
    }
    let list_of_moves = rest.split_whitespace();
    for el in list_of_moves {
        let mv = if let Some(val) = find_move(engine, el) {
            val
        } else {
            panic!("ERROR: Failed to parse '{el}' as a valid move in the current position");
=======
        let ind = args.find("fen").unwrap();
        engine.board = Board::from_fen(&args[(ind + first_arg.len() + 1)..], &engine.zobrist_info);
    }

    if rest.find("moves").is_some() {
        parse_moves(engine, &rest);
    }
    engine.search_info.tt.clear_table();
}

fn parse_moves(engine: &mut Engine, args: &str) {
    let (_, rest) = first_and_rest(args);
    let list_of_moves = rest.split(" ");

    for el in list_of_moves {
        // If current move is a promotion
        // If white is promoting - if target square has an '8', it means the pawn is being promoted on the 8th rank
        let mv_str = if el.trim().len() == 5 && el.chars().nth(3).unwrap() == '8' {
            println!("YES");
            format!(
                "{}{}",
                &el[0..4],
                el.chars().nth(4).unwrap().to_ascii_uppercase()
            )
        } else {
            el.to_string()
        };
        let mv = if let Some(val) = find_move(engine, &mv_str) {
            val
        } else {
            panic!("Failed to find move '{}'", mv_str);
>>>>>>> eb57ea2 (Completed version 1.0 of the engine)
        };
        moves::make(
            &mut engine.board,
            &engine.attack_info,
            &engine.zobrist_info,
            mv,
            moves::MoveFlag::AllMoves,
        );
    }
}

<<<<<<< HEAD
fn find_move(engine: &Engine, move_str: &str) -> Option<moves::Move> {
=======
fn find_move(engine: &Engine, move_str: &str) -> Option<Move> {
>>>>>>> eb57ea2 (Completed version 1.0 of the engine)
    assert!(move_str.len() == 4 || move_str.len() == 5);
    let mut ml = move_gen::MoveList::new();
    move_gen::generate(&engine.board, &engine.attack_info, &mut ml);
    let source = &move_str[0..2];
    let target = &move_str[2..4];
    let promoted = if move_str.len() == 5 {
        Piece::from_char(move_str.chars().nth(4).unwrap())
    } else {
        None
    };
    ml.search(Sq::from_str(source), Sq::from_str(target), promoted)
}

fn parse_go(engine: &mut Engine, args: &str) {
    let (first_arg, first_rest) = first_and_rest(args);
<<<<<<< HEAD
    let (second_arg, second_rest) = first_and_rest(&first_rest);

    if first_arg == "perft" {
=======

    if first_arg == "perft" {
        let (second_arg, _) = first_and_rest(&first_rest);
>>>>>>> eb57ea2 (Completed version 1.0 of the engine)
        let perft_depth: usize = if let Ok(val) = second_arg.parse() {
            val
        } else {
            10
        };
<<<<<<< HEAD
        perft::test(&mut engine.board, &engine.attack_info, &engine.zobrist_info, perft_depth);
    }
    handle_time(engine, &second_rest);
    let depth = parse_param(&second_rest, "depth").unwrap_or(search::MAX_PLY as u32);
=======
        perft::test(
            &mut engine.board,
            &engine.attack_info,
            &engine.zobrist_info,
            perft_depth,
        );
        return;
    }
    handle_time(engine, args);
    let depth = parse_param(&args, "depth").unwrap_or(search::MAX_SEARCH_PLY as u32);
>>>>>>> eb57ea2 (Completed version 1.0 of the engine)
    let state = &mut engine.uci_state;
    if state.move_time.is_some() {
        state.time_left = state.move_time;
        state.moves_to_go = 1;
    }

    // Example UCI command with time
    // go depth 12 wtime 180000 btime 100000 binc 1000 winc 1000 movetime 1000 movestogo 40
<<<<<<< HEAD
    // go depth 12 movetime 1000
=======
    // go movetime 1000
>>>>>>> eb57ea2 (Completed version 1.0 of the engine)
    state.start_time = get_curr_time();
    if state.time_left.is_some() {
        state.time_controlled = true;
        state.time_left = Some(state.time_left.unwrap() / state.moves_to_go);
        if state.time_left.unwrap() > 1500 {
            state.time_left = Some(state.time_left.unwrap() - 50);
        }
        state.stop_time = state.start_time + (state.time_left.unwrap() + state.increment) as u128;
<<<<<<< HEAD
        if state.time_left.unwrap() < 1500 && state.increment != 0 && depth == MAX_PLY as u32 {
=======
        if state.time_left.unwrap() < 1500 && state.increment != 0 && depth == MAX_SEARCH_PLY as u32
        {
>>>>>>> eb57ea2 (Completed version 1.0 of the engine)
            let mut a = state.increment as i32 - 50;
            if a <= 0 {
                a = 0;
            }
            state.stop_time = state.start_time + a as u128;
        }
    }
<<<<<<< HEAD
=======

    // Print debug info
    println!(
        "time: {},  start: {},  stop: {},  depth: {},  timeset: {}",
        if state.time_left.is_some() {state.time_left.unwrap() as i32} else {-1},
        state.start_time,
        state.stop_time,
        depth,
        if state.time_controlled { '1' } else { '0' }
    );
>>>>>>> eb57ea2 (Completed version 1.0 of the engine)
    search::search(
        &mut engine.search_info,
        &mut engine.board,
        &engine.attack_info,
        &engine.eval_mask,
        &mut engine.uci_state,
        &engine.zobrist_info,
        depth,
    );
}

fn handle_time(engine: &mut Engine, cmd: &str) {
    let state = &mut engine.uci_state;
    state.reset_time_control();
    if engine.board.state.side == PieceColor::Light {
        state.time_left = parse_param(cmd, "wtime");
        state.increment = parse_param(cmd, "winc").unwrap_or(0);
    } else {
        state.time_left = parse_param(cmd, "btime");
        state.increment = parse_param(cmd, "binc").unwrap_or(0);
    }
    state.move_time = parse_param(cmd, "movetime");
    state.moves_to_go = parse_param(cmd, "movestogo").unwrap_or(40);
}

fn parse_param(cmd: &str, name: &str) -> Option<u32> {
    let ind = cmd.find(name);
    if ind.is_none() {
        return None;
    }
    let ind = ind.unwrap();
    let portion = &cmd[(ind)..];
    let (_, other_args) = first_and_rest(portion);
    let (val, _) = first_and_rest(&other_args);
    if let Ok(num) = val.parse::<u32>() {
        Some(num)
    } else {
        None
    }
}

pub fn print_author_info() {
<<<<<<< HEAD
    println!("id name haze");
=======
    println!("id name haze {}", VERSION);
>>>>>>> eb57ea2 (Completed version 1.0 of the engine)
    println!("id author michabay05");
    println!("uciok");
}

fn print_help() {
    println!();
    println!("              Command name               |         Description");
    println!("-------------------------------------------------------------------------------------------------------");
    println!("                  uci                    |    Prints engine info and 'uciok'");
    println!(
        "              isready                    |    Prints 'readyok' if the engine is ready"
    );
    println!("    position startpos                    |    Set board to starting position");
    println!("    position startpos moves <move1> ...  |    Set board to starting position then playing following moves");
    println!("   position fen <FEN>                    |    Set board to a custom FEN");
    println!("   position fen <FEN> moves <move1> ...  |    Set board to a custom FEN then playing following moves");
    println!("     go depth <depth>                    |    Returns the best move after search for given amount of depth");
<<<<<<< HEAD
=======
    println!("                 stop                    |    Stops engine from calculating further");
>>>>>>> eb57ea2 (Completed version 1.0 of the engine)
    println!("                 quit                    |    Exit the UCI mode\n");
    println!(
        "------------------------------------ EXTENSIONS ----------------------------------------"
    );
    println!("              display                    |    Display board");
    println!("     go perft <depth>                    |    Calculate the total number of moves from a position for a given depth");
}

fn first_and_rest(input_str: &str) -> (String, String) {
    let space_ind = input_str.find(" ").unwrap_or(input_str.len());
    let first = &input_str[0..space_ind];
    let rest = input_str[space_ind..].trim();
    (first.to_string(), rest.to_string())
}
