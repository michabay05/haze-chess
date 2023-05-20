use crate::attack::{self, AttackInfo};
use crate::bb::BBUtil;
use crate::consts::{PieceColor, STR_COORDS};

pub struct Engine {
    pub attack: AttackInfo,
}

impl Engine {
    fn new() -> Self {
        let mut this = Self {
            attack: AttackInfo::new(),
        };
        this.init();
        this
    }

    fn init(&mut self) {
        attack::init(&mut self.attack);
    }
}

pub fn test() {
    crate::magics::init();
}

fn a() {
    let engine = Engine::new();
    for i in 0..64 {
        println!("{}\n---------------------", STR_COORDS[i]);
        engine.attack.king[i].print();
        println!();
    }
}
