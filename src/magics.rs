#![allow(dead_code)]

use crate::attack;
use crate::consts::{PieceType, Sq};

fn xor_random_u32(random_state: &mut u32) -> u32 {
    let mut number = *random_state;
    // XOR shift algorithm
    number ^= number << 13;
    number ^= number >> 17;
    number ^= number << 5;

    // Update random number state
    *random_state = number;

    number
}

const XOR_RANDOM_SEED: u32 = 1804289383;

fn xor_random_u64(random_state: &mut u32) -> u64 {
    let rand1 = (xor_random_u32(random_state) & 0xFFFF) as u64;
    let rand2 = (xor_random_u32(random_state) & 0xFFFF) as u64;
    let rand3 = (xor_random_u32(random_state) & 0xFFFF) as u64;
    let rand4 = (xor_random_u32(random_state) & 0xFFFF) as u64;
    rand1 | (rand2 << 16) | (rand3 << 32) | (rand4 << 48)
}

fn gen_random_magic(random_state: &mut u32) -> u64 {
    // RANDOM U64 NUMBER WITH A SEED
    xor_random_u64(random_state) & xor_random_u64(random_state) & xor_random_u64(random_state)

    // RANDOM U64 NUMBER WITHOUT A SEED
    // rand::random::<u64>() & rand::random::<u64>() & rand::random::<u64>()
}

#[allow(unused_assignments)]
fn find_magic_number(sq: usize, relevant_bits: u32, piece: PieceType) -> u64 {
    /* Needed for the random number generation - XOR shift algorithm */
    let mut random_state: u32 = XOR_RANDOM_SEED;

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
        magic_num = gen_random_magic(&mut random_state);
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
    0x40040822862081, 0x10201a0200411402, 0x81024288020c000,
    0x1404640080008810, 0x9004242000012008, 0x10a412020a04008,
    0x1000989008208000, 0x22010108410402, 0x2104840810014200,
    0x2150210810a080, 0x81080089061040, 0x2400a82040408008,
    0x240420005810, 0x4200022860180000, 0x4000090082504000,
    0x410402422220, 0x8140420c80200, 0x2038a4832080200,
    0xc108005006404048, 0x2208001041404049, 0xc014021880a01000,
    0x704200110101006, 0x2808c12013100, 0x6400411200444402,
    0x124240a1041400, 0x8802088010100088, 0x4020040408508,
    0x604080004006128, 0x8000848004002004, 0x4008020008405210,
    0x806000aa22902, 0x2200888682020080, 0x10c2105000400320,
    0x8018010800102208, 0x4000841102100044, 0x200800050104,
    0x160c030400280408, 0x820080320094405, 0x201e40100540101,
    0x8408248080002227, 0x8021004001040, 0x400455030034803,
    0x912020322180400, 0x8026013002800, 0xe000040810130200,
    0x2168500092000020, 0x2002304119002200, 0x1901020400400510,
    0x1000989008208000, 0x100840108020006, 0x3010020842088080,
    0x8000001b84044881, 0x8004404010510072, 0xc10801010000,
    0x4090808148101, 0x10201a0200411402, 0x22010108410402,
    0x410402422220, 0x41000114204d004, 0x4082000100840440,
    0x400c802211420200, 0x81800140828c8100, 0x2104840810014200,
    0x40040822862081,
];

#[rustfmt::skip]
pub const ROOK_MAGICS: [u64; 64] = [
    0x8a80104000800020, 0xc40100040082000, 0x100102001000840,
    0x1080041000080080, 0x4280240080020800, 0x4800a00211c0080,
    0x1080008001000200, 0x42000082c9020424, 0x2002081004200,
    0x2002081004200, 0x801000802000, 0x201001000082100,
    0xe41001005000800, 0x1022001008854200, 0x211000100020084,
    0x18801041000080, 0x80084000200040, 0x20a0024000500020,
    0x80410010200901, 0x2083090010002300, 0x808004000800,
    0x804008080040200, 0x8800040002100108, 0x20001208044,
    0x4020800080204000, 0x40008280200042, 0x820200204010,
    0x200100480080080, 0x300040080080080, 0x804008080040200,
    0x8000020400881001, 0x88808200204401, 0x6480042006400041,
    0x4080804000802000, 0x801000802000, 0x1518001000800882,
    0xe41001005000800, 0x1012001002000408, 0x140108804006201,
    0x2050882000054, 0x90080c000618011, 0xa0004000208080,
    0x22001080220043, 0x1012010050008, 0x40008008080,
    0x1100040002008080, 0x40100182040008, 0x800000648102000c,
    0x481248002c90100, 0x2002081004200, 0x400c802211420200,
    0x280200c10010100, 0x300040080080080, 0x1100040002008080,
    0x4000018802100400, 0x4310800100004080, 0x4024800508102041,
    0x88801100204001, 0x401080104200200a, 0x8010210408100101,
    0x9202002005881002, 0x8012004824011022, 0x2000011002080084,
    0x1010549228402,
];
