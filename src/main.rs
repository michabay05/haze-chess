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
mod tt;
mod uci;
mod zobrist;

use std::io::{self, Write};

use engine::Engine;

const VERSION: &str = "1.0";

fn main() {
    let mut engine = Engine::new();
    let mut buf = String::new();

    while !engine.uci_state.quit {
        let _ = io::stdout().flush();
        if let Ok(_) = io::stdin().read_line(&mut buf) {
            uci::parse(&mut engine, buf.trim());
        } else {
            eprintln!("[ERROR] Couldn't read input. Please try again!");
        }
        buf.clear();
    }
}
