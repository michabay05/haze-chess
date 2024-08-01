use crate::attack::AttackInfo;
use crate::board::Board;
use crate::move_gen::{self, MoveList};
use crate::moves::{self, MoveType, MoveUtil};

fn helper(
    board: &mut Board,
    attack_info: &AttackInfo,
    depth: usize,
    node_count: &mut usize,
) {
    if depth == 0 {
        *node_count += 1;
        return;
    }

    let mut ml = MoveList::new();
    move_gen::generate(board, attack_info, &mut ml);
    let mut clone;
    for mv in &ml.moves {
        clone = board.clone();
        if !moves::play_move(board, attack_info, *mv, MoveType::AllMoves) {
            continue;
        }
        helper(board, attack_info, depth - 1, node_count);
        *board = clone;
    }
}

pub fn driver(board: &mut Board, attack_info: &AttackInfo, depth: usize, debug: bool) -> usize {
    let mut total_nodes = 0;
    let mut ml = MoveList::new();
    move_gen::generate(board, attack_info, &mut ml);

    let mut clone;
    for mv in &ml.moves {
        clone = board.clone();
        if !moves::play_move(board, attack_info, *mv, MoveType::AllMoves) {
            continue;
        }
        // Nodes searched so far
        let nodes_searched = total_nodes;
        helper(
            board,
            attack_info,
            depth - 1,
            &mut total_nodes,
        );
        *board = clone;
        let move_str = mv.to_str();
        let move_str = if mv.promoted().is_some() {
            move_str
        } else {
            move_str[0..4].to_string()
        };
        if debug {
            println!("{} {}", move_str, total_nodes - nodes_searched);
        }
    }
    total_nodes
}

#[cfg(test)]
mod tests {
    use crate::fen::FEN_POSITIONS;

    use super::{AttackInfo, Board};

    #[test]
    fn perft_checks() {
        let expecteds = [
            (FEN_POSITIONS[1], 5, 4865609),
            (FEN_POSITIONS[2], 4, 4085603),
            (FEN_POSITIONS[3], 5, 674624),
            (FEN_POSITIONS[4], 5, 15833292),
            (FEN_POSITIONS[5], 4, 2103487),
        ];

        let mut attack_info = AttackInfo::new();
        attack_info.init();
        let mut board = Board::new();
        for (fen, depth, expected_nodes) in expecteds {
            board.set_fen(fen);
            let nodes = super::driver(&mut board, &attack_info, depth, true);
            assert_eq!(nodes, expected_nodes);
        }
    }
}
