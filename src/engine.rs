use crate::attack::{self, AttackInfo};
use crate::board::Board;
use crate::consts::{PieceColor, Sq};
use crate::fen::FEN_POSITIONS;
use crate::perft;

pub struct Engine {
    pub attack_info: AttackInfo,
    pub board: Board,
}

impl Engine {
    fn new() -> Self {
        let mut this = Self {
            attack_info: AttackInfo::new(),
            board: Board::from_fen(FEN_POSITIONS[2]),
        };
        this.init();
        this
    }

    fn init(&mut self) {
        attack::init(&mut self.attack_info);
    }
}


pub fn test() {
    let mut engine = Engine::new();
    engine.board.display();

    perft::test(&mut engine.board, &engine.attack_info, 1);
}
