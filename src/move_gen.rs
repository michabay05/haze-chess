use crate::attack::AttackInfo;
use crate::bb::{BBUtil, BB};
use crate::board::Board;
use crate::consts::{Direction, Piece, PieceColor, PieceType, Rank, Sq, MASK_RANK};
use crate::fen;
use crate::moves::{Move, MoveFlag, MoveUtil};

pub type MoveList = Vec<Move>;

trait MoveListUtil {
    fn print(&self);
}

impl MoveListUtil for MoveList {
    fn print(&self) {
        println!("     Source   |   Target  |     Flag");
        println!(" ---------------------------------------");
        for el in self {
            println!(
                "        {}    |     {}    |     {:?}",
                el.source(),
                el.target(),
                el.flag()
            );
        }
        println!("\n    Total number of moves: {}", self.len());
    }
}

pub fn generate_legal_moves(
    board: &mut Board,
    attack_info: &AttackInfo,
    side: PieceColor,
    list: &mut Vec<Move>,
) {
    assert!(side != PieceColor::Both);
    let opp = side.opposite();

    let us_bb = board.pos.units(side);
    let them_bb = board.pos.units(opp);
    let all_bb = board.pos.units(PieceColor::Both);

    let our_king =
        Sq::from_num(board.pos.bitboards[Piece::from_type(side, PieceType::King) as usize].lsb());
    let their_king =
        Sq::from_num(board.pos.bitboards[Piece::from_type(opp, PieceType::King) as usize].lsb());

    let our_diag_sliders = board.diagonal_sliders(side);
    let their_diag_sliders = board.diagonal_sliders(opp);
    let our_ortho_sliders = board.orthogonal_sliders(side);
    let their_ortho_sliders = board.orthogonal_sliders(opp);

    // Bitboards just for temporary storage
    let mut b1 = 0;
    let mut b2 = 0;
    let mut b3 = 0;

    let (rel_north, rel_south, rel_nw, rel_ne) = if side == PieceColor::Light {
        (
            Direction::North,
            Direction::South,
            Direction::Northwest,
            Direction::Northeast,
        )
    } else {
        (
            Direction::South,
            Direction::North,
            Direction::Southeast,
            Direction::Southwest,
        )
    };

    // Squares the king can't go to
    let mut danger: BB = 0;
    let their_pawns = board.pos.bitboards[Piece::from_type(opp, PieceType::Pawn) as usize];

    // Get all squares attacked by the opponent's pawns
    danger |= attack_info.get_all_pawn_attacks(opp, their_pawns) | attack_info.get_king_attack(their_king);

    // Get all squares attacked by the opponent's knights
    b1 = board.pos.bitboards[Piece::from_type(opp, PieceType::Knight) as usize];
    while b1 != 0 {
        let sq = b1.pop_lsb();
        danger |= attack_info.get_knight_attack(Sq::from_num(sq));
    }

    // Get all squares attacked by the opponent's diagonal sliders (bishop or queen)
    b1 = their_diag_sliders;
    while b1 != 0 {
        let sq = Sq::from_num(b1.pop_lsb());
        danger |= attack_info.get_bishop_attack(sq, all_bb ^ BB::from_sq(our_king));
    }

    // Get all squares attacked by the opponent's diagonal sliders (rook or queen)
    b1 = their_ortho_sliders;
    while b1 != 0 {
        let sq = Sq::from_num(b1.pop_lsb());
        danger |= attack_info.get_rook_attack(sq, all_bb ^ BB::from_sq(our_king));
    }

    // King moves
    b1 = attack_info.get_king_attack(our_king) & !(us_bb | danger);
    add_all_moves(MoveFlag::Quiet, our_king, b1 & !them_bb, list);
    add_all_moves(MoveFlag::Capture, our_king, b1 & them_bb, list);

    let mut capture_mask: BB = 0;
    let mut quiet_mask: BB = 0;
    let mut sq: Sq;

    // Are the opponent's knights able to check our king?
    let mut checkers: BB = attack_info.get_knight_attack(our_king)
        & board.pos.bitboards[Piece::from_type(opp, PieceType::Knight) as usize];
    // Are there any pawns that can check our king?
    checkers |= attack_info.get_pawn_attack(side, our_king)
        & board.pos.bitboards[Piece::from_type(opp, PieceType::Pawn) as usize];
    // NOTE: Pawns and knights have to be handled separately because if there were to check the king
    // the king would have to either move or capture the piece as it can't block the check

    // Does that opponent have any rooks or queens that can check our king?
    let mut candidates: BB = attack_info.get_rook_attack(our_king, them_bb) & their_ortho_sliders;
    // Does that opponent have any bishops or queens that can check our king?
    candidates |= attack_info.get_bishop_attack(our_king, them_bb) & their_diag_sliders;

    let mut pinned = 0;

    while candidates != 0 {
        let sq = candidates.pop_lsb();
        b1 = attack_info.squares_between[our_king as usize][sq] & us_bb;

        if b1 == 0 {
            // No, our piece between king and slider: check
            // i.e, there are no pieces of ours that are between the king and the checker
            checkers ^= BB::from_sq(Sq::from_num(sq));
        } else if (b1 & (b1 - 1)) == 0 {
            // Only one of our piece between king and slider: pinned
            pinned ^= b1;
        }
    }
    let not_pinned = !pinned;

    match checkers.count_ones() {
        // Double check: the king has to be moved
        2 => return,
        1 => {
            // Single check: Move, capture, or block
            let checker_sq = Sq::from_num(checkers.lsb());
            // TODO: handle `.unwrap()`
            let piece = board.pos.mailbox[checker_sq as usize].unwrap();
            // TODO: if possible, replace this if-else with a match
            if piece == Piece::from_type(opp, PieceType::Pawn) {
                if let Some(ep) = board.enpassant() {
                    if checkers == BB::from_sq(ep).shift(rel_south) {
                        b1 = attack_info.get_pawn_attack(opp, ep)
                            & board.pos.bitboards[Piece::from_type(side, PieceType::Pawn) as usize]
                            & not_pinned;
                        while b1 > 0 {
                            let sq = b1.pop_lsb();
                            list.push(Move::encode(Sq::from_num(sq), ep, MoveFlag::Enpassant));
                        }
                    }
                }

                // If checker is a pawn, then only a move or capture is allowed (can't block)
                // TODO: check if the implementation of `attackers_from()` function is correct
                b1 = attackers_from(board, attack_info, side, checker_sq, all_bb) & not_pinned;
                while b1 > 0 {
                    let sq = b1.pop_lsb();
                    list.push(Move::encode(
                        Sq::from_num(sq),
                        checker_sq,
                        MoveFlag::Capture,
                    ));
                }
                return;
            } else if piece == Piece::from_type(opp, PieceType::Knight) {
                // if checker is a knight, then only a move or capture is allowed (can't block)
                b1 = attackers_from(board, attack_info, side, checker_sq, all_bb) & not_pinned;
                while b1 > 0 {
                    let psq = b1.pop_lsb();
                    if let Some(p) = board.pos.mailbox[psq] {
                        let rank = Rank::Seven.relative(side) as usize;
                        let psq_sq = Sq::from_num(psq);
                        if p.piece_type() == PieceType::Pawn && ((BB::from_sq(psq_sq) & MASK_RANK[rank]) != 0) {
                            list.push(Move::encode(psq_sq, checker_sq, MoveFlag::PromCapQueen));
                            list.push(Move::encode(psq_sq, checker_sq, MoveFlag::PromCapRook));
                            list.push(Move::encode(psq_sq, checker_sq, MoveFlag::PromCapBishop));
                            list.push(Move::encode(psq_sq, checker_sq, MoveFlag::PromCapKnight));
                        } else {
                            list.push(Move::encode(psq_sq, checker_sq, MoveFlag::Capture));
                        }
                    }
                }
                return;
            } else {
                capture_mask = checkers;
                quiet_mask = attack_info.squares_between[our_king as usize][checker_sq as usize];
            }
        }
        _ => {
            // No check: do anything

            // Anything can taken
            capture_mask = them_bb;
            // or a quiet move to an empty square can be played
            quiet_mask = !all_bb;

            if let Some(ep) = board.enpassant() {
                b2 = attack_info.get_pawn_attack(opp, ep)
                    & board.pos.bitboards[Piece::from_type(side, PieceType::Pawn) as usize];
                b1 = b2 & not_pinned;

                while b1 > 0 {
                    let sq = b1.pop_lsb();
                    let blockers =
                        all_bb ^ BB::from_sq(Sq::from_num(sq)) ^ BB::from_sq(ep).shift(rel_south);
                    let mask = MASK_RANK[our_king.rank() as usize] & their_ortho_sliders;
                    if attack_info.sliding_attack(Sq::from_num(sq), blockers, mask) & their_ortho_sliders == 0 {
                        list.push(Move::encode(Sq::from_num(sq), ep, MoveFlag::Enpassant));
                    }
                }

                // Diagonal pin? Ok
                b1 = b2 & pinned & attack_info.line_of[ep as usize][our_king as usize];
                if b1 != 0 {
                    let sq = b1.lsb();
                    list.push(Move::encode(Sq::from_num(sq), ep, MoveFlag::Enpassant));
                }
            }

            // Castling
            // Castle is only allowed if:
            // 1. The king and the rook have both not moved
            // 2. No piece is attacking between the the rook and the king
            // 3. The king is not in check
            let entry = board.history[board.game_ply].entry;
            if ((entry & fen::get_oo_mask(side))
                | ((all_bb | danger) & fen::get_oo_blocker_mask(side)))
                == 0
            {
                if side == PieceColor::Light {
                    list.push(Move::encode(Sq::E1, Sq::G1, MoveFlag::KingSideCastling));
                } else {
                    list.push(Move::encode(Sq::E8, Sq::G8, MoveFlag::KingSideCastling));
                }
            }
            if ((entry & fen::get_ooo_mask(side))
                | ((all_bb | danger & !fen::ignore_ooo_danger(side))
                    & fen::get_ooo_blocker_mask(side)))
                == 0
            {
                if side == PieceColor::Light {
                    list.push(Move::encode(Sq::E1, Sq::C1, MoveFlag::QueenSideCastling));
                } else {
                    list.push(Move::encode(Sq::E8, Sq::C8, MoveFlag::QueenSideCastling));
                }
            }

            // pinned rook, bishop, or queen
            b1 = !(not_pinned
                | board.pos.bitboards[Piece::from_type(side, PieceType::Knight) as usize]);
            while b1 > 0 {
                let sq = Sq::from_num(b1.pop_lsb());

                // Only include moves that align with the king
                if let Some(p) = board.pos.mailbox[sq as usize] {
                    b2 = attack_info.get_attack(side, p.piece_type(), sq, all_bb) & attack_info.line_of[our_king as usize][sq as usize];
                    add_all_moves(MoveFlag::Quiet, sq, b2 & quiet_mask, list);
                    add_all_moves(MoveFlag::Capture, sq, b2 & capture_mask, list);
                }
            }

            b1 = !not_pinned
                & board.pos.bitboards[Piece::from_type(side, PieceType::Knight) as usize];
            while b1 > 0 {
                let sq = Sq::from_num(b1.pop_lsb());

                if sq.rank() == Rank::Seven.relative(side) {
                    // Quiet promotions are not possible here
                    b2 = attack_info.get_pawn_attack(side, sq)
                        & capture_mask
                        & attack_info.line_of[our_king as usize][sq as usize];
                    // TODO: confirm if PROMOTION_CAPTURES implies all 4 variations of promotion captures
                    add_all_moves(MoveFlag::PromCapQueen, sq, b2, list);
                    add_all_moves(MoveFlag::PromCapRook, sq, b2, list);
                    add_all_moves(MoveFlag::PromCapBishop, sq, b2, list);
                    add_all_moves(MoveFlag::PromCapKnight, sq, b2, list);
                } else {
                    b2 = attack_info.get_pawn_attack(side, sq)
                        & them_bb
                        & attack_info.line_of[sq as usize][our_king as usize];
                    add_all_moves(MoveFlag::Capture, sq, b2, list);

                    // Single pawn pushes
                    b2 = BB::from_sq(sq).shift(rel_north)
                        & !all_bb
                        & attack_info.line_of[our_king as usize][sq as usize];
                    // Double pawn pushes
                    b3 = (b2 & MASK_RANK[Rank::Three.relative(side) as usize]).shift(rel_north)
                        & !all_bb
                        & attack_info.line_of[our_king as usize][sq as usize];

                    add_all_moves(MoveFlag::Quiet, sq, b2, list);
                    add_all_moves(MoveFlag::DoublePush, sq, b3, list);
                }
            }
        }
    }

    // Non-pinned knight moves
    b1 = board.pos.bitboards[Piece::from_type(side, PieceType::Knight) as usize] & not_pinned;
    while b1 > 0 {
        let sq = Sq::from_num(b1.pop_lsb());
        b2 = attack_info.get_attack(side, PieceType::Knight, sq, all_bb);
        add_all_moves(MoveFlag::Quiet, sq, b2 & quiet_mask, list);
        add_all_moves(MoveFlag::Capture, sq, b2 & capture_mask, list);
    }

    // Non-pinned diagonal moves
    b1 = our_diag_sliders & not_pinned;
    while b1 > 0 {
        let sq = Sq::from_num(b1.pop_lsb());
        b2 = attack_info.get_attack(side, PieceType::Bishop, sq, all_bb);
        add_all_moves(MoveFlag::Quiet, sq, b2 & quiet_mask, list);
        add_all_moves(MoveFlag::Capture, sq, b2 & capture_mask, list);
    }

    // Non-pinned orthogonal moves
    b1 = our_ortho_sliders & not_pinned;
    while b1 > 0 {
        let sq = Sq::from_num(b1.pop_lsb());
        b2 = attack_info.get_attack(side, PieceType::Rook, sq, all_bb);
        add_all_moves(MoveFlag::Quiet, sq, b2 & quiet_mask, list);
        add_all_moves(MoveFlag::Capture, sq, b2 & capture_mask, list);
    }

    b1 = board.pos.bitboards[Piece::from_type(side, PieceType::Pawn) as usize]
        & not_pinned
        & !MASK_RANK[Rank::Seven.relative(side) as usize];

    // Single pushes
    b2 = b1.shift(rel_north) & !all_bb;

    // Double pushes
    b3 = (b2 & MASK_RANK[Rank::Three.relative(side) as usize]).shift(rel_north) & quiet_mask;

    b2 &= quiet_mask;

    while b2 > 0 {
        let sq = Sq::from_num(b2.pop_lsb());
        list.push(Move::encode(sq.sub(rel_north), sq, MoveFlag::Quiet));
    }

    while b3 > 0 {
        let sq = Sq::from_num(b3.pop_lsb());
        list.push(Move::encode(
            sq.sub(rel_north).sub(rel_north),
            sq,
            MoveFlag::DoublePush,
        ));
    }

    // Pawn captures
    b2 = b1.shift(rel_nw) & capture_mask;
    b3 = b1.shift(rel_ne) & capture_mask;

    while b2 > 0 {
        let sq = Sq::from_num(b2.pop_lsb());
        list.push(Move::encode(sq.sub(rel_nw), sq, MoveFlag::Capture));
    }

    while b3 > 0 {
        let sq = Sq::from_num(b3.pop_lsb());
        list.push(Move::encode(sq.sub(rel_ne), sq, MoveFlag::Capture));
    }

    // Promotions
    b1 = board.pos.bitboards[Piece::from_type(side, PieceType::Pawn) as usize]
        & not_pinned
        & MASK_RANK[Rank::Seven.relative(side) as usize];
    if b1 != 0 {
        // Quiet promotions
        b2 = b1.shift(rel_north) & quiet_mask;
        while b2 > 0 {
            let sq = Sq::from_num(b2.pop_lsb());

            list.push(Move::encode(sq.sub(rel_north), sq, MoveFlag::PromQueen));
            list.push(Move::encode(sq.sub(rel_north), sq, MoveFlag::PromRook));
            list.push(Move::encode(sq.sub(rel_north), sq, MoveFlag::PromBishop));
            list.push(Move::encode(sq.sub(rel_north), sq, MoveFlag::PromKnight));
        }

        // Promotion captures
        b2 = b1.shift(rel_nw) & capture_mask;
        b3 = b1.shift(rel_ne) & capture_mask;

        while b2 > 0 {
            let sq = Sq::from_num(b2.pop_lsb());

            list.push(Move::encode(sq.sub(rel_nw), sq, MoveFlag::PromCapQueen));
            list.push(Move::encode(sq.sub(rel_nw), sq, MoveFlag::PromCapRook));
            list.push(Move::encode(sq.sub(rel_nw), sq, MoveFlag::PromCapBishop));
            list.push(Move::encode(sq.sub(rel_nw), sq, MoveFlag::PromCapKnight));
        }

        while b3 > 0 {
            let sq = Sq::from_num(b3.pop_lsb());

            list.push(Move::encode(sq.sub(rel_ne), sq, MoveFlag::PromCapQueen));
            list.push(Move::encode(sq.sub(rel_ne), sq, MoveFlag::PromCapRook));
            list.push(Move::encode(sq.sub(rel_ne), sq, MoveFlag::PromCapBishop));
            list.push(Move::encode(sq.sub(rel_ne), sq, MoveFlag::PromCapKnight));
        }
    }

    // Finish!
}

#[inline(always)]
fn attackers_from(
    board: &Board,
    attack_info: &AttackInfo,
    side: PieceColor,
    sq: Sq,
    blocker_board: BB,
) -> BB {
    let p: BB;
    let n: BB;
    let b: BB;
    let r: BB;
    if side == PieceColor::Light {
        p = attack_info.get_pawn_attack(side.opposite(), sq) & board.pos.bitboards[Piece::LP as usize];
        n = attack_info.get_knight_attack(sq) & board.pos.bitboards[Piece::LN as usize];
    } else {
        p = attack_info.get_pawn_attack(side.opposite(), sq) & board.pos.bitboards[Piece::DP as usize];
        n = attack_info.get_knight_attack(sq) & board.pos.bitboards[Piece::DN as usize];
    }
    b = attack_info.get_bishop_attack(sq, blocker_board) & board.diagonal_sliders(side);
    r = attack_info.get_rook_attack(sq, blocker_board) & board.orthogonal_sliders(side);
    p | n | b | r
}

fn add_all_moves(flag: MoveFlag, source: Sq, mut to_bb: BB, ml: &mut Vec<Move>) {
    while to_bb != 0 {
        let target = to_bb.pop_lsb();
        ml.push(Move::encode(source, Sq::from_num(target), flag));
    }
}

#[cfg(test)]
mod tests {
    use super::{AttackInfo, Board, MoveList, MoveListUtil, PieceColor};

    #[test]
    fn move_gen_test() {
        let mut attack_info = AttackInfo::new();
        attack_info.init();
        let mut b = Board::new();
        let mut list = MoveList::new();

        use PieceColor::{Dark, Light};
        let arr = [
            // (8, Light, "8/2k5/8/8/8/8/2K5/8 w - - 0 1"),
            // (37, Light, "8/2k5/2qb4/8/8/P2Q4/KP3B2/8 w - - 0 1"),
            (1, Light, "8/5pb1/2bk2p1/4p1N1/3N4/4P3/1K3R2/8 w - - 0 1"),

            // (16, Dark, "8/2k5/3b4/8/8/8/2K5/8 b - - 0 1"),
            // // Double check test
            // (3, Dark, "8/1b6/3kn1p1/5r2/8/B2QK3/7p/8 b - - 0 1"),
            // // Single check test
            // (8, Dark, "8/1b6/3kn1p1/5r2/8/3QK3/7p/8 w - - 0 1")
            // // Quiet and capture promotion test
            // (43, Dark, "8/1b6/3kn1p1/5r2/8/2Q1K3/7p/6N1 b - - 0 1")
        ];
        for (expected, side, fen) in arr {
            b.set_fen(fen);
            list.clear();
            super::generate_legal_moves(&mut b, &attack_info, side, &mut list);
            if list.len() != expected {
                println!("FEN: '{}'", fen);
                b.display();
                println!("----------------------------------------");
                list.print();
                assert!(false);
            }
        }
    }
}
