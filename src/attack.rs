use crate::{ROW, COL};
use crate::bb::{BB, BBUtil};
use crate::consts::{Direction, PieceColor, Sq};

pub struct AttackInfo {
    pub pawn: [[BB; 64]; 2],
}

impl AttackInfo {
    fn new() -> Self {
        Self {
            pawn: [[0; 64]; 2],
        }
    }
}

pub fn init() {
    let mut attack = AttackInfo::new();
    gen_leapers(&mut attack);
}

fn gen_leapers(attack: &mut AttackInfo) {
    for sq in 0..64 {
        gen_pawn(attack, sq, PieceColor::Light);
    }
}

fn gen_pawn(attack_info: &mut AttackInfo, sq: usize, color: PieceColor) {
    let bb = &mut attack_info.pawn[color as usize][sq];
    if color == PieceColor::Light {
        if ROW!(sq) > 0 && COL!(sq) > 0 {
            bb.set((sq as i32 + Direction::SE as i32) as usize); 
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
