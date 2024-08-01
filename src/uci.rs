use crate::consts::{Piece, PieceColor, Sq};
use crate::engine::Engine;
use crate::eval;
use crate::fen::FEN_POSITIONS;
use crate::move_gen::{self, MoveList};
use crate::moves::{self, Move};
use crate::perft;
use crate::search::{self, MAX_SEARCH_PLY};
use crate::threads;
use crate::VERSION;

use std::time::{Duration, Instant, SystemTime, SystemTimeError};

pub struct UCIState {
    pub stop: bool,
    pub time_controlled: bool,
    pub depth: u32,

    time_left: Option<u32>,
    increment: u32,
    moves_to_go: u32,
    move_time: Option<u32>,
    start_time: u128,
    stop_time: u128,
}

pub fn get_curr_time() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch =
        start
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|e: SystemTimeError| {
                eprintln!("ERROR: System time error\n{}", e);
                Duration::default()
            });
    since_the_epoch.as_millis()
}

impl UCIState {
    pub fn new() -> Self {
        Self {
            stop: false,
            time_controlled: false,

            time_left: None,
            increment: 0,
            moves_to_go: 40,
            move_time: None,
            start_time: 0,
            stop_time: 0,
            depth: 0,
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
        self.stop = false;
    }
}

pub fn parse(engine: &mut Engine, input_str: &str, should_quit: &mut bool) {
    let ind = split_by_first_space(input_str);
    let rest = input_str[ind..].trim();

    match &input_str[0..ind] {
        "quit" => {
            *should_quit = true;
            {
                if let Ok(mut engine_state) = engine.uci_state.write() {
                    engine_state.stop = true;
                }
            }
            if let Some(th) = engine.search_thread.take() {
                let _ = th.join();
            }
        }
        "stop" => {
            {
                if let Ok(mut engine_state) = engine.uci_state.write() {
                    engine_state.stop = true;
                }
            }
            if let Some(th) = engine.search_thread.take() {
                let _ = th.join();
            }
            if engine.debug {
                println!("Stopping calculation . . .");
                println!("Search thread has joined the input thread.");
            }
        }
        "ucinewgame" => parse_position(engine, "startpos"),
        "uci" => print_author_info(),
        "isready" => println!("readyok"),
        "position" => parse_position(engine, &rest),
        "go" => parse_go(engine, &rest),
        "evalpos" => {
            let eval = eval::evaluate(
                &engine.board.pos,
                engine.board.state.side,
                &engine.attack_info,
                &engine.eval_mask,
            );
            println!("Current eval: {eval}");
        }
        "genmoves" => {
            let mut ml = MoveList::new();
            move_gen::generate(&engine.board, &engine.attack_info, &mut ml);
            ml.print();
        }
        "display" | "d" => engine.board.display(),
        "help" => print_help(),
        "debug" => match rest {
            "on" => {
                engine.debug = true;
                println!("Debug mode on!");
            }
            "off" => engine.debug = false,
            _ => {}
        },
        _ => {}
    }
}

fn parse_position(engine: &mut Engine, args: &str) {
    let ind = split_by_first_space(args);
    let first_arg = &args[0..ind];
    let rest = &args[ind..].trim();

    match first_arg {
        "startpos" => {
            engine.board.set_fen(FEN_POSITIONS[1]);
        }
        "fen" => {
            engine.board.set_fen(&rest.trim());
        }
        // TODO: figure out the best course of action here
        // Is it better to ignore an unknown argument or display a message stating it?
        // Good for human use, not so good when interacting with a GUI program use
        _ => {}
    }

    if let Some(i) = rest.find("moves") {
        parse_moves(engine, &rest[i..]);
    }
    // Clear the transposition table to setup for the new position
    if let Ok(mut engine_tt) = engine.search_info.tt.write() {
        engine_tt.clear_table();
    }
}

fn parse_moves(engine: &mut Engine, args: &str) {
    let ind = split_by_first_space(args);
    let rest = &args[ind..].trim();
    let list_of_moves = rest.split(' ');

    for el in list_of_moves {
        // If current move is a promotion
        // If white is promoting - if target square has an '8', it means the pawn is being promoted on the 8th rank
        let mv_str = if el.trim().len() == 5 && el.chars().nth(3).unwrap() == '8' {
            format!(
                "{}{}",
                &el[0..4],
                el.chars().nth(4).unwrap().to_ascii_uppercase()
            )
        } else {
            el.to_string()
        };
        if let Some(mv) = find_move(engine, &mv_str) {
            moves::play_move(
                &mut engine.board,
                &engine.attack_info,
                mv,
                moves::MoveType::AllMoves,
            );
        } else {
            eprintln!("Received '{mv_str}'. Unknown move.");
        }
    }
}

fn find_move(engine: &Engine, move_str: &str) -> Option<Move> {
    if move_str.len() != 4 && move_str.len() != 5 {
        println!("Got {} expected move string with length 4 or 5", move_str);
        return None;
    }
    let mut ml = move_gen::MoveList::new();
    move_gen::generate(&engine.board, &engine.attack_info, &mut ml);
    let source = &move_str[0..2];
    let target = &move_str[2..4];
    let promoted = if move_str.len() == 5 {
        Piece::from_char(move_str.chars().nth(4).unwrap())
    } else {
        None
    };
    let source = Sq::from_str(source);
    let target = Sq::from_str(target);
    if source.is_some() && target.is_some() {
        ml.search(source.unwrap(), target.unwrap(), promoted)
    } else {
        None
    }
}

fn parse_go(engine: &mut Engine, args: &str) {
    let first_ind = split_by_first_space(args);

    if &args[0..first_ind] == "perft" {
        let depth_str = &args[first_ind..].trim();
        let perft_depth: usize = if let Ok(val) = depth_str.parse() {
            val
        } else {
            10
        };
        let start = Instant::now();
        let nodes = perft::driver(
            &mut engine.board,
            &engine.attack_info,
            perft_depth,
            engine.debug
        );
        let dur = Instant::now().duration_since(start);
        println!("   Nodes: {}", nodes);
        println!("    Time: {}", dur.as_secs_f32());
        return;
    }
    handle_time(engine, args);
    let depth = parse_param(args, "depth").unwrap_or(search::MAX_SEARCH_PLY as u32);

    if let Ok(mut state) = engine.uci_state.write() {
        state.depth = depth;
        if state.move_time.is_some() {
            state.time_left = state.move_time;
            state.moves_to_go = 1;
        }

        // Example UCI command with time
        // go depth 12 wtime 180000 btime 180000 binc 1000 winc 1000 movestogo 40
        // go movetime 1000
        state.start_time = get_curr_time();
        if let Some(tl) = state.time_left {
            state.time_controlled = true;
            let mut per_move_tl = tl / state.moves_to_go;
            if per_move_tl > 1500 {
                per_move_tl -= 50;
            }
            state.time_left = Some(per_move_tl);
            state.stop_time = state.start_time + (per_move_tl + state.increment) as u128;
            if per_move_tl < 1500 && state.increment != 0 && state.depth == MAX_SEARCH_PLY as u32 {
                let a = state.increment.saturating_sub(50);
                state.stop_time = state.start_time + a as u128;
            }
        }

        // Print debug info
        if engine.debug {
            println!(
                "info string time: {}, start: {}, stop: {}, depth: {}, timecontrol: {}",
                state.time_left.unwrap_or(0),
                state.start_time,
                state.stop_time,
                depth,
                if state.time_controlled { "yes" } else { "no" }
            );
        }
    }
    threads::launch_search_thread(engine, depth);
}

fn handle_time(engine: &mut Engine, cmd: &str) {
    if let Ok(mut state) = engine.uci_state.write() {
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
}

fn parse_param(cmd: &str, name: &str) -> Option<u32> {
    let mut val: Option<u32> = None;
    if let Some(ind) = cmd.find(name) {
        let portion = &cmd[ind..];
        let i = split_by_first_space(portion);
        let name_portion = (&portion[i..]).trim();
        let ind2 = split_by_first_space(name_portion);
        if let Ok(num) = (&name_portion[0..ind2]).parse::<u32>() {
            val = Some(num);
        }
    }
    return val;
}

pub fn print_author_info() {
    println!("id name haze {}", VERSION);
    println!("id author michabay05");
    println!("option name Hash type spin default 256 min 1 max 1024");
    println!("option name Thread type spin default 1 min 1 max 4");
    println!("uciok");
}

fn print_help() {
    println!();
    println!("              Command name               |               Description");
    println!("=========================================|=============================================================");
    println!("                  uci                    |    Returns engine info accompanied with 'uciok'");
    println!(
        "              isready                    |    Returns 'readyok' if the engine is ready"
    );
    println!("    position startpos                    |    Set board to starting position");
    println!("    position startpos moves <move1> ...  |    Set board to starting position then playing the following moves");
    println!("   position fen <FEN>                    |    Set board to a custom FEN");
    println!("   position fen <FEN> moves <move1> ...  |    Set board to a custom FEN then playing the following moves");
    println!("     go depth <depth>                    |    Returns the best move after search for given amount of depth");
    println!("                debug [ on | off ]       |    Sends additional information when needed. Off by default");
    println!("                 stop                    |    Stops engine from calculating further");
    println!("                 quit                    |    Exit the UCI mode\n");
    println!(
        "------------------------------------ EXTENSIONS ----------------------------------------"
    );
    println!("              display                    |    Display board");
    println!("     go perft <depth>                    |    Calculate the total number of moves from a position for a given depth");
}

fn split_by_first_space(input_str: &str) -> usize {
    return input_str.find(' ').unwrap_or(input_str.len());
}
