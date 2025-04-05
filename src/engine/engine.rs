use chess::attack::AttackInfo;
use chess::board::Board;
use crate::eval::EvalMasks;
use crate::search::SearchInfo;
use crate::uci::UCIState;

use crate::NUM_OF_THREADS;

use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

pub struct Engine {
    pub attack_info: AttackInfo,
    pub board: Board,
    pub eval_mask: EvalMasks,
    pub search_info: SearchInfo,
    // pub zobrist_info: ZobristInfo,
    pub uci_state: Arc<RwLock<UCIState>>,
    pub search_thread: Option<JoinHandle<()>>,
    pub worker_thread_count: usize,

    pub debug: bool,
}

impl Engine {
    pub fn new() -> Self {
        let mut this = Self {
            attack_info: AttackInfo::new(),
            board: Board::new(),
            eval_mask: EvalMasks::new(),
            search_info: SearchInfo::new(),
            uci_state: Arc::new(RwLock::new(UCIState::new())),
            search_thread: None,
            worker_thread_count: NUM_OF_THREADS,
            debug: false,
        };
        // Initialize attributes
        this.attack_info.init();
        this.eval_mask.init();

        this
    }
}
