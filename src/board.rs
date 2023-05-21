use crate::SQ;
use crate::bb::{BBUtil, BB};
use crate::consts::{Piece, PieceColor, Sq};
use crate::fen;

pub struct Position {
    pub piece: [BB; 12],
    pub units: [BB; 3],
}

impl Position {
    pub fn new() -> Self {
        Position {
            piece: [0; 12],
            units: [0; 3],
        }
    }

    pub fn update_units(&mut self) {
        self.units.fill(0);
        for i in 0..12 {
	    self.units[i / 6] |= self.piece[i];
        }
        self.units[PieceColor::Both as usize] =
            self.units[PieceColor::Light as usize] | self.units[PieceColor::Dark as usize];
    }
}

pub struct State {
    pub side: PieceColor,
    pub enpassant: Sq,
    pub castling: u8,
    pub half_moves: u32,
    pub full_moves: u32,
}

pub enum CastlingType {
    WhiteKingside,
    WhiteQueenside,
    BlackKingside,
    BlackQueenside,
}

impl State {
    pub fn new() -> Self {
        State {
            side: PieceColor::Light,
            enpassant: Sq::NoSq,
            castling: 0,
            half_moves: 0,
            full_moves: 0,
        }
    }

    pub fn change_side(&mut self) {
        if self.side == PieceColor::Light {
            self.side = PieceColor::Dark;
        } else if self.side == PieceColor::Dark {
            self.side = PieceColor::Light;
        }
    }

    pub fn toggle_castling(&mut self, castling_type: usize) {
        let mut castling = self.castling as BB;
        if castling.get(castling_type) {
            castling.pop(castling_type);
        } else {
            castling.set(castling_type);
        }
        self.castling = castling as u8;
    }
}

pub struct Board {
    pub pos: Position,
    pub state: State,
}

impl Board {
    pub fn new() -> Self {
        Board {
            pos: Position::new(),
            state: State::new(),
        }
    }

    pub fn from_fen(fen: &str) -> Self {
        fen::parse(fen)
    }

    pub fn find_piece(&self, sq: usize) -> Option<Piece> {
        for i in 0..12 {
	    if self.pos.piece[i].get(sq) {
		return Piece::from_num(i);
            }
        }
        None
    }

    pub fn display(&self) {
        println!("\n    +---+---+---+---+---+---+---+---+");
        for r in 0..8 {
            print!("  {} |", 8 - r);
            for f in 0..8 {
                let piece = self.find_piece(SQ!(r, f));
                let (color, _) = Piece::to_tuple(piece);
                let mut piece_char = Piece::to_char(piece);
                if color == (PieceColor::Dark as usize) {
                    piece_char = piece_char.to_ascii_lowercase();
                }
                print!(" {} |", piece_char);
            }
            println!("\n    +---+---+---+---+---+---+---+---+");
        }
        println!("      a   b   c   d   e   f   g   h\n");
        println!(
            "\n      Side to move: {}",
            if self.state.side == PieceColor::Light {
                "white"
            } else {
                "black"
            }
        );
        self.print_castling();
        println!(
            "         Enpassant: {}",
	    Sq::to_str(self.state.enpassant as usize)
        );
        println!("        Full Moves: {}", self.state.full_moves);
    }

    pub fn print_castling(&self) {
        print!("          Castling: ");
        if self.state.castling == 0 {
            println!("none");
            return;
        }
        let mut castling_ltrs = ['-', '-', '-', '-'];
        let castling = self.state.castling as BB;
        if castling.get(CastlingType::WhiteKingside as usize) {
            castling_ltrs[CastlingType::WhiteKingside as usize] = 'K';
        }
        if castling.get(CastlingType::WhiteQueenside as usize) {
            castling_ltrs[CastlingType::WhiteQueenside as usize] = 'Q';
        }
        if castling.get(CastlingType::BlackKingside as usize) {
            castling_ltrs[CastlingType::BlackKingside as usize] = 'k';
        }
        if castling.get(CastlingType::BlackQueenside as usize) {
            castling_ltrs[CastlingType::BlackQueenside as usize] = 'q';
        }
        println!("{}", castling_ltrs.iter().collect::<String>());
    }
}
