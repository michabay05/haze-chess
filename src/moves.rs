use crate::{attack::AttackInfo, board::Board, consts::Sq};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u16)]
pub enum MoveFlag {
    // Quiets
    Quiet,
    DoublePush,
    KingSideCastling,
    QueenSideCastling,
    // Promotions
    PromKnight,
    PromBishop,
    PromRook,
    PromQueen,
    // Captures
    Capture,
    PromCapKnight,
    PromCapBishop,
    PromCapRook,
    PromCapQueen,
    Enpassant,
}

pub type Move = u16;

pub trait MoveUtil {
    fn encode(source: Sq, target: Sq, flags: MoveFlag) -> Self;
    fn source(&self) -> Sq;
    fn target(&self) -> Sq;
    fn flag(&self) -> MoveFlag;
    fn is_capture(&self) -> bool;
    fn is_promotion(&self) -> bool;
    fn from_str(move_str: &str, flag: MoveFlag) -> Option<Self>
    where
        Self: Sized;
    fn to_str(&self) -> String;
}

impl MoveUtil for Move {
    fn encode(source: Sq, target: Sq, flag: MoveFlag) -> Self {
        source as u16 | ((target as u16) << 6) | ((flag as u16) << 12)
    }

    fn source(&self) -> Sq {
        Sq::from_num((*self & 0x3F) as usize)
    }

    fn target(&self) -> Sq {
        Sq::from_num(((*self & 0xFC0) >> 6) as usize)
    }

    fn flag(&self) -> MoveFlag {
        match *self >> 12 {
            0 => MoveFlag::Quiet,
            1 => MoveFlag::DoublePush,
            2 => MoveFlag::KingSideCastling,
            3 => MoveFlag::QueenSideCastling,
            // Promotions
            4 => MoveFlag::PromKnight,
            5 => MoveFlag::PromBishop,
            6 => MoveFlag::PromRook,
            7 => MoveFlag::PromQueen,
            // Captures
            8 => MoveFlag::Capture,
            9 => MoveFlag::PromCapKnight,
            10 => MoveFlag::PromCapBishop,
            11 => MoveFlag::PromCapRook,
            12 => MoveFlag::PromCapQueen,
            13 => MoveFlag::Enpassant,
            _ => unreachable!("Unknown move flag!"),
        }
    }

    fn is_capture(&self) -> bool {
        match self.flag() {
            MoveFlag::Capture
            | MoveFlag::PromCapKnight
            | MoveFlag::PromCapBishop
            | MoveFlag::PromCapRook
            | MoveFlag::PromCapQueen
            | MoveFlag::Enpassant => true,
            _ => false,
        }
    }

    fn is_promotion(&self) -> bool {
        match self.flag() {
            MoveFlag::PromKnight
            | MoveFlag::PromBishop
            | MoveFlag::PromRook
            | MoveFlag::PromQueen
            | MoveFlag::PromCapKnight
            | MoveFlag::PromCapBishop
            | MoveFlag::PromCapRook
            | MoveFlag::PromCapQueen => true,
            _ => false,
        }
    }

    fn from_str(move_str: &str, flag: MoveFlag) -> Option<Self>
    where
        Self: Sized,
    {
        // This should be modified to include a legality check of the given move
        if move_str.len() != 4 && move_str.len() != 5 {
            return None;
        }
        let source = Sq::from_str(&move_str[0..2]);
        let target = Sq::from_str(&move_str[2..4]);
        if source.is_none() || target.is_none() {
            return None;
        }
        let source = source.unwrap();
        let target = target.unwrap();
        Some(Self::encode(source, target, flag))
    }

    fn to_str(&self) -> String {
        let source_str = Sq::to_string(self.source());
        let target_str = Sq::to_string(self.target());
        let promoted_ch = match self.flag() {
            MoveFlag::PromKnight | MoveFlag::PromCapKnight => 'n',
            MoveFlag::PromBishop | MoveFlag::PromCapBishop => 'b',
            MoveFlag::PromRook | MoveFlag::PromCapRook => 'r',
            MoveFlag::PromQueen | MoveFlag::PromCapQueen => 'q',
            _ => ' ',
        };

        format!("{}{}{}", source_str, target_str, promoted_ch)
    }
}

#[derive(PartialEq)]
pub enum MoveType {
    AllMoves,
    CapturesOnly,
}

const CASTLING_RIGHTS: [usize; 64] = [
    7, 15, 15, 15, 3, 15, 15, 11, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 13, 15, 15, 15, 12, 15, 15, 14,
];

pub fn play_move(
    _main: &mut Board,
    _attack_info: &AttackInfo,
    _mv: Move,
    _move_flag: MoveType,
) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::{Move, MoveFlag, MoveUtil, Sq};

    #[test]
    fn move_encode() {
        let mut mv: Move = 0;
        assert_eq!(mv.source(), Sq::A1);
        assert_eq!(mv.target(), Sq::A1);

        mv = Move::encode(Sq::G1, Sq::F3, MoveFlag::Quiet);
        assert_eq!(mv.source(), Sq::G1);
        assert_eq!(mv.target(), Sq::F3);
        assert_eq!(mv.flag(), MoveFlag::Quiet);

        mv = Move::encode(Sq::E2, Sq::E4, MoveFlag::DoublePush);
        assert_eq!(mv.source(), Sq::E2);
        assert_eq!(mv.target(), Sq::E4);
        assert_eq!(mv.flag(), MoveFlag::DoublePush);

        mv = Move::encode(Sq::E4, Sq::E5, MoveFlag::Capture);
        assert_eq!(mv.source(), Sq::E4);
        assert_eq!(mv.target(), Sq::E5);
        assert_eq!(mv.flag(), MoveFlag::Capture);
    }
}
// ) -> bool {
//     if move_flag == MoveType::AllMoves {
//         let clone = main.clone();

//         // Extract information about the move
//         let source = mv.source() as usize;
//         let target = mv.target() as usize;
//         // let piece = Piece::to_num(Some(mv.piece()));
//         let piece = main.pos.mailbox[source as usize];
//         let promoted = mv.promoted();
//         let is_capture = mv.is_capture();
//         let is_twosquare = mv.is_twosquare();
//         let is_enpassant = mv.is_enpassant();
//         let is_castling = mv.is_castling();

//         // Move piece from source to target by removing source bit and turning on the target bit
//         main.pos.bitboards[piece].pop(source);
//         main.pos.bitboards[piece].set(target);

//         // Update hash key and lock
//         zobrist::update(
//             ZobristAction::TogglePiece(Piece::from_num(piece).unwrap(), Sq::from_num(source)),
//             main,
//         );
//         zobrist::update(
//             ZobristAction::TogglePiece(Piece::from_num(piece).unwrap(), Sq::from_num(target)),
//             main,
//         );

//         if is_capture {
//             let (start, end) = if main.state.side == PieceColor::Light {
//                 (Piece::DP as usize, Piece::DK as usize)
//             } else {
//                 (Piece::LP as usize, Piece::LK as usize)
//             };
//             for bb_piece in start..=end {
//                 if main.pos.bitboards[bb_piece].get(target) {
//                     main.pos.bitboards[bb_piece].pop(target);
//                     zobrist::update(
//                         ZobristAction::TogglePiece(
//                             Piece::from_num(bb_piece).unwrap(),
//                             Sq::from_num(target),
//                         ),
//                         main,
//                     );
//                     break;
//                 }
//             }
//         }

//         if promoted.is_some() {
//             assert!(piece == 0 || piece == 6);
//             let promoted_num = Piece::to_num(promoted);
//             let pawn_type = if piece == 0 { Piece::LP } else { Piece::DP } as usize;
//             main.pos.bitboards[pawn_type].pop(target);
//             zobrist::update(
//                 ZobristAction::TogglePiece(
//                     Piece::from_num(pawn_type).unwrap(),
//                     Sq::from_num(target),
//                 ),
//                 main,
//             );

//             main.pos.bitboards[promoted_num].set(target);
//             zobrist::update(
//                 ZobristAction::TogglePiece(
//                     Piece::from_num(promoted_num).unwrap(),
//                     Sq::from_num(target),
//                 ),
//                 main,
//             );
//         }

//         if is_enpassant {
//             let pawn_type;
//             let direction;
//             if main.state.side == PieceColor::Light {
//                 pawn_type = Piece::DP;
//                 direction = Direction::North;
//             } else {
//                 pawn_type = Piece::LP;
//                 direction = Direction::South;
//             }
//             main.pos.bitboards[pawn_type as usize].pop((target as i32 + direction as i32) as usize);
//             zobrist::update(
//                 ZobristAction::TogglePiece(
//                     pawn_type,
//                     Sq::from_num((target as i32 + direction as i32) as usize),
//                 ),
//                 main,
//             );
//         }
//         if main.state.enpassant.is_some() {
//             zobrist::update(ZobristAction::Enpassant, main);
//         }
//         main.state.enpassant = None;

//         if is_twosquare {
//             if main.state.side == PieceColor::Light {
//                 main.state.enpassant = Some(Sq::from_num(
//                     (target as i32 + Direction::North as i32) as usize,
//                 ));
//             } else {
//                 main.state.enpassant = Some(Sq::from_num(
//                     (target as i32 + Direction::South as i32) as usize,
//                 ));
//             }
//             zobrist::update(ZobristAction::Enpassant, main);
//         }

//         if is_castling {
//             let rook_type;
//             let source_castling;
//             let target_castling;
//             match Sq::from_num(target) {
//                 Sq::G1 => {
//                     rook_type = Piece::LR;
//                     source_castling = Sq::H1;
//                     target_castling = Sq::F1;
//                 },
//                 Sq::C1 => {
//                     rook_type = Piece::LR;
//                     source_castling = Sq::A1;
//                     target_castling = Sq::D1;
//                 },
//                 Sq::G8 => {
//                     rook_type = Piece::DR;
//                     source_castling = Sq::H8;
//                     target_castling = Sq::F8;
//                 },
//                 Sq::C8 => {
//                     rook_type = Piece::DR;
//                     source_castling = Sq::A8;
//                     target_castling = Sq::D8;
//                 },
//                 _ => unreachable!("Target castling square should only be [ G1, C1 ] for white and [ G8, C8 ] for black"),
//             };
//             main.pos.bitboards[rook_type as usize].pop(source_castling as usize);
//             zobrist::update(ZobristAction::TogglePiece(rook_type, source_castling), main);

//             main.pos.bitboards[rook_type as usize].set(target_castling as usize);
//             zobrist::update(ZobristAction::TogglePiece(rook_type, target_castling), main);
//         }

//         zobrist::update(ZobristAction::Castling, main);
//         main.state.castling &= CASTLING_RIGHTS[source] as u8;
//         main.state.castling &= CASTLING_RIGHTS[target] as u8;
//         zobrist::update(ZobristAction::Castling, main);

//         main.state.change_side();
//         zobrist::update(ZobristAction::ChangeColor, main);

//         /* ============= FOR DEBUG PURPOSES ONLY ===============
//         let key_from_scratch = zobrist::gen_board_key(&zobrist_info.key, &main);
//         let lock_from_scratch = zobrist::gen_board_lock(&zobrist_info.lock, &main);
//         assert!(
//             main.state.key == key_from_scratch,
//             "Incorrect key: main.state.key({}), from_scratch({})",
//             main.state.key,
//             key_from_scratch
//         );
//         assert!(
//             main.state.lock == lock_from_scratch,
//             "Incorrect lock: main.state.lock({}), from_scratch({})",
//             main.state.lock,
//             lock_from_scratch
//         );
//          ============= FOR DEBUG PURPOSES ONLY =============== */
//         // if board::in_check(main, attack_info, main.state.side) {
//         if main.is_in_check(attack_info, main.state.side) {
//             *main = clone;
//             false
//         } else {
//             // Increment full moves
//             if main.state.side == PieceColor::Dark {
//                 main.state.full_moves += 1;
//             }
//             true
//         }
//     } else if mv.is_capture() {
//         play_move(main, attack_info, mv, MoveType::AllMoves)
//     } else {
//         false
//     }
// }
