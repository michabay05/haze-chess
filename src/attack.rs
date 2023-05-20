#![allow(unused_comparisons)]

use crate::bb::{BBUtil, BB};
use crate::consts::{Direction, PieceColor, Sq};
use crate::SQ;
use crate::{COL, ROW};

// Total number of square a bishop can go to from a certain square
#[rustfmt::skip]
pub const BISHOP_RELEVANT_BITS: [u32; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 5, 5, 5, 5, 5, 6,
];

// Total number of square a rook can go to from a certain square
#[rustfmt::skip]
pub const ROOK_RELEVANT_BITS: [u32; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    12, 11, 11, 11, 11, 11, 11, 12,
];

pub struct AttackInfo {
    pub pawn: [[BB; 64]; 2],
    pub knight: [BB; 64],
    pub king: [BB; 64],
}

impl AttackInfo {
    pub fn new() -> Self {
        Self {
            pawn: [[0; 64]; 2],
            knight: [0; 64],
            king: [0; 64],
        }
    }
}

pub fn init(attack: &mut AttackInfo) {
    gen_leapers(attack);
}

fn gen_leapers(attack_info: &mut AttackInfo) {
    for sq in 0..64 {
        gen_pawn(attack_info, sq, PieceColor::Light);
        gen_pawn(attack_info, sq, PieceColor::Dark);
        gen_knight(attack_info, sq);
        gen_king(attack_info, sq);
    }
}

fn gen_pawn(attack_info: &mut AttackInfo, sq: usize, color: PieceColor) {
    let bb = &mut attack_info.pawn[color as usize][sq];
    if color == PieceColor::Light {
        if ROW!(sq) > 0 && COL!(sq) > 0 {
            bb.set((sq as i32 + Direction::SW as i32) as usize);
        }
        if ROW!(sq) > 0 && COL!(sq) < 7 {
            bb.set((sq as i32 + Direction::SE as i32) as usize);
        }
    } else {
        if ROW!(sq) < 7 && COL!(sq) > 0 {
            bb.set(sq + Direction::NW as usize);
        }
        if ROW!(sq) < 7 && COL!(sq) < 7 {
            bb.set(sq + Direction::NE as usize);
        }
    }
}

fn gen_knight(attack_info: &mut AttackInfo, sq: usize) {
    let bb = &mut attack_info.knight[sq];
    if ROW!(sq) <= 5 && COL!(sq) >= 1 {
        bb.set((sq as i32 + Direction::NW_N as i32) as usize);
    }

    if ROW!(sq) <= 6 && COL!(sq) >= 2 {
        bb.set((sq as i32 + Direction::NW_W as i32) as usize);
    }

    if ROW!(sq) <= 6 && COL!(sq) <= 5 {
        bb.set((sq as i32 + Direction::NE_E as i32) as usize);
    }

    if ROW!(sq) <= 5 && COL!(sq) <= 6 {
        bb.set((sq as i32 + Direction::NE_N as i32) as usize);
    }

    if ROW!(sq) >= 2 && COL!(sq) <= 6 {
        bb.set((sq as i32 + Direction::SE_S as i32) as usize);
    }

    if ROW!(sq) >= 1 && COL!(sq) <= 5 {
        bb.set((sq as i32 + Direction::SE_E as i32) as usize);
    }

    if ROW!(sq) >= 1 && COL!(sq) >= 2 {
        bb.set((sq as i32 + Direction::SW_W as i32) as usize);
    }

    if ROW!(sq) >= 2 && COL!(sq) >= 1 {
        bb.set((sq as i32 + Direction::SW_S as i32) as usize);
    }
}

fn gen_king(attack_info: &mut AttackInfo, sq: usize) {
    let bb = &mut attack_info.king[sq];
    if ROW!(sq) > 0 {
        bb.set((sq as i32 + Direction::SOUTH as i32) as usize);
    }
    if ROW!(sq) < 7 {
        bb.set((sq as i32 + Direction::NORTH as i32) as usize);
    }
    if COL!(sq) > 0 {
        bb.set((sq as i32 + Direction::WEST as i32) as usize);
    }
    if COL!(sq) < 7 {
        bb.set((sq as i32 + Direction::EAST as i32) as usize);
    }
    if ROW!(sq) > 0 && COL!(sq) > 0 {
        bb.set((sq as i32 + Direction::SW as i32) as usize);
    }
    if ROW!(sq) > 0 && COL!(sq) < 7 {
        bb.set((sq as i32 + Direction::SE as i32) as usize);
    }
    if ROW!(sq) < 7 && COL!(sq) > 0 {
        bb.set((sq as i32 + Direction::NW as i32) as usize);
    }
    if ROW!(sq) < 7 && COL!(sq) < 7 {
        bb.set((sq as i32 + Direction::NE as i32) as usize);
    }
}

pub fn gen_bishop_occ(sq: usize) -> BB {
    let mut output: BB = 0;

    let mut r: i32;
    let mut f: i32;
    let sr = ROW!(sq) as i32;
    let sf = COL!(sq) as i32;

    // NE direction
    r = sr + 1;
    f = sf + 1;
    while r < 7 && f < 7 {
        output.set(SQ!(r, f) as usize);
        r += 1;
        f += 1;
    }

    // NW direction
    r = sr + 1;
    f = sf - 1;
    while r < 7 && f > 0 {
        output.set(SQ!(r, f) as usize);
        r += 1;
        f -= 1;
    }

    // SE direction
    r = sr - 1;
    f = sf + 1;
    while r > 0 && f < 7 {
        output.set(SQ!(r, f) as usize);
        r -= 1;
        f += 1;
    }

    // SW direction
    r = sr - 1;
    f = sf - 1;
    while r > 0 && f > 0 {
        output.set(SQ!(r, f) as usize);
        r -= 1;
        f -= 1;
    }
    output
}

pub fn gen_bishop_attack(sq: usize, blocker_board: BB) -> BB {
    let mut output: BB = 0;

    let mut r: i32;
    let mut f: i32;
    let sr = ROW!(sq) as i32;
    let sf = COL!(sq) as i32;

    // NE direction
    r = sr + 1;
    f = sf + 1;
    while r <= 7 && f <= 7 {
        output.set(SQ!(r, f) as usize);
        if blocker_board.get(SQ!(r, f) as usize) {
            break;
        }
        r += 1;
        f += 1;
    }

    // NW direction
    r = sr + 1;
    f = sf - 1;
    while r <= 7 && f >= 0 {
        output.set(SQ!(r, f) as usize);
        if blocker_board.get(SQ!(r, f) as usize) {
            break;
        }
        r += 1;
        f -= 1;
    }

    // SE direction
    r = sr - 1;
    f = sf + 1;
    while r >= 0 && f <= 7 {
        output.set(SQ!(r, f) as usize);
        if blocker_board.get(SQ!(r, f) as usize) {
            break;
        }
        r -= 1;
        f += 1;
    }

    // SW direction
    r = sr - 1;
    f = sf - 1;
    while r >= 0 && f >= 0 {
        output.set(SQ!(r, f) as usize);
        if blocker_board.get(SQ!(r, f) as usize) {
            break;
        }
        r -= 1;
        f -= 1;
    }
    output
}

pub fn gen_rook_occ(sq: usize) -> BB {
    let mut output: BB = 0;

    let mut r: i32;
    let mut f: i32;
    let sr = ROW!(sq) as i32;
    let sf = COL!(sq) as i32;

    // N direction
    r = sr + 1;
    while r < 7 {
        output.set(SQ!(r, sf) as usize);
        r += 1;
    }

    // S direction
    r = sr - 1;
    while r > 0 {
        output.set(SQ!(r, sf) as usize);
        r -= 1;
    }

    // E direction
    f = sf + 1;
    while f < 7 {
        output.set(SQ!(sr, f) as usize);
        f += 1;
    }

    // W direction
    f = sf - 1;
    while f > 0 {
        output.set(SQ!(sr, f) as usize);
        f -= 1;
    }
    output
}

pub fn gen_rook_attack(sq: usize, blocker_board: BB) -> BB {
    let mut output: BB = 0;

    let mut r: i32;
    let mut f: i32;
    let sr = ROW!(sq) as i32;
    let sf = COL!(sq) as i32;

    // N direction
    r = sr + 1;
    while r <= 7 {
        output.set(SQ!(r, sf) as usize);
        if blocker_board.get(SQ!(r, sf) as usize) {
            break;
        }
        r += 1;
    }

    // S direction
    r = sr - 1;
    while r >= 0 {
        output.set(SQ!(r, sf) as usize);
        if blocker_board.get(SQ!(r, sf) as usize) {
            break;
        }
        r -= 1;
    }

    // E direction
    f = sf + 1;
    while f <= 7 {
        output.set(SQ!(sr, f) as usize);
        if blocker_board.get(SQ!(sr, f) as usize) {
            break;
        }
        f += 1;
    }

    // W direction
    f = sf - 1;
    while f >= 0 {
        output.set(SQ!(sr, f) as usize);
        if blocker_board.get(SQ!(sr, f) as usize) {
            break;
        }
        f -= 1;
    }
    output
}

pub fn set_occ(ind: usize, relevant_bits: u32, mut occ_mask: BB) -> BB {
    let mut occ: BB = 0;
    for count in 0..relevant_bits {
        let lsb_index = occ_mask.lsb();
        occ_mask.pop(lsb_index);
        if (ind & (1 << count)) > 0 {
            occ.set(lsb_index);
        }
    }
    occ
}
