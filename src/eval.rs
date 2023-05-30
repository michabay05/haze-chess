use crate::attack::AttackInfo;
use crate::bb::{BBUtil, BB};
use crate::board::Position;
use crate::consts::{Piece, PieceColor, Sq};
use crate::eval_consts::*;
use crate::{COL, FLIP_SQ, ROW, SQ};

#[derive(Debug)]
enum Phase {
    Opening,
    Endgame,
    Middlegame,
}

pub struct EvalMasks {
    pub rank: [BB; 8],
    pub file: [BB; 8],
    pub isolated: [BB; 8],
    pub passed: [[BB; 64]; 2],
}

impl EvalMasks {
    pub fn new() -> Self {
        Self {
            rank: [0; 8],
            file: [0; 8],
            isolated: [0; 8],
            passed: [[0; 64]; 2],
        }
    }

    pub fn init(&mut self) {
        for i in 0..8i32 {
            self.rank[i as usize] |= set_file_and_rank(i, -1);
            self.file[i as usize] |= set_file_and_rank(-1, i);
        }

        for r in 0..8i32 {
            for f in 0..8i32 {
                let sq = SQ!(r, f) as usize;
                self.isolated[f as usize] |= set_file_and_rank(-1, f - 1);
                self.isolated[f as usize] |= set_file_and_rank(-1, f + 1);

                self.passed[PieceColor::Light as usize][sq] |= set_file_and_rank(-1, f - 1);
                self.passed[PieceColor::Light as usize][sq] |= set_file_and_rank(-1, f);
                self.passed[PieceColor::Light as usize][sq] |= set_file_and_rank(-1, f + 1);

                for i in 0..(8 - r) {
                    self.passed[PieceColor::Light as usize][sq] &= !self.rank[(7 - i) as usize];
                }

                self.passed[PieceColor::Dark as usize][sq] |= set_file_and_rank(-1, f - 1);
                self.passed[PieceColor::Dark as usize][sq] |= set_file_and_rank(-1, f);
                self.passed[PieceColor::Dark as usize][sq] |= set_file_and_rank(-1, f + 1);

                for i in 0..(r + 1) {
                    self.passed[PieceColor::Dark as usize][sq] &= !self.rank[i as usize];
                }
            }
        }
    }
}

fn set_file_and_rank(rank: i32, file: i32) -> BB {
    let mut mask = 0;
    for r in 0..8 {
        for f in 0..8 {
            let sq = SQ!(r, f);
            if (file != -1 && file == f) || (rank != -1 && rank == r) {
                mask.set(sq as usize);
            }
        }
    }
    mask
}

fn get_phase_score(pos: &Position) -> i32 {
    let mut white_piece_score = 0;
    let mut black_piece_score = 0;
    for piece in 1..=4 {
        white_piece_score +=
            pos.piece[piece].count_ones() as i32 * MATERIAL_SCORES[Phase::Opening as usize][piece];
        black_piece_score += pos.piece[piece + 6].count_ones() as i32
            * -MATERIAL_SCORES[Phase::Opening as usize][piece + 6];
    }
    white_piece_score + black_piece_score
}

fn add_positional_scores(sq: usize) -> (i32, i32) {
    let mut opening = 0;
    let mut endgame = 0;
    for i in 0..6 {
        // White pieces (add)
        opening += POSITIONAL_SCORES[Phase::Opening as usize][i][sq];
        endgame += POSITIONAL_SCORES[Phase::Endgame as usize][i][sq];

        // Black pieces (subtract)
        opening -= POSITIONAL_SCORES[Phase::Opening as usize][i][FLIP_SQ!(sq)];
        endgame -= POSITIONAL_SCORES[Phase::Endgame as usize][i][FLIP_SQ!(sq)];
    }
    (opening, endgame)
}

pub fn evaluate(
    pos: &Position,
    side: PieceColor,
    attack_info: &AttackInfo,
    mask: &EvalMasks,
) -> i32 {
    let phase_score = get_phase_score(pos);
    let phase = if phase_score >= OPENING_PHASE_SCORE {
        Phase::Opening
    } else if phase_score <= ENDGAME_PHASE_SCORE {
        Phase::Endgame
    } else {
        Phase::Middlegame
    };

    let mut opening = 0;
    let mut endgame = 0;
    let mut bb_copy; // The current piece's bitboard copy
    let mut sq;

    for piece in (Piece::LP as usize)..=(Piece::DK as usize) {
        bb_copy = pos.piece[piece];
        while bb_copy != 0 {
            sq = bb_copy.pop_lsb();

            // Add material score to opening and endgame scores
            opening += MATERIAL_SCORES[Phase::Opening as usize][piece];
            endgame += MATERIAL_SCORES[Phase::Endgame as usize][piece];

            let (positional_opening, positional_endgame) = add_positional_scores(sq);
            opening += positional_opening;
            endgame += positional_endgame;
            if let Some(val) = Piece::from_num(piece) {
                match val {
                    Piece::LP | Piece::LN | Piece::LB | Piece::LR | Piece::LQ | Piece::LK => {
                        let (light_opening, light_endgame) =
                            eval_light_pieces(val, pos, attack_info, mask, sq);
                        opening += light_opening;
                        endgame += light_endgame;
                    }
                    Piece::DP | Piece::DN | Piece::DB | Piece::DR | Piece::DQ | Piece::DK => {
                        let (dark_opening, dark_endgame) =
                            eval_dark_pieces(val, pos, attack_info, mask, sq);
                        opening += dark_opening;
                        endgame += dark_endgame;
                    }
                }
            }
        }
    }

    let score = match phase {
        Phase::Opening => opening,
        Phase::Endgame => endgame,
        Phase::Middlegame => {
            ((opening * phase_score) + (endgame * (OPENING_PHASE_SCORE - phase_score)))
                / OPENING_PHASE_SCORE
        }
    };

    if side == PieceColor::Light {
        score
    } else {
        -score
    }
}

fn eval_light_pieces(
    white_piece: Piece,
    pos: &Position,
    attack_info: &AttackInfo,
    mask: &EvalMasks,
    sq: usize,
) -> (i32, i32) {
    let mut opening = 0;
    let mut endgame = 0;
    match white_piece {
        Piece::LP => {
            let num_of_doubled_pawns =
                (pos.piece[Piece::LP as usize] & mask.file[COL!(sq)]).count_ones() as i32 - 1;
            if num_of_doubled_pawns > 0 {
                opening += num_of_doubled_pawns * DOUBLED_PAWN_PENALTY[Phase::Opening as usize];
                endgame += num_of_doubled_pawns * DOUBLED_PAWN_PENALTY[Phase::Endgame as usize];
            }
            if (pos.piece[Piece::LP as usize] & mask.isolated[COL!(sq)]) == 0 {
                opening += ISOLATED_PAWN_PENALTY[Phase::Opening as usize];
                endgame += ISOLATED_PAWN_PENALTY[Phase::Endgame as usize];
            }
            if (pos.piece[Piece::DP as usize] & mask.passed[PieceColor::Light as usize][sq]) == 0 {
                opening += PASSED_PAWN_BONUS[ROW!(sq)];
                endgame += PASSED_PAWN_BONUS[ROW!(sq)];
            }
        }

        Piece::LB => {
            opening += (attack_info
                .get_bishop_attack(Sq::from_num(sq), pos.units[PieceColor::Both as usize])
                - BISHOP_UNIT as u64)
                .count_ones() as i32
                * BISHOP_MOBILITY_BONUS[Phase::Opening as usize];
            endgame += (attack_info
                .get_bishop_attack(Sq::from_num(sq), pos.units[PieceColor::Both as usize])
                - BISHOP_UNIT as u64)
                .count_ones() as i32
                * BISHOP_MOBILITY_BONUS[Phase::Endgame as usize];
        }

        Piece::LR => {
            if (pos.piece[Piece::LP as usize] & mask.file[COL!(sq)]) == 0 {
                opening += SEMI_OPEN_FILE_BONUS;
                endgame += SEMI_OPEN_FILE_BONUS;
            }
            if ((pos.piece[Piece::LP as usize] | pos.piece[Piece::DP as usize])
                & mask.file[COL!(sq)])
                == 0
            {
                opening += OPEN_FILE_BONUS;
                endgame += OPEN_FILE_BONUS;
            }
        }

        Piece::LQ => {
            opening += (attack_info
                .get_queen_attack(Sq::from_num(sq), pos.units[PieceColor::Both as usize])
                - QUEEN_UNIT as u64)
                .count_ones() as i32
                * QUEEN_MOBILITY_BONUS[Phase::Opening as usize];
            endgame += (attack_info
                .get_queen_attack(Sq::from_num(sq), pos.units[PieceColor::Both as usize])
                - QUEEN_UNIT as u64)
                .count_ones() as i32
                * QUEEN_MOBILITY_BONUS[Phase::Endgame as usize];
        }

        Piece::LK => {
            if (pos.piece[Piece::LP as usize] & mask.file[COL!(sq)]) == 0 {
                // Semi open file penalty
                opening -= SEMI_OPEN_FILE_BONUS;
                endgame -= SEMI_OPEN_FILE_BONUS;
            }
            if ((pos.piece[Piece::LP as usize] | pos.piece[Piece::DP as usize])
                & mask.file[COL!(sq)])
                == 0
            {
                // Open file penalty
                opening -= OPEN_FILE_BONUS;
                endgame -= OPEN_FILE_BONUS;
            }
            // King safety bonus
            opening += (attack_info.king[sq] & pos.units[PieceColor::Light as usize]).count_ones()
                as i32
                * KING_SHIELD_BONUS;
            endgame += (attack_info.king[sq] & pos.units[PieceColor::Light as usize]).count_ones()
                as i32
                * KING_SHIELD_BONUS;
        }
        _ => {}
    }
    (opening, endgame)
}

fn eval_dark_pieces(
    black_piece: Piece,
    pos: &Position,
    attack_info: &AttackInfo,
    mask: &EvalMasks,
    sq: usize,
) -> (i32, i32) {
    let mut opening = 0;
    let mut endgame = 0;
    match black_piece {
        Piece::DP => {
            let num_of_doubled_pawns =
                (pos.piece[Piece::DP as usize] & mask.file[COL!(sq)]).count_ones() as i32 - 1;
            if num_of_doubled_pawns > 0 {
                opening -= num_of_doubled_pawns * DOUBLED_PAWN_PENALTY[Phase::Opening as usize];
                endgame -= num_of_doubled_pawns * DOUBLED_PAWN_PENALTY[Phase::Endgame as usize];
            }
            if (pos.piece[Piece::DP as usize] & mask.isolated[COL!(sq)]) == 0 {
                opening -= ISOLATED_PAWN_PENALTY[Phase::Opening as usize];
                endgame -= ISOLATED_PAWN_PENALTY[Phase::Endgame as usize];
            }
            if (pos.piece[Piece::LP as usize] & mask.passed[PieceColor::Dark as usize][sq]) == 0 {
                opening -= PASSED_PAWN_BONUS[ROW!(sq)];
                endgame -= PASSED_PAWN_BONUS[ROW!(sq)];
            }
        }

        Piece::DB => {
            opening -= (attack_info
                .get_bishop_attack(Sq::from_num(sq), pos.units[PieceColor::Both as usize])
                - BISHOP_UNIT as u64)
                .count_ones() as i32
                * BISHOP_MOBILITY_BONUS[Phase::Opening as usize];
            endgame -= (attack_info
                .get_bishop_attack(Sq::from_num(sq), pos.units[PieceColor::Both as usize])
                - BISHOP_UNIT as u64)
                .count_ones() as i32
                * BISHOP_MOBILITY_BONUS[Phase::Endgame as usize];
        }

        Piece::DR => {
            if (pos.piece[Piece::DP as usize] & mask.file[COL!(sq)]) == 0 {
                opening -= SEMI_OPEN_FILE_BONUS;
                endgame -= SEMI_OPEN_FILE_BONUS;
            }
            if ((pos.piece[Piece::LP as usize] | pos.piece[Piece::DP as usize])
                & mask.file[COL!(sq)])
                == 0
            {
                opening -= OPEN_FILE_BONUS;
                endgame -= OPEN_FILE_BONUS;
            }
        }

        Piece::DQ => {
            opening -= (attack_info
                .get_queen_attack(Sq::from_num(sq), pos.units[PieceColor::Both as usize])
                - QUEEN_UNIT as u64)
                .count_ones() as i32
                * QUEEN_MOBILITY_BONUS[Phase::Opening as usize];
            endgame -= (attack_info
                .get_queen_attack(Sq::from_num(sq), pos.units[PieceColor::Both as usize])
                - QUEEN_UNIT as u64)
                .count_ones() as i32
                * QUEEN_MOBILITY_BONUS[Phase::Endgame as usize];
        }

        Piece::DK => {
            if (pos.piece[Piece::DP as usize] & mask.file[COL!(sq)]) == 0 {
                // The semi open file bonus for the rook is used as a penalty for the king because the king isn't being shielded
                opening += SEMI_OPEN_FILE_BONUS;
                endgame += SEMI_OPEN_FILE_BONUS;
            }
            if ((pos.piece[Piece::LP as usize] | pos.piece[Piece::DP as usize])
                & mask.file[COL!(sq)])
                == 0
            {
                // The open file bonus for the rook is used as a penalty for the king because the king isn't being shielded
                opening += OPEN_FILE_BONUS;
                endgame += OPEN_FILE_BONUS;
            }
            // King safety bonus
            opening -= (attack_info.king[sq] & pos.units[PieceColor::Dark as usize]).count_ones()
                as i32
                * KING_SHIELD_BONUS;
            endgame -= (attack_info.king[sq] & pos.units[PieceColor::Dark as usize]).count_ones()
                as i32
                * KING_SHIELD_BONUS;
        }
        _ => {}
    }
    (opening, endgame)
}
