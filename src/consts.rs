#[macro_export]
macro_rules! SQ {
    ($row: expr, $col: expr) => {
        /* '8' is used here because chess game always is an 8x8 game */
        ($row * 8) + $col
    };
}

#[macro_export]
macro_rules! ROW {
    ($sq: expr) => {
        $sq >> 3
    };
}

#[macro_export]
macro_rules! COL {
    ($sq: expr) => {
        $sq & 7
    };
}

pub use {SQ, ROW, COL};

#[rustfmt::skip]
pub const PIECE_CHAR: [char; 13] = ['P', 'N', 'B', 'R', 'Q', 'K', 'p', 'n', 'b', 'r', 'q', 'k', ' '];
#[rustfmt::skip]
pub const STR_COORDS: [&str; 65] = [
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", " "
];

#[derive(Copy, Clone, PartialEq)]
pub enum PieceColor {
    Light,
    Dark,
    Both,
}

#[derive(Copy, Clone, PartialEq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[rustfmt::skip]
pub enum Piece {
    LP, DP,
    LN, DN,
    LB, DB,
    LR, DR,
    LQ, DQ,
    LK, DK
}

#[rustfmt::skip]
pub enum Sq {
    A8, B8, C8, D8, E8, F8, G8, H8,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A1, B1, C1, D1, E1, F1, G1, H1,
    NoSq,
}

#[allow(non_camel_case_types)]
pub enum Direction {
    NORTH = 8,
    SOUTH = -8,
    WEST = -1,
    EAST = 1,
    NE = 9,     // NORTH + EAST
    NW = 7,     // NORTH + WEST
    SE = -7,    // SOUTH + EAST
    SW = -9,    // SOUTH + WEST
    NE_N = 17,  // 2(NORTH) + EAST -> 'KNIGHT ONLY'
    NE_E = 10,  // NORTH + 2(EAST) -> 'KNIGHT ONLY'
    NW_N = 15,  // 2(NORTH) + WEST -> 'KNIGHT ONLY'
    NW_W = 6,   // NORTH + 2(WEST) -> 'KNIGHT ONLY'
    SE_S = -15, // 2(SOUTH) + EAST -> 'KNIGHT ONLY'
    SE_E = -6,  // SOUTH + 2(EAST) -> 'KNIGHT ONLY'
    SW_S = -17, // 2(SOUTH) + WEST -> 'KNIGHT ONLY'
    SW_W = -10, // SOUTH + 2(WEST) -> 'KNIGHT ONLY'
}
