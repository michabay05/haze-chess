use crate::attack::AttackInfo;
use crate::bb::{BBUtil, BB};
use crate::consts::{Piece, PieceColor, Sq};
use crate::{fen, zobrist};
use crate::zobrist::{ZobristAction, ZobristInfo};
use crate::SQ;

#[derive(Clone)]
pub struct Position {
    pub bitboards: [BB; 12],
    pub mailbox: [Option<Piece>; 64],
}

impl Position {
    pub fn new() -> Self {
        Position {
            bitboards: [0; 12],
            mailbox: [None; 64],
        }
    }

    pub fn reset(&mut self) {
        self.bitboards.fill(0);
        self.mailbox.fill(None);
    }

    pub fn units(&self, color: PieceColor) -> BB {
        match color {
            PieceColor::Light => {
                self.bitboards[Piece::LP as usize]
                    | self.bitboards[Piece::LN as usize]
                    | self.bitboards[Piece::LB as usize]
                    | self.bitboards[Piece::LR as usize]
                    | self.bitboards[Piece::LQ as usize]
                    | self.bitboards[Piece::LK as usize]
            }
            PieceColor::Dark => {
                self.bitboards[Piece::DP as usize]
                    | self.bitboards[Piece::DN as usize]
                    | self.bitboards[Piece::DB as usize]
                    | self.bitboards[Piece::DR as usize]
                    | self.bitboards[Piece::DQ as usize]
                    | self.bitboards[Piece::DK as usize]
            }
            PieceColor::Both => self.units(PieceColor::Light) | self.units(PieceColor::Dark),
        }
    }
}

#[derive(Clone)]
pub struct State {
    pub side: PieceColor,
    pub xside: PieceColor,
    pub enpassant: Option<Sq>,
    pub castling: u8,
    pub half_moves: u32,
    pub full_moves: u32,
    // ========= Zobrist keys
    // The 'key' is the primary zobrist hashing key, while the 'lock'
    // serves as the secondary hashing key. This is done to prevent the
    // chance of collision(two positions with the same key). Even though,
    // the chances of two positions having the same key is small, it may
    // happen. Therefore, this chance can be significantly reduced by adding
    // a second key to every position
    pub key: u64,
    pub lock: u64,
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
            xside: PieceColor::Dark,
            enpassant: None,
            castling: 0,
            half_moves: 0,
            full_moves: 0,
            key: 0,
            lock: 0,
        }
    }

    pub fn reset(&mut self) {
        self.side = PieceColor::Light;
        self.xside = PieceColor::Dark;
        self.enpassant = None;
        self.castling = 0;
        self.half_moves = 0;
        self.full_moves = 0;
        self.key = 0;
        self.lock = 0;
    }

    pub fn change_side(&mut self) {
        if self.side == PieceColor::Light {
            self.side = PieceColor::Dark;
            self.xside = PieceColor::Light;
        } else {
            self.side = PieceColor::Light;
            self.xside = PieceColor::Dark;
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

#[derive(Clone)]
pub struct Board {
    pub pos: Position,
    pub state: State,
    pub zobrist_info: ZobristInfo,
}

impl Board {
    pub fn new() -> Self {
        let mut this = Board {
            pos: Position::new(),
            state: State::new(),
            zobrist_info: ZobristInfo::new(),
        };
        this.zobrist_info.init();
        this
    }

    pub fn set_fen(&mut self, fen: &str) {
        self.pos.reset();
        self.state.reset();
        fen::parse(fen, self);
    }

    pub fn add_piece(self: &mut Self, piece: Option<Piece>, square: Option<Sq>) {
        if let Some(p) = piece {
            if let Some(sq) = square {
                self.pos.bitboards[p as usize].set(sq as usize);
                self.pos.mailbox[sq as usize] = piece;
                zobrist::update(ZobristAction::TogglePiece(p, sq), self);
            }
        }
    }

    pub fn remove_piece(self: &mut Self, square: Option<Sq>) {
        if let Some(sq) = square {
            if let Some(p) = self.pos.mailbox[sq as usize].take() {
                self.pos.bitboards[p as usize].set(sq as usize);
                zobrist::update(ZobristAction::TogglePiece(p, sq), self);
            }
        }
    }

    pub fn display(&self) {
        println!("\n    +---+---+---+---+---+---+---+---+");
        for r in 0..8 {
            print!("  {} |", 8 - r);
            for f in 0..8 {
                let piece = self.pos.mailbox[SQ!(r, f)];
                let piece_char = Piece::to_char(piece);
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
        if let Some(enpass) = self.state.enpassant {
            println!("         Enpassant: {}", enpass);
        } else {
            println!("         Enpassant: none");
        }
        println!("        Full Moves: {}\n", self.state.full_moves);
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

    #[allow(dead_code)]
    pub fn is_in_check(&self, attack_info: &AttackInfo, checker_side: PieceColor) -> bool {
        let king_type = if checker_side == PieceColor::Light {
            Piece::DK
        } else {
            Piece::LK
        } as usize;
        sq_attacked(
            &self.pos,
            attack_info,
            Sq::from_num(self.pos.bitboards[king_type].lsb()),
            checker_side,
        )
    }
}

pub fn sq_attacked(pos: &Position, attack_info: &AttackInfo, sq: Sq, side: PieceColor) -> bool {
    assert!(side != PieceColor::Both);
    let both_units = pos.units(PieceColor::Both);
    if side == PieceColor::Light
        && ((attack_info.pawn[PieceColor::Dark as usize][sq as usize]
            & pos.bitboards[Piece::LP as usize])
            != 0)
    {
        return true;
    }
    if side == PieceColor::Dark
        && ((attack_info.pawn[PieceColor::Light as usize][sq as usize]
            & pos.bitboards[Piece::DP as usize])
            != 0)
    {
        return true;
    }
    if (attack_info.knight[sq as usize] & pos.bitboards[(side as usize) * 6 + 1]) != 0 {
        return true;
    }
    if (attack_info.get_bishop_attack(sq, both_units) & pos.bitboards[(side as usize) * 6 + 2]) != 0 {
        return true;
    }
    if (attack_info.get_rook_attack(sq, both_units) & pos.bitboards[(side as usize) * 6 + 3]) != 0 {
        return true;
    }
    if (attack_info.get_queen_attack(sq, both_units) & pos.bitboards[(side as usize) * 6 + 4]) != 0 {
        return true;
    }
    if (attack_info.king[sq as usize] & pos.bitboards[(side as usize) * 6 + 5]) != 0 {
        return true;
    }
    false
}

/// `side` refers to the attacking side
#[allow(dead_code)]
pub fn print_attacked_sqs(board: &Board, attack_info: &AttackInfo, side: PieceColor) {
    for r in 0..8 {
        for f in 0..8 {
            let sq = SQ!(r, f);
            if f == 0 {
                print!("  {} ", 8 - r);
            }
            print!(
                " {}",
                if sq_attacked(&board.pos, attack_info, Sq::from_num(sq), side) {
                    '1'
                } else {
                    '.'
                }
            );
        }
        println!();
    }
    println!("     - - - - - - - -");
    println!("     a b c d e f g h\n");
}
