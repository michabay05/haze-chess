#[macro_export]
macro_rules! SQ {
    ($row: expr, $col: expr) => {
        /* '8' is used here because chess game always is an 8x8 game */
        ($row * 8) + $col
    }
}

#[macro_export]
macro_rules! ROW {
    ($sq: expr) => {
        $sq >> 3
    }
}

#[macro_export]
macro_rules! COL {
    ($sq: expr) => {
        $sq & 7
    }
}

pub const PIECE_CHAR: [char; 13] = [ 'P', 'N', 'B', 'R', 'Q', 'K', 'p', 'n', 'b', 'r', 'q', 'k', ' ' ];

#[derive(Copy, Clone, PartialEq)]
pub enum PieceColor {
    Light,
    Dark,
    Both
}

pub enum Piece {
    LP,
    LN,
    LB,
    LR,
    LQ,
    LK,
    DP,
    DN,
    DB,
    DR,
    DQ,
    DK,
}

pub enum Sq {
    A8, B8, C8, D8, E8, F8, G8, H8,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A1, B1, C1, D1, E1, F1, G1, H1, NoSq
}

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
