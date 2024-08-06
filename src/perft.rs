use crate::attack::AttackInfo;
use crate::board::Board;
use crate::consts::PieceColor;
use crate::move_gen;
use crate::moves::{Move, MoveFlag, MoveUtil};

#[derive(Debug, Default)]
pub struct PerftInfo {
    pub nodes: u32,
    // Move types
    captures: u32,
    enpassant: u32,
    promotions: u32,
    castling: u32
}

impl PerftInfo {
    fn add_acc(&mut self, other: Self) {
        self.nodes += other.nodes;
        self.captures += other.captures;
        self.enpassant += other.enpassant;
        self.promotions += other.promotions;
        self.castling += other.castling;
    }

    fn identify_move(&mut self, mv: Move) {
        match mv.flag() {
            MoveFlag::Enpassant => self.enpassant += 1,
            MoveFlag::KingSideCastling | MoveFlag::QueenSideCastling => self.castling += 1,
            _ => {
                if mv.is_capture() {
                    self.captures += 1;
                }
                if mv.is_promotion() {
                    self.promotions += 1;
                }
            }
        }
    }
}

fn helper(
    board: &mut Board,
    side: PieceColor,
    attack_info: &AttackInfo,
    depth: usize,
    info: &mut PerftInfo,
) {
    if depth == 0 {
        info.nodes += 1;
        return;
    }

    let mut ml = Vec::<Move>::new();
    move_gen::generate_legal_moves(board, attack_info, side, &mut ml);

    // let mut clone;
    for mv in &ml {
        // clone = board.clone();
        board.play_move(side, *mv);

        info.identify_move(*mv);
        helper(board, side.opposite(), attack_info, depth - 1, info);
        board.undo_move(side, *mv);
        // *board = clone;
    }
}

pub fn driver(
    board: &mut Board,
    side: PieceColor,
    attack_info: &AttackInfo,
    depth: usize,
    debug: bool,
) -> PerftInfo {
    let mut ml = Vec::<Move>::new();
    move_gen::generate_legal_moves(board, attack_info, side, &mut ml);
    // ml.print();

    board.display();
    let mut info = PerftInfo::default();

    // let mut clone;
    for mv in &ml {
        let mut mv_info = PerftInfo::default();
        let move_str = mv.to_str();
        let move_str = if mv.is_promotion() {
            move_str
        } else {
            move_str[0..4].to_string()
        };

        board.play_move(side, *mv);
        mv_info.identify_move(*mv);
        helper(board, side.opposite(), attack_info, depth - 1, &mut mv_info);
        board.undo_move(side, *mv);

        if debug {
            eprintln!("{} {}", move_str, mv_info.nodes);
        }
        info.add_acc(mv_info);
    }

    info
}

#[cfg(test)]
mod tests {
    use crate::fen::FEN_POSITIONS;

    use super::{AttackInfo, Board};

    #[test]
    fn perft_checks() {
        let expecteds = [
            ("6k1/5b2/8/8/8/2Q5/3K4/8 w - - 0 1", 4, 1),
            // (FEN_POSITIONS[1], 5, 4865609),
            // (FEN_POSITIONS[2], 4, 4085603),
            // (FEN_POSITIONS[3], 5, 674624),
            // (FEN_POSITIONS[4], 5, 15833292),
            // (FEN_POSITIONS[5], 4, 2103487),
        ];

        let mut attack_info = AttackInfo::new();
        attack_info.init();
        let mut board = Board::new();
        for (fen, depth, expected_nodes) in expecteds {
            board.set_fen(fen);
            let side = board.state.side;
            let p_info = super::driver(&mut board, side, &attack_info, depth, true);
            eprintln!("{:#?}", p_info);
            // assert_eq!(nodes, expected_nodes);
            assert!(false);
        }
    }
}
