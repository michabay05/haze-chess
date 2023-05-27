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

use std::env;

fn main() {
    engine::test();
}
