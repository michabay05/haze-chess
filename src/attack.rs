#![allow(unused_comparisons)]

use crate::bb::{BBUtil, BB};
use crate::consts::{self, Direction, PieceColor, PieceType, Sq};
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
    pub line_of: [[BB; 64]; 64],
}

impl AttackInfo {
    pub fn new() -> Self {
        Self {
            bishop_occ_mask: [0; 64],
            bishop_attack: vec![vec![0; 512]; 64],
            rook_occ_mask: [0; 64],
            rook_attack: vec![vec![0; 4096]; 64],
            squares_between: [[0; 64]; 64],
            line_of: [[0; 64]; 64],
        }
    }

    pub fn init(&mut self) {
        // NOTE: the function below is only used to generate the leaper pieces' constants
        // gen_leapers();
        gen_sliding(self, PieceType::Bishop);
        gen_sliding(self, PieceType::Rook);
        self.gen_squares_between();
        self.gen_line_of();
    }

    #[inline(always)]
    pub fn get_pawn_attack(&self, color: PieceColor, sq: Sq) -> BB {
        if color == PieceColor::Light {
            LIGHT_PAWN_ATTACKS[sq as usize]
        } else {
            DARK_PAWN_ATTACKS[sq as usize]
        }
    }

    #[inline(always)]
    pub fn get_all_pawn_attacks(&self, color: PieceColor, bb: BB) -> BB {
        let mut all_attacks = 0;
        if color == PieceColor::Light {
            all_attacks |= bb.shift(Direction::Northwest);
            all_attacks |= bb.shift(Direction::Northeast);
        } else {
            all_attacks |= bb.shift(Direction::Southwest);
            all_attacks |= bb.shift(Direction::Southeast);
        }
        all_attacks
    }

    #[inline(always)]
    pub fn get_knight_attack(&self, sq: Sq) -> BB {
        KNIGHT_ATTACKS[sq as usize]
    }

    #[inline(always)]
    pub fn get_king_attack(&self, sq: Sq) -> BB {
        KING_ATTACKS[sq as usize]
    }

    pub fn get_attack(&self, color: PieceColor, pt: PieceType, sq: Sq, blocker_board: BB) -> BB {
        match pt {
            // Sliders
            PieceType::Rook => self.get_rook_attack(sq, blocker_board),
            PieceType::Bishop => self.get_bishop_attack(sq, blocker_board),
            PieceType::Queen => {
                self.get_bishop_attack(sq, blocker_board) | self.get_rook_attack(sq, blocker_board)
            }
            // Leapers
            PieceType::Pawn => {
                if color == PieceColor::Light {
                    LIGHT_PAWN_ATTACKS[sq as usize]
                } else {
                    DARK_PAWN_ATTACKS[sq as usize]
                }
            }
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

    // Hyperbola Quintessence Algorithm
    #[inline(always)]
    pub fn sliding_attack(&self, square: Sq, blocker_board: BB, mask: BB) -> BB {
        let sq = square as usize;
        let mut sq_bb = BB::from_sq(square);
        let first_line = (mask & blocker_board).wrapping_sub(sq_bb.wrapping_mul(2));
        let second_line = reverse_bitboard(
            reverse_bitboard(mask & blocker_board).wrapping_sub(
                reverse_bitboard(sq_bb).wrapping_mul(2)
            )
        );
        (first_line ^ second_line) & mask
    }

    #[inline(always)]
    fn get_rook_attacks_for_init(&self, sq: Sq, blocker: BB) -> BB {
        self.sliding_attack(sq, blocker, consts::MASK_FILE[sq.file() as usize])
            | self.sliding_attack(sq, blocker, consts::MASK_RANK[sq.rank() as usize])
    }

    #[inline(always)]
    fn get_bishop_attacks_for_init(&self, sq: Sq, blocker: BB) -> BB {
        self.sliding_attack(sq, blocker, consts::MASK_DIAGONAL[Sq::diagonal(sq as i32) as usize])
        | self.sliding_attack(sq, blocker, consts::MASK_ANTI_DIAGONAL[Sq::anti_diagonal(sq as i32) as usize])
    }

    pub fn gen_line_of(&mut self) {
        let mut sq1 = Sq::A1 as usize;
        while sq1 < Sq::H8 as usize {
            let mut sq2 = Sq::A1 as usize;

            while sq2 < Sq::H8 as usize {
                let mut sqs: BB = 0;
                sqs.set(sq1);
                sqs.set(sq2);

                if COL!(sq1) == COL!(sq2) || ROW!(sq1) == ROW!(sq2) {
                    self.line_of[sq1][sq2] = self.get_rook_attacks_for_init(Sq::from_num(sq1), 0)
                        & self.get_rook_attacks_for_init(Sq::from_num(sq2), 0)
                        | BB::from_sq(Sq::from_num(sq1))
                        | BB::from_sq(Sq::from_num(sq2));
                } else if Sq::diagonal(sq1 as i32) == Sq::diagonal(sq2 as i32)
                    || Sq::anti_diagonal(sq1 as i32) == Sq::anti_diagonal(sq2 as i32)
                {
                    self.line_of[sq1][sq2] = self.get_bishop_attacks_for_init(Sq::from_num(sq1), 0)
                        & self.get_bishop_attacks_for_init(Sq::from_num(sq2), 0)
                        | BB::from_sq(Sq::from_num(sq1))
                        | BB::from_sq(Sq::from_num(sq2));
                } else {
                    self.line_of[sq1][sq2] = 0;
                }

                sq2 += 1;
            }

            sq1 += 1;
        }
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
                    self.squares_between[sq1][sq2] = self.get_rook_attacks_for_init(Sq::from_num(sq1), sqs)
                        & self.get_rook_attacks_for_init(Sq::from_num(sq2), sqs);
                } else if Sq::diagonal(sq1 as i32) == Sq::diagonal(sq2 as i32)
                    || Sq::anti_diagonal(sq1 as i32) == Sq::anti_diagonal(sq2 as i32)
                {
                    self.squares_between[sq1][sq2] = self.get_bishop_attacks_for_init(Sq::from_num(sq1), sqs)
                        & self.get_bishop_attacks_for_init(Sq::from_num(sq2), sqs);
                } else {
                    self.squares_between[sq1][sq2] = 0;
                }
                    if sq1 == Sq::F8 as usize && sq2 == Sq::H8 as usize {
                        eprintln!("You ought to be over here!");
                        assert!(false);
                    }

                sq2 += 1;
            }

            sq1 += 1;
        }
    }
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

#[inline(always)]
fn reverse_bitboard(bb: BB) -> BB {
    let mut b = bb;
    b = (b & 0x5555555555555555) << 1 | ((b >> 1) & 0x5555555555555555);
    b = (b & 0x3333333333333333) << 2 | ((b >> 2) & 0x3333333333333333);
    b = (b & 0x0f0f0f0f0f0f0f0f) << 4 | ((b >> 4) & 0x0f0f0f0f0f0f0f0f);
    b = (b & 0x00ff00ff00ff00ff) << 8 | ((b >> 8) & 0x00ff00ff00ff00ff);

    return (b << 48) | ((b & 0xffff0000) << 16) | ((b >> 16) & 0xffff0000) | (b >> 48);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn squares_between() {
        let mut attack_info = AttackInfo::new();
        attack_info.init();

        use Sq::*;
        let arr = [
            //(B4, F4, 0x1c000000),
            //(E3, E7, 0x101010000000),
            //(B2, G7, 0x201008040000),
            //(B7, G2, 0x40810200000),
            //(A1, G4, 0),
            (F8, H8, 0x4000000000000000),
        ];

        for (sq1, sq2, expected) in arr {
            let res = attack_info.squares_between[sq1 as usize][sq2 as usize];
            if res != expected {
                println!("FAILED: sq1({}) sq2({})", sq1, sq2);
                println!("result:");
                res.print();
                println!("expected:");
                expected.print();
                assert!(false);
            }
        }
    }

    #[test]
    fn line_of() {
        let mut attack_info = AttackInfo::new();
        attack_info.init();

        use Sq::*;
        let arr = [
            (E3, E7, 0x1010101010101010),
            (B4, F4, 0xff000000),
            (B2, G7, 0x8040201008040201),
            (B7, G2, 0x102040810204080),
            (A1, G4, 0),
        ];

        for (sq1, sq2, expected) in arr {
            let res = attack_info.line_of[sq1 as usize][sq2 as usize];
            if res != expected {
                println!("FAILED: sq1({}) sq2({})", sq1, sq2);
                println!("result:");
                res.print();
                println!("expected:");
                expected.print();
                assert!(false);
            }
        }
    }

    #[test]
    fn pawn_attacks() {
        let mut attack_info = AttackInfo::new();
        attack_info.init();

        assert_eq!(attack_info.get_pawn_attack(PieceColor::Dark, Sq::E5), 0x28000000);
        assert_eq!(attack_info.get_pawn_attack(PieceColor::Light, Sq::E4), 0x2800000000);
        assert_eq!(attack_info.get_pawn_attack(PieceColor::Light, Sq::A5), 0x20000000000);

        assert_eq!(attack_info.get_all_pawn_attacks(PieceColor::Light, 0x2800000200400), 0x5400000500a0000);
        assert_eq!(attack_info.get_all_pawn_attacks(PieceColor::Dark, 0x2800000200400), 0x5400000500a);
    }
}
