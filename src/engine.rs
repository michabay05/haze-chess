use crate::attack::AttackInfo;
use crate::board::Board;
use crate::eval::EvalMasks;
use crate::search::SearchInfo;
use crate::uci::UCIState;
use crate::zobrist::ZobristInfo;

use std::sync::{Arc, RwLock};

pub struct Engine {
    pub attack_info: AttackInfo,
    pub board: Board,
    pub eval_mask: EvalMasks,
    pub search_info: SearchInfo,
    pub zobrist_info: ZobristInfo,
    pub uci_state: Arc<RwLock<UCIState>>,
}

impl Engine {
    pub fn new() -> Self {
        let mut this = Self {
            attack_info: AttackInfo::new(),
            board: Board::new(),
            eval_mask: EvalMasks::new(),
            search_info: SearchInfo::new(),
            zobrist_info: ZobristInfo::new(),
            uci_state: Arc::new(RwLock::new(UCIState::new())),
        };
        // Initialize attributes
        this.attack_info.init();
        this.eval_mask.init();
        this.zobrist_info.init();

        this
    }
}
