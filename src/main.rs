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
mod prng;
mod search;
mod threads;
mod tt;
mod uci;
mod zobrist;

use std::io::{self, Write};

use engine::Engine;

const VERSION: &str = "0.3";
const NUM_OF_THREADS: usize = 1;

fn main() {
    let mut engine = Engine::new();
    let mut buf = String::new();
    let mut quit = false;

    while !quit {
        let _ = io::stdout().flush();
        if io::stdin().read_line(&mut buf).is_ok() {
            let buf = buf.trim();
            uci::parse(&mut engine, buf, &mut quit);
        } else {
            eprintln!("[ERROR] Couldn't read input. Please try again!");
        }
        buf.clear();
    }
}
