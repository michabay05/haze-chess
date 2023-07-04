use crate::{board::Board, search};

#[derive(Copy, Clone, Default, PartialEq)]
pub enum TTFlag {
    #[default]
    Exact,
    Alpha,
    Beta,
}

#[derive(Copy, Clone, Default)]
pub struct TT {
    pub key: u64,
    pub lock: u64,
    pub score: i32,
    pub depth: u32,
    pub flag: TTFlag,
}

const HASH_MB: usize = 10;
const HASH_SIZE: usize = 0x100000 * HASH_MB;
const HASH_ENTRIES: usize = HASH_SIZE / std::mem::size_of::<TT>();

pub struct HashTT {
    table: Vec<TT>,
}

fn get_tt_ind(key: u64) -> usize {
    key as usize % HASH_ENTRIES
}

impl HashTT {
    pub fn new() -> Self {
        let mut this = Self {
            table: Vec::new(),
        };
        // Initialize the vec with all the values
        for _ in 0..HASH_ENTRIES {
            this.table.push(TT::default());
        }
        this
    }

    pub fn clear_table(&mut self) {
        // self.table.clear();
        for el in self.table.iter_mut() {
            *el = TT::default();
        }
    }

    pub fn read_entry(
        &mut self,
        board: &Board,
        alpha: i32,
        beta: i32,
        depth: u32,
        ply: u32,
    ) -> Option<i32> {
        let entry = self.table.get_mut(get_tt_ind(board.state.key));
        if entry.is_none() {
            return None;
        }
        let entry = entry.unwrap();
        if entry.key == board.state.key && entry.lock == board.state.lock {
            // Check if depth is the same
            if entry.depth >= depth {
                // Exact score from hash entry
                // Or extract mate distance from actual position
                if entry.score < -search::MATE_SCORE {
                    entry.score += ply as i32;
                }
                if entry.score > search::MATE_SCORE {
                    entry.score -= ply as i32;
                }

                // Match EXACT (PV node) score
                if entry.flag == TTFlag::Exact {
                    return Some(entry.score);
                }
                // Match ALPHA (fail-low node) score
                if (entry.flag == TTFlag::Alpha) && (entry.score <= alpha) {
                    return Some(alpha);
                }
                // Match BETA (fail-high node) score
                if (entry.flag == TTFlag::Beta) && (entry.score >= beta) {
                    return Some(beta);
                }
            }
        }
        return None;
    }

    pub fn write_entry(
        &mut self,
        board: &Board,
        depth: u32,
        mut score: i32,
        flag: TTFlag,
        ply: u32,
    ) {
        let ind = get_tt_ind(board.state.key);
        // Store mate score independent from the actual path
        if score < -search::MATE_SCORE {
            score -= ply as i32;
        }
        if score > search::MATE_SCORE {
            score += ply as i32;
        }

        // Write entry into hash table
        self.table[ind].key = board.state.key;
        self.table[ind].lock = board.state.lock;
        self.table[ind].score = score;
        self.table[ind].depth = depth;
        self.table[ind].flag = flag
    }
}
