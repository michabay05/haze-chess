#![allow(dead_code)]

use std::fmt;

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
        ($sq >> 3)
    };
}

#[macro_export]
macro_rules! COL {
    ($sq: expr) => {
        ($sq & 7)
    };
}

#[macro_export]
macro_rules! FLIP_SQ {
    ($sq: expr) => {
        ($sq ^ 56)
    };
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PieceColor {
    Light,
    Dark,
    Both,
}

impl PieceColor {
    pub fn opposite(self) -> Self {
        if self == Self::Light {
            Self::Dark
        } else {
            Self::Light
        }
    }
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

#[derive(Copy, Clone, PartialEq)]
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

impl Piece {
    pub fn from_tuple(color: usize, piece_type: usize) -> Option<Self> {
        if color != (PieceColor::Both as usize) {
            return None;
        }
        Self::from_num(color * 6 + piece_type)
    }

    pub fn from_num(piece_num: usize) -> Option<Self> {
        assert!(piece_num <= 12);
        if piece_num == 12 {
            return None;
        }
        let piece = match piece_num {
            0 => Self::LP,
            1 => Self::LN,
            2 => Self::LB,
            3 => Self::LR,
            4 => Self::LQ,
            5 => Self::LK,
            6 => Self::DP,
            7 => Self::DN,
            8 => Self::DB,
            9 => Self::DR,
            10 => Self::DQ,
            11 => Self::DK,
            _ => unreachable!("Piece::from_num() should only contain these values: [0-11]"),
        };
        Some(piece)
    }

    pub fn from_type(color: PieceColor, pt: PieceType) -> Self {
        match color {
            PieceColor::Light => match pt {
                PieceType::Pawn => Self::LP,
                PieceType::Knight => Self::LN,
                PieceType::Bishop => Self::LB,
                PieceType::Rook => Self::LR,
                PieceType::Queen => Self::LQ,
                PieceType::King => Self::LK
            }
            PieceColor::Dark => match pt {
                PieceType::Pawn => Self::DP,
                PieceType::Knight => Self::DN,
                PieceType::Bishop => Self::DB,
                PieceType::Rook => Self::DR,
                PieceType::Queen => Self::DQ,
                PieceType::King => Self::DK
            }
            PieceColor::Both => unreachable!("Shouldn't be here!"),
        }
    }

    pub fn piece_type(&self) -> PieceType {
        match *self {
            Self::LP | Self::DP => PieceType::Pawn,
            Self::LN | Self::DN => PieceType::Knight,
            Self::LB | Self::DB => PieceType::Bishop,
            Self::LR | Self::DR => PieceType::Rook,
            Self::LQ | Self::DQ => PieceType::Queen,
            Self::LK | Self::DK => PieceType::King,
        }
    }

    pub fn from_char(piece_char: char) -> Option<Self> {
        assert!(PIECE_CHAR.into_iter().any(|x| x == piece_char));
        let char_index = PIECE_CHAR
            .into_iter()
            .position(|x| x == piece_char)
            .unwrap();
        Self::from_num(char_index)
    }

    pub fn to_tuple(piece: Option<Piece>) -> (usize, usize) {
        if piece.is_none() {
            return (2, 0);
        }
        let piece_num = piece.unwrap() as usize;
        (piece_num / 6, piece_num % 6)
    }

    pub fn to_num(piece: Option<Piece>) -> usize {
        if piece.is_none() {
            return 12;
        }
        piece.unwrap() as usize
    }

    pub fn to_char(piece: Option<Piece>) -> char {
        PIECE_CHAR[Self::to_num(piece)]
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[repr(i32)]
pub enum Direction {
    North = 8,
    South = -8,
    West = -1,
    East = 1,
    Northeast = 9,     // NORTH + EAST
    Northwest = 7,     // NORTH + WEST
    Southeast = -7,    // SOUTH + EAST
    Southwest = -9,    // SOUTH + WEST
    NorthNorth = 16,
    SouthSouth = -16,
    NE_N = 17,  // 2(NORTH) + EAST -> 'KNIGHT ONLY'
    NE_E = 10,  // NORTH + 2(EAST) -> 'KNIGHT ONLY'
    NW_N = 15,  // 2(NORTH) + WEST -> 'KNIGHT ONLY'
    NW_W = 6,   // NORTH + 2(WEST) -> 'KNIGHT ONLY'
    SE_S = -15, // 2(SOUTH) + EAST -> 'KNIGHT ONLY'
    SE_E = -6,  // SOUTH + 2(EAST) -> 'KNIGHT ONLY'
    SW_S = -17, // 2(SOUTH) + WEST -> 'KNIGHT ONLY'
    SW_W = -10, // SOUTH + 2(WEST) -> 'KNIGHT ONLY'
}

impl Direction {
    pub fn relative(&self, side: PieceColor) -> Self {
        if side == PieceColor::Light {
            return *self;
        }
        match *self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
            Self::NorthNorth => Self::SouthSouth,
            Self::SouthSouth => Self::NorthNorth,
            Self::Northeast => Self::Southwest,
            Self::Northwest => Self::Southeast,
            Self::Southeast => Self::Northwest,
            Self::Southwest => Self::Northeast,
            Self::NE_E => Self::SE_E,
            Self::NE_N => Self::SE_S,
            Self::NW_W => Self::SW_W,
            Self::NW_N => Self::SW_S,
            Self::SE_E => Self::NE_E,
            Self::SE_S => Self::NE_N,
            Self::SW_W => Self::NW_W,
            Self::SW_S => Self::NW_N,
        }
    }
}

#[rustfmt::skip]
#[derive(Copy, Clone, PartialEq)]
pub enum File {
    A, B, C, D, E, F, G, H
}

#[rustfmt::skip]
#[derive(Copy, Clone, PartialEq)]
pub enum Sq {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8
}

impl fmt::Display for Sq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Sq::to_string(*self))
    }
}

impl Sq {
    pub fn from_str(sq_str: &str) -> Option<Sq> {
        if sq_str.len() != 2 {
            return None;
        }
        let file = (sq_str.chars().nth(0).unwrap() as u8) - b'a';
        let rank = 8 - (sq_str.chars().nth(1).unwrap() as u8 - b'0');
        Some(Self::from_num(SQ!(rank, file) as usize))
    }

    pub fn from_num(sq_num: usize) -> Self {
        match sq_num {
            0 => Self::A1,
            1 => Self::B1,
            2 => Self::C1,
            3 => Self::D1,
            4 => Self::E1,
            5 => Self::F1,
            6 => Self::G1,
            7 => Self::H1,
            8 => Self::A2,
            9 => Self::B2,
            10 => Self::C2,
            11 => Self::D2,
            12 => Self::E2,
            13 => Self::F2,
            14 => Self::G2,
            15 => Self::H2,
            16 => Self::A3,
            17 => Self::B3,
            18 => Self::C3,
            19 => Self::D3,
            20 => Self::E3,
            21 => Self::F3,
            22 => Self::G3,
            23 => Self::H3,
            24 => Self::A4,
            25 => Self::B4,
            26 => Self::C4,
            27 => Self::D4,
            28 => Self::E4,
            29 => Self::F4,
            30 => Self::G4,
            31 => Self::H4,
            32 => Self::A5,
            33 => Self::B5,
            34 => Self::C5,
            35 => Self::D5,
            36 => Self::E5,
            37 => Self::F5,
            38 => Self::G5,
            39 => Self::H5,
            40 => Self::A6,
            41 => Self::B6,
            42 => Self::C6,
            43 => Self::D6,
            44 => Self::E6,
            45 => Self::F6,
            46 => Self::G6,
            47 => Self::H6,
            48 => Self::A7,
            49 => Self::B7,
            50 => Self::C7,
            51 => Self::D7,
            52 => Self::E7,
            53 => Self::F7,
            54 => Self::G7,
            55 => Self::H7,
            56 => Self::A8,
            57 => Self::B8,
            58 => Self::C8,
            59 => Self::D8,
            60 => Self::E8,
            61 => Self::F8,
            62 => Self::G8,
            63 => Self::H8,
            _ => unreachable!("Sq::from_num() should only contain these values: [0-63]"),
        }
    }

    pub fn to_string(sq_num: Self) -> String {
        STR_COORDS[sq_num as usize].to_string()
    }
}

pub use SQ;

use crate::bb::BB;

#[rustfmt::skip]
const PIECE_CHAR: [char; 13] = ['P', 'N', 'B', 'R', 'Q', 'K', 'p', 'n', 'b', 'r', 'q', 'k', ' '];
#[rustfmt::skip]
const STR_COORDS: [&str; 64] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
];

pub const MASK_FILE: [BB; 8] = [
    0x101010101010101, 0x202020202020202, 0x404040404040404, 0x808080808080808,
    0x1010101010101010, 0x2020202020202020, 0x4040404040404040, 0x8080808080808080,
];

pub const MASK_RANK: [BB; 8] = [
    0xff, 0xff00, 0xff0000, 0xff000000,
    0xff00000000, 0xff0000000000, 0xff000000000000, 0xff00000000000000
];
