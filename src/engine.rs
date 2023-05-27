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
    pub fn new() -> Self {
        let mut this = Self {
            attack_info: AttackInfo::new(),
	    board: Board::new(),
        };
        this.init();
        this
    }

    fn init(&mut self) {
        attack::init(&mut self.attack_info);
    }

    pub fn set_fen(&mut self, fen: &str) {
	self.board = Board::from_fen(fen);
    }
}


use crate::move_gen::{self, MoveList};
use crate::consts::Piece;
use crate::bb::BBUtil;
pub fn test() {
    let mut engine = Engine::new();
    // engine.set_fen(FEN_POSITIONS[1]);
    engine.set_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    engine.board.display();

    let mut ml = MoveList::new();
    move_gen::generate(&engine.board, &engine.attack_info, &mut ml);
    ml.print();
}
