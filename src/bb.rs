use crate::consts::{Direction, File, Sq, MASK_FILE};
use crate::SQ;

pub type BB = u64;

pub trait BBUtil {
    fn from_sq(sq: Sq) -> Self;
    fn set(&mut self, ind: usize);
    fn get(&self, ind: usize) -> bool;
    fn pop(&mut self, ind: usize);
    fn toggle(&self, ind: usize) -> Self;
    fn lsb(&self) -> usize;
    fn pop_lsb(&mut self) -> usize;
    fn shift(&self, dir: Direction) -> Self;
    #[allow(dead_code)]
    fn print(&self);
}

impl BBUtil for BB {
    #[inline(always)]
    fn from_sq(sq: Sq) -> Self {
        1 << (sq as usize)
    }

    #[inline(always)]
    fn set(&mut self, ind: usize) {
        *self |= 1 << ind;
    }

    #[inline(always)]
    fn get(&self, ind: usize) -> bool {
        *self & (1 << ind) > 0
    }

    #[inline(always)]
    fn pop(&mut self, ind: usize) {
        *self &= !(1 << ind);
    }

    #[inline(always)]
    fn toggle(&self, ind: usize) -> Self {
        *self ^ (1 << ind)
    }

    #[inline(always)]
    fn lsb(&self) -> usize {
        self.trailing_zeros() as usize
        // let num = *self ^ (*self - 1);
        // let count = num.count_ones() as usize;
        // if count == 0 {
        //     return 0;
        // }
        // count - 1
    }

    #[inline(always)]
    fn pop_lsb(&mut self) -> usize {
        let ind = self.lsb();
        self.pop(ind);
        ind
    }

    #[inline(always)]
    fn shift(&self, dir: Direction) -> Self {
        match dir {
            Direction::North => *self << 8,
            Direction::South => *self >> 8,
            Direction::NorthNorth => *self << 16,
            Direction::SouthSouth => *self >> 16,
            Direction::East => (*self & !MASK_FILE[File::H as usize]) << 1,
            Direction::West => (*self & !MASK_FILE[File::A as usize]) >> 1,
            Direction::Northeast => (*self & !MASK_FILE[File::H as usize]) << 9,
            Direction::Northwest => (*self & !MASK_FILE[File::A as usize]) << 7,
            Direction::Southeast => (*self & !MASK_FILE[File::H as usize]) >> 7,
            Direction::Southwest => (*self & !MASK_FILE[File::A as usize]) >> 9,
            _ => {
                eprintln!("Unhandled bitboard shifting direction");
                *self
            }
        }
    }

    fn print(&self) {
        for r in (0..8).rev() {
            print!(" {} |", r + 1);
            for f in 0..8 {
                print!(" {}", if self.get(SQ!(r, f)) { '1' } else { '.' });
            }
            println!();
        }
        println!("     - - - - - - - -");
        println!("     a b c d e f g h");
    }
}

#[cfg(test)]
mod bb_tests {
    use crate::consts::{Direction, PieceColor, Sq};

    use super::{BBUtil, BB};

    #[test]
    fn test_shift() {
        let mut bb: BB;
        let mut expected: BB;

        use Direction::*;
        use PieceColor::*;
        use Sq::*;

        let arr = [
            (Light, E2, NorthNorth, E4),
            (Light, E7, SouthSouth, E5),
            (Light, E4, North, E5),
            (Light, E4, South, E3),
            (Light, E4, East, F4),
            (Light, E4, West, D4),
            (Light, H4, West, G4),
            (Light, H7, North, H8),
            (Light, C5, Northeast, D6),
            (Light, C5, Northwest, B6),
            (Light, C5, Southeast, D4),
            (Light, C5, Southwest, B4),
            (Dark, C7, NorthNorth, C5),
            (Dark, H2, SouthSouth, H4),
            (Dark, E5, North, E4),
            (Dark, E5, South, E6),
            (Dark, E5, East, D5),
            (Dark, E5, West, F5),
            (Dark, B4, East, A4),
            (Dark, H8, North, H7),
            (Dark, G3, Northeast, F2),
            (Dark, G3, Northwest, H2),
            (Dark, G3, Southeast, F4),
            (Dark, G3, Southwest, H4),
        ];

        for (side, sq, dir, shifted) in arr {
            bb = 0;
            expected = 0;
            bb.set(sq as usize);
            bb.shift(dir.relative(side));
            expected.set(shifted as usize);
            if bb != expected {
                eprintln!(
                    "[{:?}] {} shifted {:?} = {} (expected = {})",
                    side,
                    sq,
                    dir,
                    Sq::from_num(bb.lsb()),
                    Sq::from_num(expected.lsb())
                );
                bb.print();
                eprintln!("-----------------------");
                expected.print();
                assert!(false);
            }
        }
    }
}
