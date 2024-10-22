#![allow(dead_code)]

use crate::attack;
use crate::consts::{PieceType, Sq};
use crate::prng::PRNG;

const XOR_RANDOM_SEED: u32 = 1804289383;

#[allow(unused_assignments)]
fn find_magic_number(sq: usize, relevant_bits: u32, piece: PieceType) -> u64 {
    /* Needed for the random number generation - XOR shift algorithm */
    let mut prng = PRNG::new(XOR_RANDOM_SEED as u128);

    let mut used_attacks = [0u64; 4096];
    let mut occupancies = [0u64; 4096];
    let mut attacks = [0u64; 4096];
    let mut magic_num = 0u64;
    let possible_occ = if piece == PieceType::Bishop {
        attack::gen_bishop_occ(sq)
    } else {
        attack::gen_rook_occ(sq)
    };
    let occ_indices = 1 << relevant_bits;
    for ind in 0..occ_indices {
        occupancies[ind] = attack::set_occ(ind, relevant_bits, possible_occ);
        attacks[ind] = if piece == PieceType::Bishop {
            attack::gen_bishop_attack(sq, occupancies[ind])
        } else {
            attack::gen_rook_attack(sq, occupancies[ind])
        };
    }

    for _ in 0..100_000_000 {
        magic_num = prng.sparse_rand64();
        let product = u128::from(possible_occ) * u128::from(magic_num);
        if (product & 0xFF00000000000000).count_ones() < 6 {
            continue;
        }
        used_attacks = [0u64; 4096];
        let mut count = 0;
        let mut fail = false;
        while !fail && count < occ_indices {
            let num = u128::from(occupancies[count]) * u128::from(magic_num);
            let magic_ind = (num as u64) >> (64 - relevant_bits);
            if used_attacks[magic_ind as usize] == 0 {
                used_attacks[magic_ind as usize] = attacks[count];
            } else if used_attacks[magic_ind as usize] != attacks[count] {
                fail = true;
            }
            count += 1;
        }
        if !fail {
            return magic_num;
        }
    }
    println!("Failed to find magic number on {}", Sq::from_num(sq));
    0
}

pub fn init() {
    println!("BISHOP: [");
    for sq in 0..64 {
        let relevant_bits = attack::BISHOP_RELEVANT_BITS[sq];
        println!(
            "    0x{:x},",
            find_magic_number(sq, relevant_bits, PieceType::Bishop)
        );
    }
    println!("];\n\nROOK: [");
    for sq in 0..64 {
        let relevant_bits = attack::ROOK_RELEVANT_BITS[sq];
        println!(
            "    0x{:x},",
            find_magic_number(sq, relevant_bits, PieceType::Rook)
        );
    }
    println!("]; ");
}

// ======================== MAGIC CONSTANTS ========================

#[rustfmt::skip]
pub const BISHOP_MAGICS: [u64; 64] = [
    0x220925001090011, 0x40212080d050028, 0x52082084108010a0,
    0x8820802a084000, 0xc04410a800c89000, 0x2020220402203,
    0x8202011059040000, 0x400200a4840c6080, 0x50802040408,
    0x800410104200822a, 0x4010e0a1004, 0x8820090401082042,
    0x60141028302000, 0x6100009010091000, 0x4884108a000,
    0x4884108a000, 0x808004110342080, 0x604009001220400,
    0x4048201000204150, 0x658000104110200, 0x2010402310200,
    0x42010020842001, 0x2000980048041000, 0x406111080101,
    0x4840210b11000, 0x2a08484860920480, 0x801048000c081010,
    0x4002004120080, 0x8400848044002000, 0x20810048090800,
    0x5021090504340102, 0x428200a848413, 0x210100880108200,
    0x82503000640110, 0x180404804104800, 0x209020080880082,
    0x6da8100400004102, 0x4001005100120100, 0x8110102004800,
    0x84089200422b00, 0x2101007000810, 0x6080841002000820,
    0x8081084848091000, 0x1004001044002021, 0x102020a000404,
    0x2181001008424, 0x20a002040111c051, 0x1031880109000142,
    0x8202011059040000, 0x1022209a10108130, 0x100000220110014a,
    0x200441002a080000, 0x320000420820005, 0x89200530208000,
    0x800410104200822a, 0x40212080d050028, 0x400200a4840c6080,
    0x4884108a000, 0xa088c80500884401, 0x404000010208800,
    0x8880010840104120, 0x1000220820080084, 0x50802040408,
    0x220925001090011,
];

#[rustfmt::skip]
pub const ROOK_MAGICS: [u64; 64] = [
    0x80002082544000, 0x2240001001200246, 0x4900114108200100,
    0x480042800300080, 0x1080080080f40002, 0x100020400010008,
    0xb0004260004a900, 0x8100008900034022, 0x80802040008008,
    0x10401000402000, 0x21004020010010, 0x52801002808804,
    0x1802800400080080, 0x1a01800400020180, 0x1404000801041002,
    0x501000100108c62, 0x2218000804001, 0x20c0404010002004,
    0x200808020001009, 0x1018808010000800, 0x818008001c02,
    0x1020808002000401, 0x40044001008211a, 0x20004318841,
    0x8040802380044000, 0x68b00040c0002004, 0x1000208200104200,
    0xc0080280100083, 0x80100100500, 0x1202040080800200,
    0x1005080400900102, 0x501000100108c62, 0x400804000800022,
    0x4540400082802000, 0x801004802006, 0x1080082801004,
    0x400800800800400, 0x2084000200800480, 0x2124704003008,
    0x86c184102000084, 0x4380002000404004, 0x4002814001070022,
    0x21004020010010, 0x40080010008080, 0x2008020004004040,
    0x2000408020010, 0x4080100201040008, 0x901006884020001,
    0x4180004000200040, 0x10244010810300, 0x8830700080200380,
    0x40080010008080, 0x2008020004004040, 0x1a01800400020180,
    0x5003422001100, 0x90010440a40600, 0x3000800010442901,
    0x8010802040001105, 0x1220004020140901, 0x1000822041001,
    0x102006008100c06, 0x89000204000801, 0x802000b008022104,
    0x4000004100802402,
];
