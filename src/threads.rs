use crate::engine::Engine;
use crate::search;

use std::sync::Arc;

pub fn launch_search_thread(engine: &mut Engine, depth: u32) {
    let uci_state = Arc::clone(&engine.uci_state);
    let mut board = engine.board.clone();
    let mut search_info = engine.search_info.clone();
    let attack_info = engine.attack_info.clone();
    let eval_mask = engine.eval_mask.clone();
    let zobrist_info = engine.zobrist_info.clone();
    let th = std::thread::spawn(move || {
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

    engine.search_thread = Some(Box::new(th));
}
