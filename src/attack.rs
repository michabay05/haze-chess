#![allow(unused_comparisons)]

use crate::bb::{BBUtil, BB};
use crate::consts::{PieceColor, PieceType, Sq};
use crate::magics::{BISHOP_MAGICS, ROOK_MAGICS};
use crate::{COL, ROW, SQ};

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

#[derive(Clone)]
pub struct AttackInfo {
    pub bishop_occ_mask: [BB; 64],
    pub bishop_attack: Vec<Vec<BB>>,
    pub rook_occ_mask: [BB; 64],
    pub rook_attack: Vec<Vec<BB>>,
    pub squares_between: [[BB; 64]; 64],
}

impl AttackInfo {
    pub fn new() -> Self {
        Self {
            bishop_occ_mask: [0; 64],
            bishop_attack: vec![vec![0; 512]; 64],
            rook_occ_mask: [0; 64],
            rook_attack: vec![vec![0; 4096]; 64],
            squares_between: [[0; 64]; 64],
        }
    }

    pub fn init(&mut self) {
        // NOTE: the function below is only used to generate the leaper pieces' constants
        // gen_leapers();
        gen_sliding(self, PieceType::Bishop);
        gen_sliding(self, PieceType::Rook);
        self.gen_squares_between();
    }

    pub fn get_attack(&self, color: PieceColor, pt: PieceType, sq: Sq, blocker_board: BB) -> BB {
        match pt {
            // Sliders
            PieceType::Rook => self.get_rook_attack(sq, blocker_board),
            PieceType::Bishop => self.get_bishop_attack(sq, blocker_board),
            PieceType::Queen => self.get_bishop_attack(sq, blocker_board) | self.get_rook_attack(sq, blocker_board),
            // Leapers
            PieceType::Pawn => {
                if color == PieceColor::Light {
                    LIGHT_PAWN_ATTACKS[sq as usize]
                } else {
                    DARK_PAWN_ATTACKS[sq as usize]
                }
            },
            PieceType::Knight => KNIGHT_ATTACKS[sq as usize],
            PieceType::King => KING_ATTACKS[sq as usize],
        }
    }

    pub fn get_bishop_attack(&self, sq: Sq, blocker_board: BB) -> BB {
        let mut blocker = blocker_board;
        blocker &= self.bishop_occ_mask[sq as usize];
        let num = u128::from(blocker) * u128::from(BISHOP_MAGICS[sq as usize]);
        blocker = (num as u64) >> (64 - BISHOP_RELEVANT_BITS[sq as usize]);
        self.bishop_attack[sq as usize][blocker as usize]
    }

    pub fn get_rook_attack(&self, sq: Sq, blocker_board: BB) -> BB {
        let mut blocker = blocker_board;
        blocker &= self.rook_occ_mask[sq as usize];
        let num = u128::from(blocker) * u128::from(ROOK_MAGICS[sq as usize]);
        blocker = (num as u64) >> (64 - ROOK_RELEVANT_BITS[sq as usize]);
        self.rook_attack[sq as usize][blocker as usize]
    }

    pub fn get_queen_attack(&self, sq: Sq, blocker_board: BB) -> BB {
        self.get_bishop_attack(sq, blocker_board) | self.get_rook_attack(sq, blocker_board)
    }

    pub fn gen_squares_between(&mut self) {
        let mut sq1 = Sq::A1 as usize;
        while sq1 < Sq::H8 as usize {
            let mut sq2 = Sq::A1 as usize;

            while sq2 < Sq::H8 as usize {
                let mut sqs: BB = 0;
                sqs.set(sq1);
                sqs.set(sq2);

                if COL!(sq1) == COL!(sq2) || ROW!(sq1) == ROW!(sq2) {
                    self.squares_between[sq1][sq2] = self.get_rook_attack(Sq::from_num(sq1), sqs)
                        & self.get_rook_attack(Sq::from_num(sq2), sqs);
                } else if diagonal(sq1 as i32) == diagonal(sq2 as i32)
                    || anti_diagonal(sq1 as i32) == anti_diagonal(sq2 as i32)
                {
                    self.squares_between[sq1][sq2] = self.get_bishop_attack(Sq::from_num(sq1), sqs)
                        & self.get_bishop_attack(Sq::from_num(sq2), sqs);
                } else {
                    self.squares_between[sq1][sq2] = 0;
                }

                sq2 += 1;
            }

            sq1 += 1;
        }
    }
}

fn diagonal(sq: i32) -> i32 {
    7 + ROW!(sq) + COL!(sq)
}

fn anti_diagonal(sq: i32) -> i32 {
    ROW!(sq) + COL!(sq)
}

fn gen_sliding(attack_info: &mut AttackInfo, piece: PieceType) {
    for sq in 0..64 {
        attack_info.bishop_occ_mask[sq] = gen_bishop_occ(sq);
        attack_info.rook_occ_mask[sq] = gen_rook_occ(sq);

        let curr_mask = if piece == PieceType::Bishop {
            attack_info.bishop_occ_mask[sq]
        } else {
            attack_info.rook_occ_mask[sq]
        };

        let bit_count = curr_mask.count_ones();
        for count in 0..(1 << bit_count) {
            let occupancy = set_occ(count, bit_count, curr_mask);
            let magic_ind;
            if piece == PieceType::Bishop {
                let num = u128::from(occupancy) * u128::from(BISHOP_MAGICS[sq]);
                magic_ind = (num as u64) >> (64 - bit_count);
                attack_info.bishop_attack[sq][magic_ind as usize] =
                    gen_bishop_attack(sq, occupancy);
            } else {
                let num = u128::from(occupancy) * u128::from(ROOK_MAGICS[sq]);
                magic_ind = (num as u64) >> (64 - bit_count);
                attack_info.rook_attack[sq][magic_ind as usize] = gen_rook_attack(sq, occupancy);
            }
        }
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

#[rustfmt::skip]
pub const LIGHT_PAWN_ATTACKS: [BB; 64] = [
 0x200, 0x500, 0xa00, 0x1400,
    0x2800, 0x5000, 0xa000, 0x4000,
    0x20000, 0x50000, 0xa0000, 0x140000,
    0x280000, 0x500000, 0xa00000, 0x400000,
    0x2000000, 0x5000000, 0xa000000, 0x14000000,
    0x28000000, 0x50000000, 0xa0000000, 0x40000000,
    0x200000000, 0x500000000, 0xa00000000, 0x1400000000,
    0x2800000000, 0x5000000000, 0xa000000000, 0x4000000000,
    0x20000000000, 0x50000000000, 0xa0000000000, 0x140000000000,
    0x280000000000, 0x500000000000, 0xa00000000000, 0x400000000000,
    0x2000000000000, 0x5000000000000, 0xa000000000000, 0x14000000000000,
    0x28000000000000, 0x50000000000000, 0xa0000000000000, 0x40000000000000,
    0x200000000000000, 0x500000000000000, 0xa00000000000000, 0x1400000000000000,
    0x2800000000000000, 0x5000000000000000, 0xa000000000000000, 0x4000000000000000,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
];

#[rustfmt::skip]
pub const DARK_PAWN_ATTACKS: [BB; 64] = [
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x2, 0x5, 0xa, 0x14,
    0x28, 0x50, 0xa0, 0x40,
    0x200, 0x500, 0xa00, 0x1400,
    0x2800, 0x5000, 0xa000, 0x4000,
    0x20000, 0x50000, 0xa0000, 0x140000,
    0x280000, 0x500000, 0xa00000, 0x400000,
    0x2000000, 0x5000000, 0xa000000, 0x14000000,
    0x28000000, 0x50000000, 0xa0000000, 0x40000000,
    0x200000000, 0x500000000, 0xa00000000, 0x1400000000,
    0x2800000000, 0x5000000000, 0xa000000000, 0x4000000000,
    0x20000000000, 0x50000000000, 0xa0000000000, 0x140000000000,
    0x280000000000, 0x500000000000, 0xa00000000000, 0x400000000000,
    0x2000000000000, 0x5000000000000, 0xa000000000000, 0x14000000000000,
    0x28000000000000, 0x50000000000000, 0xa0000000000000, 0x40000000000000,
];

#[rustfmt::skip]
pub const KNIGHT_ATTACKS: [BB; 64] = [
    0x20400, 0x50800, 0xa1100, 0x142200,
    0x284400, 0x508800, 0xa01000, 0x402000,
    0x2040004, 0x5080008, 0xa110011, 0x14220022,
    0x28440044, 0x50880088, 0xa0100010, 0x40200020,
    0x204000402, 0x508000805, 0xa1100110a, 0x1422002214,
    0x2844004428, 0x5088008850, 0xa0100010a0, 0x4020002040,
    0x20400040200, 0x50800080500, 0xa1100110a00, 0x142200221400,
    0x284400442800, 0x508800885000, 0xa0100010a000, 0x402000204000,
    0x2040004020000, 0x5080008050000, 0xa1100110a0000, 0x14220022140000,
    0x28440044280000, 0x50880088500000, 0xa0100010a00000, 0x40200020400000,
    0x204000402000000, 0x508000805000000, 0xa1100110a000000, 0x1422002214000000,
    0x2844004428000000, 0x5088008850000000, 0xa0100010a0000000, 0x4020002040000000,
    0x400040200000000, 0x800080500000000, 0x1100110a00000000, 0x2200221400000000,
    0x4400442800000000, 0x8800885000000000, 0x100010a000000000, 0x2000204000000000,
    0x4020000000000, 0x8050000000000, 0x110a0000000000, 0x22140000000000,
    0x44280000000000, 0x88500000000000, 0x10a00000000000, 0x20400000000000
];

#[rustfmt::skip]
pub const KING_ATTACKS: [BB; 64] = [
    0x302, 0x705, 0xe0a, 0x1c14,
    0x3828, 0x7050, 0xe0a0, 0xc040,
    0x30203, 0x70507, 0xe0a0e, 0x1c141c,
    0x382838, 0x705070, 0xe0a0e0, 0xc040c0,
    0x3020300, 0x7050700, 0xe0a0e00, 0x1c141c00,
    0x38283800, 0x70507000, 0xe0a0e000, 0xc040c000,
    0x302030000, 0x705070000, 0xe0a0e0000, 0x1c141c0000,
    0x3828380000, 0x7050700000, 0xe0a0e00000, 0xc040c00000,
    0x30203000000, 0x70507000000, 0xe0a0e000000, 0x1c141c000000,
    0x382838000000, 0x705070000000, 0xe0a0e0000000, 0xc040c0000000,
    0x3020300000000, 0x7050700000000, 0xe0a0e00000000, 0x1c141c00000000,
    0x38283800000000, 0x70507000000000, 0xe0a0e000000000, 0xc040c000000000,
    0x302030000000000, 0x705070000000000, 0xe0a0e0000000000, 0x1c141c0000000000,
    0x3828380000000000, 0x7050700000000000, 0xe0a0e00000000000, 0xc040c00000000000,
    0x203000000000000, 0x507000000000000, 0xa0e000000000000, 0x141c000000000000,
    0x2838000000000000, 0x5070000000000000, 0xa0e0000000000000, 0x40c0000000000000,
];

// fn gen_leapers() {
//     gen_pawn(PieceColor::Light);
//     gen_pawn(PieceColor::Dark);
//     gen_knight();
//     gen_king();
// }

// fn gen_pawn(side: PieceColor) {
//     println!("pub const {:?}_PAWN_ATTACKS: [BB; 64] = [", side);
//     for sq in 0..64 {
//         let mut bb = 0;
//         if side == PieceColor::Light {
//             if ROW!(sq) < 7 && COL!(sq) > 0 {
//                 bb.set((sq as i32 + Direction::Northwest.relative(side) as i32) as usize);
//             }
//             if ROW!(sq) < 7 && COL!(sq) < 7 {
//                 bb.set((sq as i32 + Direction::Northeast.relative(side) as i32) as usize);
//             }
//         } else {
//             if ROW!(sq) > 0 && COL!(sq) > 0 {
//                 bb.set((sq as i32 + Direction::Northeast.relative(side) as i32) as usize);
//             }
//             if ROW!(sq) > 0 && COL!(sq) < 7 {
//                 bb.set((sq as i32 + Direction::Northwest.relative(side) as i32) as usize);
//             }
//         }
//         print!("0x{:x}, ", bb);
//         if sq % 4 == 3 {
//             println!();
//         }
//     }
//     println!("];");
// }

// fn gen_knight() {
//     println!("pub const KNIGHT_ATTACKS: [BB; 64] = [");
//     for sq in 0..64 {
//         let mut bb = 0;
//         if ROW!(sq) <= 5 && COL!(sq) >= 1 {
//             bb.set((sq as i32 + Direction::NW_N as i32) as usize);
//         }

//         if ROW!(sq) <= 6 && COL!(sq) >= 2 {
//             bb.set((sq as i32 + Direction::NW_W as i32) as usize);
//         }

//         if ROW!(sq) <= 6 && COL!(sq) <= 5 {
//             bb.set((sq as i32 + Direction::NE_E as i32) as usize);
//         }

//         if ROW!(sq) <= 5 && COL!(sq) <= 6 {
//             bb.set((sq as i32 + Direction::NE_N as i32) as usize);
//         }

//         if ROW!(sq) >= 2 && COL!(sq) <= 6 {
//             bb.set((sq as i32 + Direction::SE_S as i32) as usize);
//         }

//         if ROW!(sq) >= 1 && COL!(sq) <= 5 {
//             bb.set((sq as i32 + Direction::SE_E as i32) as usize);
//         }

//         if ROW!(sq) >= 1 && COL!(sq) >= 2 {
//             bb.set((sq as i32 + Direction::SW_W as i32) as usize);
//         }

//         if ROW!(sq) >= 2 && COL!(sq) >= 1 {
//             bb.set((sq as i32 + Direction::SW_S as i32) as usize);
//         }
//         print!("0x{:x}, ", bb);
//         if sq % 4 == 3 {
//             println!();
//         }
//     }
//     println!("];");
// }

// fn gen_king() {
//     println!("pub const KING_ATTACKS: [BB; 64] = [");
//     for sq in 0..64 {
//         let mut bb = 0;
//         if ROW!(sq) > 0 {
//             bb.set((sq as i32 + Direction::South as i32) as usize);
//         }
//         if ROW!(sq) < 7 {
//             bb.set((sq as i32 + Direction::North as i32) as usize);
//         }
//         if COL!(sq) > 0 {
//             bb.set((sq as i32 + Direction::West as i32) as usize);
//         }
//         if COL!(sq) < 7 {
//             bb.set((sq as i32 + Direction::East as i32) as usize);
//         }
//         if ROW!(sq) > 0 && COL!(sq) > 0 {
//             bb.set((sq as i32 + Direction::Southwest as i32) as usize);
//         }
//         if ROW!(sq) > 0 && COL!(sq) < 7 {
//             bb.set((sq as i32 + Direction::Southeast as i32) as usize);
//         }
//         if ROW!(sq) < 7 && COL!(sq) > 0 {
//             bb.set((sq as i32 + Direction::Northwest as i32) as usize);
//         }
//         if ROW!(sq) < 7 && COL!(sq) < 7 {
//             bb.set((sq as i32 + Direction::Northeast as i32) as usize);
//         }
//         print!("0x{:x}, ", bb);
//         if sq % 4 == 3 {
//             println!();
//         }
//     }
//     println!("];");
// }
