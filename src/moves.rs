use crate::attack::AttackInfo;
use crate::bb::BBUtil;
use crate::board::{self, Board};
use crate::consts::{Direction, Piece, PieceColor, Sq};

pub type Move = u32;

pub trait MoveUtil {
    fn encode(
        source: Sq,
        target: Sq,
        piece: Piece,
        promoted: Option<Piece>,
        capture: bool,
        twosquare: bool,
        enpassant: bool,
        castling: bool,
    ) -> Self;
    fn source(&self) -> Sq;
    fn target(&self) -> Sq;
    fn piece(&self) -> Piece;
    fn promoted(&self) -> Option<Piece>;
    fn is_capture(&self) -> bool;
    fn is_twosquare(&self) -> bool;
    fn is_enpassant(&self) -> bool;
    fn is_castling(&self) -> bool;
    fn from_str(
        move_str: &str,
        piece: Piece,
        capture: bool,
        twosquare: bool,
        enpassant: bool,
        castling: bool,
    ) -> Self;
    fn to_str(&self) -> String;
}

impl MoveUtil for Move {
    fn encode(
        source: Sq,
        target: Sq,
        piece: Piece,
        promoted: Option<Piece>,
        capture: bool,
        twosquare: bool,
        enpassant: bool,
        castling: bool,
    ) -> Self {
        source as u32
            | ((target as u32) << 6)
            | ((Piece::to_num(Some(piece)) as u32) << 12)
            | ((Piece::to_num(promoted) as u32) << 16)
            | ((capture as u32) << 20)
            | ((twosquare as u32) << 21)
            | ((enpassant as u32) << 22)
            | ((castling as u32) << 23)
    }

    fn source(&self) -> Sq {
        Sq::from_num((*self & 0x3F) as usize)
    }

    fn target(&self) -> Sq {
        Sq::from_num(((*self & 0xFC0) >> 6) as usize)
    }

    fn piece(&self) -> Piece {
        let piece = Piece::from_num(((*self & 0xF000) >> 12) as usize);
        assert!(piece.is_some());
        piece.unwrap()
    }

    fn promoted(&self) -> Option<Piece> {
        Piece::from_num(((*self & 0xF0000) >> 16) as usize)
    }

    fn is_capture(&self) -> bool {
        (*self & 0x100000) > 0
    }

    fn is_twosquare(&self) -> bool {
        (*self & 0x200000) > 0
    }

    fn is_enpassant(&self) -> bool {
        (*self & 0x400000) > 0
    }

    fn is_castling(&self) -> bool {
        (*self & 0x800000) > 0
    }

    fn from_str(
        move_str: &str,
        piece: Piece,
        capture: bool,
        twosquare: bool,
        enpassant: bool,
        castling: bool,
    ) -> Self {
        assert!(move_str.len() == 4 || move_str.len() == 5);
        let source = Sq::from_str(&move_str[0..2]);
        let target = Sq::from_str(&move_str[2..4]);
        let promoted = if move_str.len() == 5 {
            Piece::from_char(move_str.chars().nth(4).unwrap())
        } else {
            None
        };
        Self::encode(
            source, target, piece, promoted, capture, twosquare, enpassant, castling,
        )
    }

    fn to_str(&self) -> String {
        let source_str = Sq::to_string(self.source());
        let target_str = Sq::to_string(self.target());
        let promoted_str = Piece::to_char(self.promoted());
        format!("{}{}{}", source_str, target_str, promoted_str)
    }
}

#[derive(PartialEq)]
pub enum MoveFlag {
    AllMoves,
    CapturesOnly,
}

const CASTLING_RIGHTS: [usize; 64] = [
    7, 15, 15, 15, 3, 15, 15, 11, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 13, 15, 15, 15, 12, 15, 15, 14,
];

pub fn make(main: &mut Board, attack_info: &AttackInfo, mv: Move, move_flag: MoveFlag) -> bool {
    if move_flag == MoveFlag::AllMoves {
        let clone = main.clone();

        // Extract information about the move
        let source = mv.source() as usize;
        let target = mv.target() as usize;
        let piece = Piece::to_num(Some(mv.piece()));
        let promoted = mv.promoted();
        let is_capture = mv.is_capture();
        let is_twosquare = mv.is_twosquare();
        let is_enpassant = mv.is_enpassant();
        let is_castling = mv.is_castling();

        // Move piece from source to target by removing source bit and turning on the target bit
        main.pos.piece[piece].pop(source);
        main.pos.piece[piece].set(target);

        if is_capture {
            let (start, end) = if main.state.side == PieceColor::Light {
                (Piece::DP as usize, Piece::DK as usize)
            } else {
                (Piece::LP as usize, Piece::LK as usize)
            };
            for bb_piece in start..=end {
                if main.pos.piece[bb_piece].get(target) {
                    main.pos.piece[bb_piece].pop(target);
                    break;
                }
            }
        }

        if promoted.is_some() {
            let promoted_num = Piece::to_num(promoted);
            main.pos.piece[piece].pop(target);
            main.pos.piece[promoted_num].set(target);
        }

        if is_enpassant {
            let mut pawn_type;
            let mut direction;
            if main.state.side == PieceColor::Light {
                pawn_type = Piece::DP;
                direction = Direction::NORTH;
            } else {
                pawn_type = Piece::LP;
                direction = Direction::SOUTH;
            }
            main.pos.piece[pawn_type as usize].pop((target as i32 + direction as i32) as usize);
        }

        main.state.enpassant = Sq::NoSq;
        if is_twosquare {
            if main.state.side == PieceColor::Light {
                main.state.enpassant =
                    Sq::from_num((target as i32 + Direction::NORTH as i32) as usize);
            } else {
                main.state.enpassant =
                    Sq::from_num((target as i32 + Direction::SOUTH as i32) as usize);
            }
        }

        if is_castling {
            let mut rook_type;
            let mut source_castling;
            let mut target_castling;
            match Sq::from_num(target) {
		Sq::G1 => {
		    rook_type = Piece::LR;
		    source_castling = Sq::H1;
		    target_castling = Sq::F1;
		},
		Sq::C1 => {
		    rook_type = Piece::LR;
		    source_castling = Sq::A1;
		    target_castling = Sq::D1;
		},
		Sq::G8 => {
		    rook_type = Piece::DR;
		    source_castling = Sq::H8;
		    target_castling = Sq::F8;
		},
		Sq::C8 => {
		    rook_type = Piece::DR;
		    source_castling = Sq::A8;
		    target_castling = Sq::D8;
		},
		_ => unreachable!("Target castling square should only be [ G1, C1 ] for white and [ G8, C8 ] for black"),
	    };
            main.pos.piece[rook_type as usize].pop(source_castling as usize);
            main.pos.piece[rook_type as usize].set(target_castling as usize);
        }
        main.state.castling &= CASTLING_RIGHTS[source] as u8;
        main.state.castling &= CASTLING_RIGHTS[target] as u8;

        main.pos.update_units();
        main.state.change_side();

        let king_type = if main.state.side == PieceColor::Light {
            Piece::DK
        } else {
            Piece::LK
        } as usize;
        if board::sq_attacked(
            &main.pos,
            attack_info,
            Sq::from_num(main.pos.piece[king_type].lsb()),
            main.state.side,
        ) {
            *main = clone;
            return false;
        } else {
            // Increment full moves
            if main.state.side == PieceColor::Light {
                main.state.full_moves += 1;
            }
            return true;
        }
    } else {
        if mv.is_capture() {
            return make(main, attack_info, mv, MoveFlag::AllMoves);
        } else {
            return false;
        }
    }
}
