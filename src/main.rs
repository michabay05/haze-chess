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

use std::env;

fn main() {
    let mut engine = engine::Engine::new();
    engine.test();
}
