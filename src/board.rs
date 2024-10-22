use crate::attack::AttackInfo;
use crate::bb::{BBUtil, BB};
use crate::consts::{Direction, Piece, PieceColor, PieceType, Sq};
use crate::moves::{Move, MoveFlag, MoveUtil};
use crate::zobrist::{ZobristAction, ZobristInfo};
use crate::SQ;
use crate::{fen, zobrist};

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

// TODO: remove this struct, it's unnecessary
#[derive(Clone)]
pub struct State {
    pub side: PieceColor,
    // pub castling: u8,
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
            key: 0,
            lock: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = State::new();
    }

    pub fn change_side(&mut self) {
        if self.side == PieceColor::Light {
            self.side = PieceColor::Dark;
        } else {
            self.side = PieceColor::Light;
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Undo {
    // Bitboard of changed pieces
    pub entry: BB,
    // Captured piece
    captured: Option<Piece>,
    pub enpassant: Option<Sq>,
    // 50-move counter
    fifty: usize,
}

impl Undo {
    pub fn from_prev(prev: &Self) -> Self {
        Self {
            entry: prev.entry,
            captured: None,
            enpassant: None,
            fifty: prev.fifty + 1,
        }
    }
}

#[derive(Clone)]
pub struct Board {
    pub pos: Position,
    pub state: State,
    pub zobrist: ZobristInfo,
    pub history: [Undo; 2048],
    pub game_ply: usize,
}

impl Board {
    pub fn new() -> Self {
        let mut this = Board {
            pos: Position::new(),
            state: State::new(),
            zobrist: ZobristInfo::new(),
            history: [Undo::default(); 2048],
            game_ply: 0,
        };
        this.zobrist.init();
        this
    }

    pub fn enpassant(&self) -> Option<Sq> {
        self.history[self.game_ply].enpassant
    }

    pub fn set_enpassant(&mut self, sq: Option<Sq>) {
        self.history[self.game_ply].enpassant = sq;
    }

    pub fn set_fen(&mut self, fen: &str) {
        self.pos.reset();
        self.state.reset();
        self.history.fill(Undo::default());
        fen::parse(fen, self);
    }

    #[inline(always)]
    pub fn diagonal_sliders(&self, side: PieceColor) -> BB {
        if side == PieceColor::Light {
            self.pos.bitboards[Piece::LB as usize] | self.pos.bitboards[Piece::LQ as usize]
        } else {
            self.pos.bitboards[Piece::DB as usize] | self.pos.bitboards[Piece::DQ as usize]
        }
    }

    #[inline(always)]
    pub fn orthogonal_sliders(&self, side: PieceColor) -> BB {
        if side == PieceColor::Light {
            self.pos.bitboards[Piece::LR as usize] | self.pos.bitboards[Piece::LQ as usize]
        } else {
            self.pos.bitboards[Piece::DR as usize] | self.pos.bitboards[Piece::DQ as usize]
        }
    }

    #[inline(always)]
    pub fn add_piece(&mut self, piece: Option<Piece>, square: Option<Sq>) {
        if let Some(p) = piece {
            if let Some(sq) = square {
                // TODO: evaluator.add_piece()
                self.pos.mailbox[sq as usize] = Some(p);
                self.pos.bitboards[p as usize].set(sq as usize);
                zobrist::update(ZobristAction::TogglePiece(p, sq), self);
            }
        }
    }

    #[inline(always)]
    pub fn remove_piece(&mut self, square: Option<Sq>) {
        if let Some(sq) = square {
            if let Some(p) = self.pos.mailbox[sq as usize].take() {
                // TODO: evaluator.remove_piece()
                self.pos.bitboards[p as usize].pop(sq as usize);
                zobrist::update(ZobristAction::TogglePiece(p, sq), self);
            }
        }
    }

    #[inline(always)]
    pub fn move_piece(&mut self, source: Sq, target: Sq) {
        // If there's a piece, free up the target square
        self.remove_piece(Some(target));
        self.move_piece_quiet(source, target);
    }

    #[inline(always)]
    pub fn move_piece_quiet(&mut self, source: Sq, target: Sq) {
        if let Some(piece) = self.pos.mailbox[source as usize].take() {
            // TODO: evaluator.move_piece_quiet()
            // Update hash to account piece's movement
            zobrist::update(ZobristAction::TogglePiece(piece, source), self);
            zobrist::update(ZobristAction::TogglePiece(piece, target), self);
            // Remove the piece from the source square and place it on the target
            self.pos.bitboards[piece as usize].pop(source as usize);
            self.pos.bitboards[piece as usize].set(target as usize);
            // Update the mailbox with the piece's new position
            self.pos.mailbox[target as usize] = Some(piece);
        }
    }

    pub fn play_move(&mut self, side: PieceColor, mv: Move) {
        // Change side to move
        self.state.change_side();
        zobrist::update(ZobristAction::ChangeColor, self);

        // Increment the half move clock
        self.game_ply += 1;
        let mut current = Undo::from_prev(&self.history[self.game_ply - 1]);

        let source = mv.source();
        let target = mv.target();
        let flag = mv.flag();
        current.entry.set(source as usize);
        current.entry.set(target as usize);

        if let Some(p) = self.pos.mailbox[mv.source() as usize] {
            if p.piece_type() == PieceType::Pawn || mv.is_capture() {
                current.fifty = 0;
            }
        }

        match flag {
            MoveFlag::Quiet => {
                self.move_piece_quiet(source, target);
            }
            MoveFlag::DoublePush => {
                self.move_piece_quiet(source, target);
                let enpass = source.add(Direction::North.relative(side));
                current.enpassant = Some(enpass);
                zobrist::update(ZobristAction::SetEnpassant(enpass), self);
            }
            MoveFlag::KingSideCastling => {
                // Move king and rook to the kingside
                if side == PieceColor::Light {
                    self.move_piece_quiet(Sq::E1, Sq::G1);
                    self.move_piece_quiet(Sq::H1, Sq::F1);
                } else {
                    self.move_piece_quiet(Sq::E8, Sq::G8);
                    self.move_piece_quiet(Sq::H8, Sq::F8);
                }
            }
            MoveFlag::QueenSideCastling => {
                // Move king and rook to the queenside
                if side == PieceColor::Light {
                    self.move_piece_quiet(Sq::E1, Sq::C1);
                    self.move_piece_quiet(Sq::A1, Sq::D1);
                } else {
                    self.move_piece_quiet(Sq::E8, Sq::C8);
                    self.move_piece_quiet(Sq::A8, Sq::D8);
                }
            }
            MoveFlag::Enpassant => {
                self.move_piece_quiet(source, target);
                let sq = target.add(Direction::South.relative(side));
                self.remove_piece(Some(sq));
            }
            MoveFlag::PromKnight => {
                self.remove_piece(Some(source));
                self.add_piece(
                    Some(Piece::from_type(side, PieceType::Knight)),
                    Some(target),
                );
            }
            MoveFlag::PromBishop => {
                self.remove_piece(Some(source));
                self.add_piece(
                    Some(Piece::from_type(side, PieceType::Bishop)),
                    Some(target),
                );
            }
            MoveFlag::PromRook => {
                self.remove_piece(Some(source));
                self.add_piece(Some(Piece::from_type(side, PieceType::Rook)), Some(target));
            }
            MoveFlag::PromQueen => {
                self.remove_piece(Some(source));
                self.add_piece(Some(Piece::from_type(side, PieceType::Queen)), Some(target));
            }
            MoveFlag::PromCapKnight => {
                self.remove_piece(Some(source));
                current.captured = self.pos.mailbox[target as usize];
                self.remove_piece(Some(target));
                self.add_piece(
                    Some(Piece::from_type(side, PieceType::Knight)),
                    Some(target),
                );
            }
            MoveFlag::PromCapBishop => {
                self.remove_piece(Some(source));
                current.captured = self.pos.mailbox[target as usize];
                self.remove_piece(Some(target));
                self.add_piece(
                    Some(Piece::from_type(side, PieceType::Bishop)),
                    Some(target),
                );
            }
            MoveFlag::PromCapRook => {
                self.remove_piece(Some(source));
                current.captured = self.pos.mailbox[target as usize];
                self.remove_piece(Some(target));
                self.add_piece(Some(Piece::from_type(side, PieceType::Rook)), Some(target));
            }
            MoveFlag::PromCapQueen => {
                self.remove_piece(Some(source));
                current.captured = self.pos.mailbox[target as usize];
                self.remove_piece(Some(target));
                self.add_piece(Some(Piece::from_type(side, PieceType::Queen)), Some(target));
            }
            MoveFlag::Capture => {
                current.captured = self.pos.mailbox[target as usize];
                self.move_piece(source, target);
            }
        }

        self.history[self.game_ply] = current;
    }

    pub fn undo_move(&mut self, side: PieceColor, mv: Move) {
        let source = mv.source();
        let target = mv.target();
        let flag = mv.flag();
        let opp = side.opposite();

        match flag {
            MoveFlag::Quiet => {
                self.move_piece_quiet(target, source);
            }
            MoveFlag::DoublePush => {
                self.move_piece_quiet(target, source);
                // TODO: replace this with a zobrist action
                if let Some(enpass) = self.history[self.game_ply].enpassant {
                    zobrist::update(ZobristAction::SetEnpassant(enpass), self);
                    // self.state.key ^= self.zobrist.key.enpassant[enpass as usize];
                    // self.state.lock ^= self.zobrist.lock.enpassant[enpass as usize];
                }
            }
            MoveFlag::KingSideCastling => {
                // Move king and rook to the kingside
                if side == PieceColor::Light {
                    self.move_piece_quiet(Sq::G1, Sq::E1);
                    self.move_piece_quiet(Sq::F1, Sq::H1);
                } else {
                    self.move_piece_quiet(Sq::G8, Sq::E8);
                    self.move_piece_quiet(Sq::F8, Sq::H8);
                }
            }
            MoveFlag::QueenSideCastling => {
                // Move king and rook to the kingside
                if side == PieceColor::Light {
                    self.move_piece_quiet(Sq::C1, Sq::E1);
                    self.move_piece_quiet(Sq::D1, Sq::A1);
                } else {
                    self.move_piece_quiet(Sq::C8, Sq::E8);
                    self.move_piece_quiet(Sq::D8, Sq::A8);
                }
            }
            MoveFlag::Enpassant => {
                self.move_piece_quiet(target, source);
                let sq = target.add(Direction::South.relative(side));
                self.add_piece(Some(Piece::from_type(opp, PieceType::Pawn)), Some(sq));
            }
            MoveFlag::PromKnight
            | MoveFlag::PromBishop
            | MoveFlag::PromRook
            | MoveFlag::PromQueen => {
                self.remove_piece(Some(target));
                self.add_piece(Some(Piece::from_type(side, PieceType::Pawn)), Some(source));
            }
            MoveFlag::PromCapKnight
            | MoveFlag::PromCapBishop
            | MoveFlag::PromCapRook
            | MoveFlag::PromCapQueen => {
                self.remove_piece(Some(target));
                self.add_piece(Some(Piece::from_type(side, PieceType::Pawn)), Some(source));
                self.add_piece(self.history[self.game_ply].captured, Some(target));
            }
            MoveFlag::Capture => {
                self.move_piece(target, source);
                self.add_piece(self.history[self.game_ply].captured, Some(target));
            }
        }

        self.state.change_side();
        zobrist::update(ZobristAction::ChangeColor, self);
        self.game_ply -= 1;
    }

    pub fn display(&self) {
        println!("\n    +---+---+---+---+---+---+---+---+");
        for r in (0..8).rev() {
            // for r in 0..8 {
            print!("  {} |", r + 1);
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
        if let Some(enpass) = self.enpassant() {
            println!("         Enpassant: {}", enpass);
        } else {
            println!("         Enpassant: none");
        }
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
        && ((attack_info.get_pawn_attack(PieceColor::Dark, sq) & pos.bitboards[Piece::LP as usize])
            != 0)
    {
        return true;
    }
    if side == PieceColor::Dark
        && ((attack_info.get_pawn_attack(PieceColor::Light, sq)
            & pos.bitboards[Piece::DP as usize])
            != 0)
    {
        return true;
    }
    if (attack_info.get_knight_attack(sq) & pos.bitboards[(side as usize) * 6 + 1]) != 0 {
        return true;
    }
    if (attack_info.get_bishop_attack(sq, both_units) & pos.bitboards[(side as usize) * 6 + 2]) != 0
    {
        return true;
    }
    if (attack_info.get_rook_attack(sq, both_units) & pos.bitboards[(side as usize) * 6 + 3]) != 0 {
        return true;
    }
    if (attack_info.get_queen_attack(sq, both_units) & pos.bitboards[(side as usize) * 6 + 4]) != 0
    {
        return true;
    }
    if (attack_info.get_king_attack(sq) & pos.bitboards[(side as usize) * 6 + 5]) != 0 {
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

#[cfg(test)]
mod tests {
    use super::fen;
    use super::{AttackInfo, BB, BBUtil, Board, Move, MoveUtil, MoveFlag, Piece, PieceColor, Sq};

    #[test]
    fn board_manipulation() {
        let mut attack_info = AttackInfo::new();
        attack_info.init();
        let mut b = Board::new();

        assert_eq!(b.game_ply, 0);
        assert_eq!(b.state.side, PieceColor::Light);
        assert_eq!(b.state.key, 0);
        assert_eq!(b.state.lock, 0);
        assert_eq!(b.pos.bitboards.iter().any(|x| *x != 0), false);
        assert_eq!(b.pos.mailbox.iter().any(|x| x.is_some()), false);

        b.add_piece(Some(Piece::LN), Some(Sq::F3));
        assert_eq!(b.pos.mailbox[Sq::F3 as usize], Some(Piece::LN));
        assert_eq!(b.pos.bitboards[Piece::LN as usize], BB::from_sq(Sq::F3));

        b.remove_piece(Some(Sq::F3));
        assert_eq!(b.pos.mailbox[Sq::F3 as usize], None);
        assert_eq!(b.pos.bitboards[Piece::LN as usize], 0);

        // white is in check
        b.set_fen("rnb1kbnr/pppp1ppp/8/4p3/4PP1q/8/PPPP2PP/RNBQKBNR w KQkq");
        assert_eq!(b.is_in_check(&attack_info, PieceColor::Light), false);
        assert_eq!(b.is_in_check(&attack_info, PieceColor::Dark), true);

        // blocked
        b.set_fen("rnb1kbnr/pppp1ppp/8/4p3/4PP1q/6P1/PPPP3P/RNBQKBNR b KQkq");
        assert_eq!(b.is_in_check(&attack_info, PieceColor::Light), false);
        assert_eq!(b.is_in_check(&attack_info, PieceColor::Dark), false);

        // Re-set board from the starting position
        b.set_fen(fen::FEN_POSITIONS[1]);
        let mut mv = Move::encode(Sq::D2, Sq::D4, MoveFlag::Quiet);
        assert_eq!(b.state.side, PieceColor::Light);
        b.play_move(PieceColor::Light, mv);
        assert_eq!(b.pos.mailbox[Sq::D2 as usize], None);
        assert_eq!(b.pos.mailbox[Sq::D4 as usize], Some(Piece::LP));
        assert_eq!(b.pos.bitboards[Piece::LP as usize].get(Sq::D2 as usize), false);
        assert_eq!(b.pos.bitboards[Piece::LP as usize].get(Sq::D4 as usize), true);
        assert_eq!(b.state.side, PieceColor::Dark);

        b.undo_move(PieceColor::Light, mv);
        assert_eq!(b.pos.mailbox[Sq::D2 as usize], Some(Piece::LP));
        assert_eq!(b.pos.mailbox[Sq::D4 as usize], None);
        assert_eq!(b.pos.bitboards[Piece::LP as usize].get(Sq::D2 as usize), true);
        assert_eq!(b.pos.bitboards[Piece::LP as usize].get(Sq::D4 as usize), false);
        assert_eq!(b.state.side, PieceColor::Light);

        // Setup a different position
        b.set_fen(fen::FEN_POSITIONS[2]);
        mv = Move::encode(Sq::E2, Sq::A6, MoveFlag::Capture);
        b.play_move(PieceColor::Light, mv);
        mv = Move::encode(Sq::B4, Sq::C3, MoveFlag::Capture);
        b.play_move(PieceColor::Dark, mv);
        mv = Move::encode(Sq::D2, Sq::C3, MoveFlag::Capture);
        b.play_move(PieceColor::Light, mv);
        mv = Move::encode(Sq::B6, Sq::C8, MoveFlag::Quiet);
        b.play_move(PieceColor::Dark, mv);

        assert_eq!(b.pos.bitboards[Piece::LB as usize], BB::from_sq(Sq::A6) | BB::from_sq(Sq::C3));
        assert_eq!(b.pos.bitboards[Piece::LN as usize], BB::from_sq(Sq::E5));
        assert_eq!(b.pos.bitboards[Piece::DB as usize], BB::from_sq(Sq::G7));
        assert_eq!(b.pos.bitboards[Piece::DN as usize], BB::from_sq(Sq::C8) | BB::from_sq(Sq::F6));
        assert_eq!(b.pos.bitboards[Piece::DP as usize].get(Sq::C3 as usize), false);
        assert_eq!(b.game_ply, 4);

        mv = Move::encode(Sq::B6, Sq::C8, MoveFlag::Quiet);
        b.undo_move(PieceColor::Dark, mv);
        mv = Move::encode(Sq::D2, Sq::C3, MoveFlag::Capture);
        b.undo_move(PieceColor::Light, mv);
        assert_eq!(b.pos.bitboards[Piece::LB as usize], BB::from_sq(Sq::A6) | BB::from_sq(Sq::D2));
        assert_eq!(b.pos.bitboards[Piece::LN as usize], BB::from_sq(Sq::E5));
        assert_eq!(b.pos.bitboards[Piece::DB as usize], BB::from_sq(Sq::G7));
        assert_eq!(b.pos.bitboards[Piece::DN as usize], BB::from_sq(Sq::B6) | BB::from_sq(Sq::F6));
        assert_eq!(b.pos.bitboards[Piece::DP as usize].get(Sq::C3 as usize), true);
        assert_eq!(b.game_ply, 2);
    }
}
