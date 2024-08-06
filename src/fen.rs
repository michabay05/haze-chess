use crate::bb::BB;
use crate::board::Board;
use crate::consts::{Piece, PieceColor, Sq};
use crate::zobrist::{self, ZobristAction};
use crate::SQ;

pub const FEN_POSITIONS: [&str; 8] = [
    "8/8/8/8/8/8/8/8 w - - 0 1",
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq e6 0 1",
];

pub const WHITE_OO_MASK: BB = 0x90;
pub const WHITE_OOO_MASK: BB = 0x11;

pub const WHITE_OO_BETWEEN_MASK: BB = 0x60;
pub const WHITE_OOO_BETWEEN_MASK: BB = 0xe;

pub const BLACK_OO_MASK: BB = 0x9000000000000000;
pub const BLACK_OOO_MASK: BB = 0x1100000000000000;

pub const BLACK_OO_BETWEEN_MASK: BB = 0x6000000000000000;
pub const BLACK_OOO_BETWEEN_MASK: BB = 0xe00000000000000;

pub const ALL_CASTLING_MASK: BB = 0x9100000000000091;

pub fn parse(fen: &str, board: &mut Board) {
    let mut fen_parts = fen.split_ascii_whitespace();

    // Place piece on square
    parse_pieces(fen_parts.next().unwrap(), board);

    // Set side to move
    let side_to_move_str: &str = fen_parts.next().unwrap();
    if side_to_move_str == "w" {
        board.state.side = PieceColor::Light;
    } else if side_to_move_str == "b" {
        board.state.side = PieceColor::Dark;
        zobrist::update(ZobristAction::ChangeColor, board);
    }

    // Set castling right
    let mut castling_rights: BB = ALL_CASTLING_MASK;
    for ch in fen_parts.next().unwrap().chars() {
        match ch {
            'K' => castling_rights &= !WHITE_OO_MASK,
            'Q' => castling_rights &= !WHITE_OOO_MASK,
            'k' => castling_rights &= !BLACK_OO_MASK,
            'q' => castling_rights &= !BLACK_OOO_MASK,
            _ => {}
        }
    }
    board.history[board.game_ply].entry = castling_rights;

    // NOTE: for now, castling isn't included in the zobrist hashing
    // zobrist::update(ZobristAction::Castling, board);

    // Set enpassant square
    let enpass_square = fen_parts.next().unwrap();
    if enpass_square != "-" {
        board.history[board.game_ply].enpassant = Sq::from_str(enpass_square);
    }
    if let Some(sq) = board.enpassant() {
        zobrist::update(ZobristAction::SetEnpassant(sq), board);
    }
    /*
    Not sure if I need this part . . .
    // Set 50 move rule
    let half_moves = fen_parts.next().unwrap_or("0").parse::<u16>().unwrap();
    board.state.half_moves = half_moves;
    // Set move counter
    let full_moves = fen_parts.next().unwrap_or("1").parse::<u16>().unwrap();
    board.state.full_moves = full_moves;
    */
}

fn parse_pieces(fen_piece: &str, board: &mut Board) {
    // todo!("fix this function");
    let (mut r, mut c) = (7, 0);
    for piece_char in fen_piece.chars() {
        if piece_char == '/' {
            r -= 1;
            c = 0;
        } else if piece_char.is_ascii_digit() {
            // Retrieve the int value of the offset from the char value
            let offset = piece_char as u8 - b'0';
            // Add offset value to square counter
            c += offset as usize;
        } else if piece_char.is_ascii_alphabetic() {
            let (piece_color, _) = Piece::to_tuple(Piece::from_char(piece_char));
            if piece_color == PieceColor::Both as usize {
                continue;
            }
            board.add_piece(Piece::from_char(piece_char), Some(Sq::from_num(SQ!(r, c))));
            // Increment the current square
            c += 1;
        }
    }
}

pub fn get_oo_mask(side: PieceColor) -> BB {
    match side {
        PieceColor::Light => WHITE_OO_MASK,
        PieceColor::Dark => BLACK_OO_MASK,
        _ => unreachable!("unknown kingside castling mask")
    }
}

pub fn get_ooo_mask(side: PieceColor) -> BB {
    match side {
        PieceColor::Light => WHITE_OOO_MASK,
        PieceColor::Dark => BLACK_OOO_MASK,
        _ => unreachable!("unknown queenside castling mask")
    }
}

pub fn get_oo_blocker_mask(side: PieceColor) -> BB {
    match side {
        PieceColor::Light => WHITE_OO_BETWEEN_MASK,
        PieceColor::Dark => BLACK_OO_BETWEEN_MASK,
        _ => unreachable!("unknown kingside castling mask")
    }
}

pub fn get_ooo_blocker_mask(side: PieceColor) -> BB {
    match side {
        PieceColor::Light => WHITE_OOO_BETWEEN_MASK,
        PieceColor::Dark => BLACK_OOO_BETWEEN_MASK,
        _ => unreachable!("unknown queenside castling mask")
    }
}

// Ignore danger on b1 for white, and b8 for black
pub fn ignore_ooo_danger(side: PieceColor) -> BB {
    match side {
        PieceColor::Light => 0x2,
        PieceColor::Dark => 0x200000000000000,
        _ => unreachable!("unknown 'queenside danger mask'")
    }
}
