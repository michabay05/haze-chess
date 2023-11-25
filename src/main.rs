mod attack;
mod bb;
mod board;
mod consts;
mod engine;
mod eval;
mod fen;
mod magics;
mod move_gen;
mod moves;
mod perft;
mod search;
mod threads;
mod tt;
mod uci;
mod zobrist;

use std::io::{self, Write};

use engine::Engine;

const VERSION: &str = "1.0";
const NUM_OF_THREADS: usize = 3;

fn main() {
    let mut engine = Engine::new();
    let mut buf = String::new();
    let mut quit = false;

    while !quit {
        let _ = io::stdout().flush();
        if let Ok(_) = io::stdin().read_line(&mut buf) {
            let buf = buf.trim();
            uci::parse(&mut engine, buf, &mut quit);
        } else {
            eprintln!("[ERROR] Couldn't read input. Please try again!");
        }
        buf.clear();
    }
}
