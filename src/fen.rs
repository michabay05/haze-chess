use crate::board::{Board, CastlingType};
use crate::consts::{Piece, PieceColor, Sq};
use crate::zobrist::{self, ZobristAction};

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
    for castling_type in fen_parts.next().unwrap().chars() {
        if castling_type == 'K' {
            board
                .state
                .toggle_castling(CastlingType::WhiteKingside as usize);
        } else if castling_type == 'Q' {
            board
                .state
                .toggle_castling(CastlingType::WhiteQueenside as usize);
        } else if castling_type == 'k' {
            board
                .state
                .toggle_castling(CastlingType::BlackKingside as usize);
        } else if castling_type == 'q' {
            board
                .state
                .toggle_castling(CastlingType::BlackQueenside as usize);
        }
    }
    zobrist::update(ZobristAction::Castling, board);

    // Set enpassant square
    let enpass_square = fen_parts.next().unwrap();
    if enpass_square != "-" {
        board.state.enpassant = Sq::from_str(enpass_square);
    }
    zobrist::update(ZobristAction::Enpassant, board);
    // Set 50 move rule
    let half_moves = fen_parts.next().unwrap().parse::<u32>().unwrap();
    board.state.half_moves = half_moves;
    // Set move counter
    let full_moves = fen_parts.next().unwrap().parse::<u32>().unwrap();
    board.state.full_moves = full_moves;
}

fn parse_pieces(fen_piece: &str, board: &mut Board) {
    let mut sq: usize = 0;
    for piece_char in fen_piece.chars() {
        if piece_char == '/' {
            continue;
        } else if piece_char.is_ascii_digit() {
            // Retrieve the int value of the offset from the char value
            let offset = piece_char as u8 - b'0';
            // Add offset value to square counter
            sq += offset as usize;
        } else if piece_char.is_ascii_alphabetic() {
            let (piece_color, _) = Piece::to_tuple(Piece::from_char(piece_char));
            if piece_color == PieceColor::Both as usize {
                continue;
            }
            board.add_piece(Piece::from_char(piece_char), Sq::from_num(sq));
            // Increment the current square
            sq += 1;
        }
    }
}
