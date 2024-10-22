use crate::attack::AttackInfo;
use crate::bb::{BBUtil, BB};
use crate::board::{self, Board, CastlingType};
use crate::consts::{Direction, Piece, PieceColor, Sq};
use crate::moves::{Move, MoveFlag, MoveUtil};

// TODO: this struct is not useful.
// Make `MoveList` an alias for a Vector of moves
pub struct MoveList {
    pub moves: Vec<Move>,
}

impl MoveList {
    pub fn new() -> Self {
        let a = 10 + 2;
        Self { moves: vec![] }
    }

    pub fn print(&self) {
        println!("    Source   |   Target  |     Flag");
        println!("  --------------------------------------------");
        for el in &self.moves {
            println!("       {}    |     {}    |     {:?}",
                el.source(),
                el.target(),
                el.flag()
            );
        }
        println!("\n    Total number of moves: {}", self.moves.len());
    }

    pub fn search(&mut self, source: Sq, target: Sq, promoted: Option<Piece>) -> Option<Move> {
        let (flag_a, flag_b) = if let Some(p) = promoted {
            match p {
                Piece::LN | Piece::DN => (MoveFlag::PromKnight, MoveFlag::PromCapKnight),
                Piece::LB | Piece::DB => (MoveFlag::PromBishop, MoveFlag::PromCapBishop),
                Piece::LR | Piece::DR => (MoveFlag::PromRook, MoveFlag::PromCapRook),
                Piece::LQ | Piece::DQ => (MoveFlag::PromQueen, MoveFlag::PromCapQueen),
                Piece::LP | Piece::DP | Piece::LK | Piece::DK => (MoveFlag::Quiet, MoveFlag::Quiet),
            }
        } else {
            // Not sure if making the fallback 'quiet' is good
            (MoveFlag::Quiet, MoveFlag::Quiet)
        };
        self.moves
            .iter()
            .find(|mv| {
                mv.source() == source
                    && mv.target() == target
                    && (mv.flag() == flag_a || mv.flag() == flag_b)
            })
            .copied()
    }
}

pub fn generate(board: &Board, attack_info: &AttackInfo, ml: &mut MoveList) {
    generate_pawns(board, attack_info, ml);
    generate_knights(board, attack_info, ml);
    generate_bishops(board, attack_info, ml);
    generate_rooks(board, attack_info, ml);
    generate_queens(board, attack_info, ml);
    generate_kings(board, attack_info, ml);
}

fn generate_pawns(board: &Board, attack_info: &AttackInfo, ml: &mut MoveList) {
    const PROMOTED_FLAG: [[MoveFlag; 4]; 2] = {
        use MoveFlag::*;
        [
            [PromQueen, PromRook, PromBishop, PromKnight],
            [PromCapQueen, PromCapRook, PromCapBishop, PromCapKnight],
        ]
    };

    let mut bb_copy: BB;
    let mut attack_copy: BB;
    let promotion_start; // The first square of the promoting rank
    let twosquarepush_start; // The first square of the current side's pawn starting square
    let enemy_rank_start; // The first square of the current side's pawn starting square
    let direction: Direction;
    let piece: Piece;
    let enemy_color: PieceColor;
    let is_white = board.state.side == PieceColor::Light;
    if is_white {
        piece = Piece::LP;
        enemy_color = PieceColor::Dark;
        promotion_start = Sq::A7;
        twosquarepush_start = Sq::A2;
        enemy_rank_start = Sq::A8;
        direction = Direction::South;
    } else {
        piece = Piece::DP;
        enemy_color = PieceColor::Light;
        promotion_start = Sq::A2;
        twosquarepush_start = Sq::A7;
        enemy_rank_start = Sq::H1;
        direction = Direction::North;
    }

    bb_copy = board.pos.bitboards[piece as usize];
    let mut source: i32;
    let mut target: i32;

    while bb_copy > 0 {
        source = bb_copy.pop_lsb() as i32;
        target = source + direction as i32;
        let are_pawns_in_bound = if is_white {
            target >= enemy_rank_start as i32
        } else {
            target <= enemy_rank_start as i32
        };
        if are_pawns_in_bound && !board.pos.units(PieceColor::Both).get(target as usize) {
            // If true, this move is a promotion
            if (source >= promotion_start as i32) && (source <= (promotion_start as i32 + 7)) {
                for i in 0..4 {
                    ml.moves.push(Move::encode(
                        Sq::from_num(source as usize),
                        Sq::from_num(target as usize),
                        PROMOTED_FLAG[0][i],
                    ));
                }
            } else {
                // If false, this is a normal(quiet or non-capture) move
                ml.moves.push(Move::encode(
                    Sq::from_num(source as usize),
                    Sq::from_num(target as usize),
                    MoveFlag::Quiet,
                ));
                let source_is_in_bound = (source >= twosquarepush_start as i32)
                    && (source <= (twosquarepush_start as i32 + 7));
                if source_is_in_bound
                    && !board
                        .pos
                        .units(PieceColor::Both)
                        .get((target + (direction as i32)) as usize)
                {
                    ml.moves.push(Move::encode(
                        Sq::from_num(source as usize),
                        Sq::from_num((target + (direction as i32)) as usize),
                        MoveFlag::DoublePush,
                    ));
                }
            }
        }

        attack_copy = attack_info.pawn[board.state.side as usize][source as usize]
            & board.pos.units(enemy_color);
        while attack_copy > 0 {
            target = attack_copy.pop_lsb() as i32;
            if (source >= promotion_start as i32) && (source <= (promotion_start as i32 + 7)) {
                // If true, this move is a capture promotion
                for i in 0..4 {
                    ml.moves.push(Move::encode(
                        Sq::from_num(source as usize),
                        Sq::from_num(target as usize),
                        PROMOTED_FLAG[1][i],
                    ));
                }
            } else {
                // If false, this move is a normal capture
                ml.moves.push(Move::encode(
                    Sq::from_num(source as usize),
                    Sq::from_num(target as usize),
                    MoveFlag::Capture,
                ));
            }
        }
        if let Some(sq) = board.enpassant() {
            let enpassant_capture =
                attack_info.pawn[board.state.side as usize][source as usize] & (1 << (sq as usize));
            if enpassant_capture != 0 {
                let enpassant_target = enpassant_capture.lsb();
                ml.moves.push(Move::encode(
                    Sq::from_num(source as usize),
                    Sq::from_num(enpassant_target),
                    MoveFlag::Enpassant,
                ));
            }
        }
    }
}

fn generate_knights(board: &Board, attack_info: &AttackInfo, ml: &mut MoveList) {
    let mut source;
    let mut target;
    let is_white = board.state.side == PieceColor::Light;
    let piece = if is_white { Piece::LN } else { Piece::DN };
    let color = if is_white {
        PieceColor::Light
    } else {
        PieceColor::Dark
    };
    let enemy_color = if is_white {
        PieceColor::Dark
    } else {
        PieceColor::Light
    };
    let mut bb_copy = board.pos.bitboards[piece as usize];
    let mut attack_copy;

    while bb_copy > 0 {
        source = bb_copy.pop_lsb();
        attack_copy = attack_info.knight[source] & (!board.pos.units(color));
        while attack_copy > 0 {
            target = attack_copy.pop_lsb();
            let is_capture_move = board.pos.units(enemy_color).get(target);
            let flag = if is_capture_move {
                MoveFlag::Capture
            } else {
                MoveFlag::Quiet
            };
            ml.moves.push(Move::encode(
                Sq::from_num(source),
                Sq::from_num(target),
                flag,
            ));
        }
    }
}

fn generate_bishops(board: &Board, attack_info: &AttackInfo, ml: &mut MoveList) {
    let mut source;
    let mut target;
    let is_white = board.state.side == PieceColor::Light;
    let piece = if is_white { Piece::LB } else { Piece::DB };
    let color = if is_white {
        PieceColor::Light
    } else {
        PieceColor::Dark
    };
    let enemy_color = color.opposite();
    let mut bb_copy = board.pos.bitboards[piece as usize];
    let mut attack_copy;

    while bb_copy > 0 {
        source = bb_copy.pop_lsb();
        attack_copy = attack_info
            .get_bishop_attack(Sq::from_num(source), board.pos.units(PieceColor::Both))
            & (!board.pos.units(color));
        while attack_copy > 0 {
            target = attack_copy.pop_lsb();
            let is_capture_move = board.pos.units(enemy_color).get(target);
            let flag = if is_capture_move {
                MoveFlag::Capture
            } else {
                MoveFlag::Quiet
            };
            ml.moves.push(Move::encode(
                Sq::from_num(source),
                Sq::from_num(target),
                flag
            ));
        }
    }
}

fn generate_rooks(board: &Board, attack_info: &AttackInfo, ml: &mut MoveList) {
    let mut source;
    let mut target;
    let is_white = board.state.side == PieceColor::Light;
    let piece = if is_white { Piece::LR } else { Piece::DR };
    let color = if is_white {
        PieceColor::Light
    } else {
        PieceColor::Dark
    };
    let enemy_color = color.opposite();
    let mut bb_copy = board.pos.bitboards[piece as usize];
    let mut attack_copy;

    while bb_copy > 0 {
        source = bb_copy.pop_lsb();
        attack_copy = attack_info
            .get_rook_attack(Sq::from_num(source), board.pos.units(PieceColor::Both))
            & (!board.pos.units(color));
        while attack_copy > 0 {
            target = attack_copy.pop_lsb();
            let is_capture_move = board.pos.units(enemy_color).get(target);
            let flag = if is_capture_move {
                MoveFlag::Capture
            } else {
                MoveFlag::Quiet
            };
            ml.moves.push(Move::encode(
                Sq::from_num(source),
                Sq::from_num(target),
                flag
            ));
        }
    }
}

fn generate_queens(board: &Board, attack_info: &AttackInfo, ml: &mut MoveList) {
    let mut source;
    let mut target;
    let is_white = board.state.side == PieceColor::Light;
    let piece = if is_white { Piece::LQ } else { Piece::DQ };
    let color = if is_white {
        PieceColor::Light
    } else {
        PieceColor::Dark
    };
    let enemy_color = color.opposite();
    let mut bb_copy = board.pos.bitboards[piece as usize];
    let mut attack_copy;

    while bb_copy > 0 {
        source = bb_copy.pop_lsb();
        attack_copy = attack_info
            .get_queen_attack(Sq::from_num(source), board.pos.units(PieceColor::Both))
            & (!board.pos.units(color));
        while attack_copy > 0 {
            target = attack_copy.pop_lsb();
            let is_capture_move = board.pos.units(enemy_color).get(target);
            let flag = if is_capture_move {
                MoveFlag::Capture
            } else {
                MoveFlag::Quiet
            };
            ml.moves.push(Move::encode(
                Sq::from_num(source),
                Sq::from_num(target),
                flag
            ));
        }
    }
}

fn generate_kings(board: &Board, attack_info: &AttackInfo, ml: &mut MoveList) {
    let mut source;
    let mut target;
    let is_white = board.state.side == PieceColor::Light;
    let piece = if is_white { Piece::LK } else { Piece::DK };
    let color = if is_white {
        PieceColor::Light
    } else {
        PieceColor::Dark
    };
    let enemy_color = color.opposite();
    let mut bb_copy = board.pos.bitboards[piece as usize];
    let mut attack_copy;

    while bb_copy > 0 {
        source = bb_copy.pop_lsb();
        attack_copy = attack_info.king[source] & (!board.pos.units(color));
        while attack_copy > 0 {
            target = attack_copy.pop_lsb();
            let is_capture_move = board.pos.units(enemy_color).get(target);
            let flag = if is_capture_move {
                MoveFlag::Capture
            } else {
                MoveFlag::Quiet
            };
            ml.moves.push(Move::encode(
                Sq::from_num(source),
                Sq::from_num(target),
                flag
            ));
        }
    }
    if is_white {
        gen_light_castling(board, attack_info, ml);
    } else {
        gen_dark_castling(board, attack_info, ml);
    }
}

fn gen_light_castling(board: &Board, attack_info: &AttackInfo, ml: &mut MoveList) {
    let castling = board.state.castling as BB;
    let both_units = board.pos.units(PieceColor::Both);
    if castling.get(CastlingType::WhiteKingside as usize) {
        if !both_units.get(Sq::F1 as usize) && !both_units.get(Sq::G1 as usize) {
            if !board::sq_attacked(&board.pos, attack_info, Sq::E1, PieceColor::Dark)
                && !board::sq_attacked(&board.pos, attack_info, Sq::F1, PieceColor::Dark)
            {
                if let Some(mv) = Move::from_str("e1g1", MoveFlag::KingSideCastling) {
                    ml.moves.push(mv);
                }
            }
        }
    }

    if castling.get(CastlingType::WhiteQueenside as usize) {
        if !both_units.get(Sq::B1 as usize)
            && !both_units.get(Sq::C1 as usize)
            && !both_units.get(Sq::D1 as usize)
        {
            if !board::sq_attacked(&board.pos, attack_info, Sq::D1, PieceColor::Dark)
                && !board::sq_attacked(&board.pos, attack_info, Sq::E1, PieceColor::Dark)
            {
                if let Some(mv) = Move::from_str("e1c1", MoveFlag::QueenSideCastling) {
                    ml.moves.push(mv);
                }
            }
        }
    }
}

fn gen_dark_castling(board: &Board, attack_info: &AttackInfo, ml: &mut MoveList) {
    let castling = board.state.castling as BB;
    let both_units = board.pos.units(PieceColor::Both);
    if castling.get(CastlingType::BlackKingside as usize) {
        if !both_units.get(Sq::F8 as usize) && !both_units.get(Sq::G8 as usize) {
            if !board::sq_attacked(&board.pos, attack_info, Sq::E8, PieceColor::Light)
                && !board::sq_attacked(&board.pos, attack_info, Sq::F8, PieceColor::Light)
            {
                if let Some(mv) = Move::from_str("e8g8", MoveFlag::KingSideCastling) {
                    ml.moves.push(mv);
                }
            }
        }
    }

    if castling.get(CastlingType::BlackQueenside as usize) {
        if !both_units.get(Sq::B8 as usize)
            && !both_units.get(Sq::C8 as usize)
            && !both_units.get(Sq::D8 as usize)
        {
            if !board::sq_attacked(&board.pos, attack_info, Sq::D8, PieceColor::Light)
                && !board::sq_attacked(&board.pos, attack_info, Sq::E8, PieceColor::Light)
            {
                if let Some(mv) = Move::from_str("e8c8", MoveFlag::QueenSideCastling) {
                    ml.moves.push(mv);
                }
            }
        }
    }
}
