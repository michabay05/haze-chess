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
use std::thread;

use engine::Engine;

const VERSION: &str = "1.0";

fn run() {
    let mut engine = engine::Engine::new();
    start(&mut engine);
}

fn main() {
    // Stack size of 8MB
    const STACK_SIZE: usize = 8 * 1024 * 1024;

    // Spawn thread with explicit stack size
    let child = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(run)
        .unwrap();

    // Wait for thread to join
    child.join().unwrap();
}

pub fn start(engine: &mut Engine) {
    while !engine.uci_state.quit {
        let mut buf = String::new();
        let _ = io::stdout().flush();
        io::stdin().read_line(&mut buf).unwrap();

        uci::parse(engine, buf.trim());
    }
}
