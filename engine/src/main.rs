use std::io::{self, Write};

use libengine::engine::Engine;
use libengine::uci;

fn main() {
    let mut engine = Engine::new();
    let mut buf = String::new();
    let mut quit = false;

    while !quit {
        let _ = io::stdout().flush();
        if io::stdin().read_line(&mut buf).is_ok() {
            uci::parse(&mut engine, buf.trim(), &mut quit);
        } else {
            eprintln!("[ERROR] Couldn't read input. Please try again!");
        }
        buf.clear();
    }
}
