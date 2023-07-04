use crate::attack::AttackInfo;
use crate::board::Board;
use crate::eval::EvalMasks;
use crate::search::SearchInfo;
use crate::uci::UCIState;
use crate::zobrist::ZobristInfo;

pub struct Engine {
    pub attack_info: AttackInfo,
    pub board: Board,
    pub eval_mask: EvalMasks,
    pub search_info: SearchInfo,
    pub zobrist_info: ZobristInfo,
    pub uci_state: UCIState,
}

impl Engine {
    pub fn new() -> Self {
        let mut this = Self {
            attack_info: AttackInfo::new(),
            board: Board::new(),
            eval_mask: EvalMasks::new(),
            search_info: SearchInfo::new(),
            zobrist_info: ZobristInfo::new(),
            uci_state: UCIState::new(),
        };
        this.init();
        this
    }

    fn init(&mut self) {
        self.attack_info.init();
        self.eval_mask.init();
        self.zobrist_info.init();
    }
}