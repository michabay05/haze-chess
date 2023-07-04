mod attack;
mod bb;
mod board;
mod consts;
mod engine;
mod eval;
mod eval_consts;
mod fen;
mod magic_consts;
mod magics;
mod move_gen;
mod moves;
mod perft;
mod search;
mod uci;
mod tt;
mod zobrist;

use std::io::{self, Write};
use engine::Engine;

fn main() {
    let mut engine = engine::Engine::new();
    run(&mut engine)
}


pub fn run(engine: &mut Engine) {
    while !engine.uci_state.quit {
        let mut buf = String::new();
        let _ = io::stdout().flush();
        io::stdin().read_line(&mut buf).unwrap();

        uci::parse(engine, buf.trim());
    }
}
