use crate::attack::AttackInfo;
use crate::board::Board;
use crate::move_gen::{self, MoveList};
use crate::moves::{self, MoveType, MoveUtil};

fn driver(
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
        driver(board, attack_info, depth - 1, node_count);
        *board = clone;
    }
}

pub fn test(board: &mut Board, attack_info: &AttackInfo, depth: usize) {
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
        driver(
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
        println!("{} {}", move_str, total_nodes - nodes_searched);
    }
    println!("\nNodes: {total_nodes}");
}
