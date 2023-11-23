use crate::board::Board;
use crate::engine::Engine;
use crate::search::{self, SearchInfo};
use crate::uci::UCIState;

use std::sync::{Arc, RwLock};

struct SearchThreadData {
    board: Board,
    search_info: SearchInfo,
    uci_state: Arc<RwLock<UCIState>>,
}

pub fn launch_search_thread(engine: &Engine, depth: u32) {
    let uci_state = Arc::clone(&engine.uci_state);
    let mut board = engine.board.clone();
    let mut search_info = engine.search_info.clone();
    let attack_info = engine.attack_info.clone();
    let eval_mask = engine.eval_mask.clone();
    let zobrist_info = engine.zobrist_info.clone();
    std::thread::spawn(move || {
        search::search(
            &mut search_info,
            &mut board,
            &attack_info,
            &eval_mask,
            &uci_state,
            &zobrist_info,
            depth
        );
    });
}

pub fn join_search_thread() {
    unimplemented!("Join the search thread with the input thread");
}
