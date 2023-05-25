use std::time::Instant;

use crate::attack::AttackInfo;
use crate::board::Board;
use crate::move_gen::{self, MoveList};
use crate::moves::{self, MoveFlag};

fn driver(board: &mut Board, attack_info: &AttackInfo, depth: usize, node_count: &mut usize) {
    if depth == 0 {
	*node_count += 1;
	return;
    }
	
    let mut ml = MoveList::new();
    move_gen::generate(board, attack_info, &mut ml);
    let mut clone;
    for mv in &ml.moves {
	clone = board.clone();
	if !moves::make(board, attack_info, *mv, MoveFlag::AllMoves) {
	    continue;
	}
	driver(board, attack_info, depth - 1, node_count);
	*board = clone;
    }
}


pub fn test(board: &mut Board, attack_info: &AttackInfo, depth: usize) {
    println!("\n--------------- Performance Test (d = {depth}) ---------------");
    let mut total_nodes = 0;
    let mut ml = MoveList::new();
    move_gen::generate(board, attack_info, &mut ml);
    // Start timer
    let start = Instant::now();

    let mut clone;
    for mv in &ml.moves {
	clone = board.clone();
	if !moves::make(board, attack_info, *mv, MoveFlag::AllMoves) {
	    continue;
	}
	// Nodes searched so far
	let nodes_searched = total_nodes;
	driver(board, attack_info, depth - 1, &mut total_nodes);
	*board = clone;
	println!("     {}: {}", mv.to_string(), total_nodes - nodes_searched);
    }
    let end = Instant::now();
    println!("\n     Depth: {depth}");
    println!("     Nodes: {total_nodes}");
    println!("      Time: {}", end.duration_since(start).as_secs());
}
