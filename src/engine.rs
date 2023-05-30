use crate::attack::{self, AttackInfo};
use crate::board::Board;
use crate::consts::{PieceColor, Sq};
use crate::eval::{self, EvalMasks};
use crate::fen::FEN_POSITIONS;
use crate::perft;
use crate::search::{self, SearchInfo};

use std::io::{self, Write};

pub struct Engine {
    attack_info: AttackInfo,
    board: Board,
    eval_mask: EvalMasks,
    search_info: SearchInfo,
}

impl Engine {
    pub fn new() -> Self {
        let mut this = Self {
            attack_info: AttackInfo::new(),
            board: Board::new(),
            eval_mask: EvalMasks::new(),
            search_info: SearchInfo::new(),
        };
        this.init();
        this
    }

    fn init(&mut self) {
        self.attack_info.init();
        self.eval_mask.init();
    }

    pub fn set_fen(&mut self, fen: &str) {
        self.board = Board::from_fen(fen);
    }

    pub fn test(&mut self) {
	loop {
	    let mut buf = String::new();
	    print!("FEN: ");
	    let _ = io::stdout().flush();
	    io::stdin().read_line(&mut buf).unwrap();
	    let line = buf.trim();
	    if line == "q" {
		break;
	    }
	    self.set_fen(line);

	    search::search(
		&mut self.search_info,
		&mut self.board,
		&self.attack_info,
		&self.eval_mask,
		12,
	    );
	    println!("==============================================");
	}
    }
}
