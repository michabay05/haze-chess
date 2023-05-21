use crate::consts::{Piece, Sq};

type Move = u32;

trait MoveUtil {
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
    fn to_string(&self) -> String;
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
        let promoted_num = if promoted.is_some() {
            Piece::to_num(promoted) as u32
        } else {
            0
        };
        source as u32
            | ((target as u32) << 6)
            | ((Piece::to_num(Some(piece)) as u32) << 12)
            | (promoted_num << 16)
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
    fn to_string(&self) -> String {
        let source_str = Sq::to_str(self.source());
        let target_str = Sq::to_str(self.target());
        let promoted_str = Piece::to_char(self.promoted());
        format!("{}{}{}", source_str, target_str, promoted_str)
    }
}
