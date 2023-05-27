mod attack;
mod bb;
mod board;
mod consts;
mod engine;
mod fen;
mod magic_consts;
mod magics;
mod move_gen;
mod moves;
mod perft;
mod perft_test;

use std::env;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    perft_test::test(args);
    // engine::test();
}
