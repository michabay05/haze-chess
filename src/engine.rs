use crate::attack::{self, AttackInfo};
use crate::board::Board;
use crate::consts::{PieceColor, Sq};
use crate::fen::FEN_POSITIONS;

pub struct Engine {
    pub attack: AttackInfo,
    pub board: Board,
}

impl Engine {
    fn new() -> Self {
        let mut this = Self {
            attack: AttackInfo::new(),
	    board: Board::from_fen(FEN_POSITIONS[2]),
        };
        this.init();
        this
    }

    fn init(&mut self) {
        attack::init(&mut self.attack);
    }
}

pub fn test() {
    let mut engine = Engine::new();
    engine.board.display();
}
