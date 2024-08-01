use crate::attack::AttackInfo;
use crate::bb::{BBUtil, BB};
use crate::board::Position;
use crate::consts::{Piece, PieceColor, Sq};
use crate::{FLIP_SQ, ROW, SQ};

#[derive(Debug)]
enum Phase {
    Opening,
    Endgame,
    Middlegame,
}

#[derive(Clone)]
pub struct EvalMasks {
    pub rank: [BB; 64],
    pub file: [BB; 64],
    pub isolated: [BB; 64],
    pub passed: [[BB; 64]; 2],
}

impl EvalMasks {
    pub fn new() -> Self {
        Self {
            rank: [0; 64],
            file: [0; 64],
            isolated: [0; 64],
            passed: [[0; 64]; 2],
        }
    }

    pub fn init(&mut self) {
        for r in 0..8i32 {
            for f in 0..8i32 {
                let sq = SQ!(r, f) as usize;
                self.rank[sq] |= set_file_and_rank(r, -1);
                self.file[sq] |= set_file_and_rank(-1, f);
            }
        }

        for r in 0..8i32 {
            for f in 0..8i32 {
                let sq = SQ!(r, f) as usize;
                self.isolated[sq] |= set_file_and_rank(-1, f - 1);
                self.isolated[sq] |= set_file_and_rank(-1, f + 1);

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
    for piece in (Piece::LN as usize)..=(Piece::LQ as usize) {
        white_piece_score +=
            pos.bitboards[piece].count_ones() as i32 * MATERIAL_SCORES[Phase::Opening as usize][piece];
    }
    for piece in (Piece::DN as usize)..=(Piece::DQ as usize) {
        black_piece_score += pos.bitboards[piece].count_ones() as i32
            * -MATERIAL_SCORES[Phase::Opening as usize][piece];
    }
    white_piece_score + black_piece_score
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
        bb_copy = pos.bitboards[piece];
        while bb_copy != 0 {
            sq = bb_copy.pop_lsb();

            // Add material score to opening and endgame scores
            opening += MATERIAL_SCORES[Phase::Opening as usize][piece];
            endgame += MATERIAL_SCORES[Phase::Endgame as usize][piece];

            if let Some(p) = Piece::from_num(piece) {
                match p {
                    Piece::LP | Piece::LN | Piece::LB | Piece::LR | Piece::LQ | Piece::LK => {
                        opening +=
                            POSITIONAL_SCORES[Phase::Opening as usize][(p as usize) % 6][sq];
                        endgame +=
                            POSITIONAL_SCORES[Phase::Endgame as usize][(p as usize) % 6][sq];

                        eval_light_pieces(
                            p,
                            pos,
                            attack_info,
                            mask,
                            sq,
                            &mut opening,
                            &mut endgame,
                        );
                    }
                    Piece::DP | Piece::DN | Piece::DB | Piece::DR | Piece::DQ | Piece::DK => {
                        opening -= POSITIONAL_SCORES[Phase::Opening as usize][(p as usize) % 6]
                            [FLIP_SQ!(sq)];
                        endgame -= POSITIONAL_SCORES[Phase::Endgame as usize][(p as usize) % 6]
                            [FLIP_SQ!(sq)];

                        eval_dark_pieces(
                            p,
                            pos,
                            attack_info,
                            mask,
                            sq,
                            &mut opening,
                            &mut endgame,
                        );
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
    opening: &mut i32,
    endgame: &mut i32
) {
    match white_piece {
        Piece::LP => {
            let num_of_doubled_pawns =
                (pos.bitboards[Piece::LP as usize] & mask.file[sq]).count_ones() as i32 - 1;
            if num_of_doubled_pawns > 0 {
                *opening += num_of_doubled_pawns * DOUBLED_PAWN_PENALTY[Phase::Opening as usize];
                *endgame += num_of_doubled_pawns * DOUBLED_PAWN_PENALTY[Phase::Endgame as usize];
            }
            if (pos.bitboards[Piece::LP as usize] & mask.isolated[sq]) == 0 {
                *opening += ISOLATED_PAWN_PENALTY[Phase::Opening as usize];
                *endgame += ISOLATED_PAWN_PENALTY[Phase::Endgame as usize];
            }
            if (pos.bitboards[Piece::DP as usize] & mask.passed[PieceColor::Light as usize][sq]) == 0 {
                *opening += PASSED_PAWN_BONUS[7 - ROW!(sq)];
                *endgame += PASSED_PAWN_BONUS[7 - ROW!(sq)];
            }
        }

        Piece::LB => {
            let both_units = pos.units(PieceColor::Both);
            *opening += (attack_info
                .get_bishop_attack(Sq::from_num(sq), both_units)
                .count_ones() as i32
                - BISHOP_UNIT)
                * BISHOP_MOBILITY_BONUS[Phase::Opening as usize];
            *endgame += (attack_info
                .get_bishop_attack(Sq::from_num(sq), both_units)
                .count_ones() as i32
                - BISHOP_UNIT)
                * BISHOP_MOBILITY_BONUS[Phase::Endgame as usize];
        }

        Piece::LR => {
            if (pos.bitboards[Piece::LP as usize] & mask.file[sq]) == 0 {
                *opening += SEMI_OPEN_FILE_BONUS;
                *endgame += SEMI_OPEN_FILE_BONUS;
            }
            if ((pos.bitboards[Piece::LP as usize] | pos.bitboards[Piece::DP as usize]) & mask.file[sq])
                == 0
            {
                *opening += OPEN_FILE_BONUS;
                *endgame += OPEN_FILE_BONUS;
            }
        }

        Piece::LQ => {
            let both_units = pos.units(PieceColor::Both);
            *opening += (attack_info
                .get_queen_attack(Sq::from_num(sq), both_units)
                .count_ones() as i32
                - QUEEN_UNIT)
                * QUEEN_MOBILITY_BONUS[Phase::Opening as usize];
            *endgame += (attack_info
                .get_queen_attack(Sq::from_num(sq), both_units)
                .count_ones() as i32
                - QUEEN_UNIT)
                * QUEEN_MOBILITY_BONUS[Phase::Endgame as usize];
        }

        Piece::LK => {
            if (pos.bitboards[Piece::LP as usize] & mask.file[sq]) == 0 {
                // Semi open file penalty
                *opening -= SEMI_OPEN_FILE_BONUS;
                *endgame -= SEMI_OPEN_FILE_BONUS;
            }
            if ((pos.bitboards[Piece::LP as usize] | pos.bitboards[Piece::DP as usize]) & mask.file[sq])
                == 0
            {
                // Open file penalty
                *opening -= OPEN_FILE_BONUS;
                *endgame -= OPEN_FILE_BONUS;
            }
            // King safety bonus
            let light_units = pos.units(PieceColor::Light);
            *opening += (attack_info.king[sq] & light_units).count_ones()
                as i32
                * KING_SHIELD_BONUS;
            *endgame += (attack_info.king[sq] & light_units).count_ones()
                as i32
                * KING_SHIELD_BONUS;
        }
        _ => {}
    };
}

fn eval_dark_pieces(
    black_piece: Piece,
    pos: &Position,
    attack_info: &AttackInfo,
    mask: &EvalMasks,
    sq: usize,
    opening: &mut i32,
    endgame: &mut i32
) {
    match black_piece {
        Piece::DP => {
            let num_of_doubled_pawns =
                (pos.bitboards[Piece::DP as usize] & mask.file[FLIP_SQ!(sq)]).count_ones() as i32 - 1;
            if num_of_doubled_pawns > 0 {
                *opening -= num_of_doubled_pawns * DOUBLED_PAWN_PENALTY[Phase::Opening as usize];
                *endgame -= num_of_doubled_pawns * DOUBLED_PAWN_PENALTY[Phase::Endgame as usize];
            }
            if (pos.bitboards[Piece::DP as usize] & mask.isolated[sq]) == 0 {
                *opening -= ISOLATED_PAWN_PENALTY[Phase::Opening as usize];
                *endgame -= ISOLATED_PAWN_PENALTY[Phase::Endgame as usize];
            }
            if (pos.bitboards[Piece::LP as usize] & mask.passed[PieceColor::Dark as usize][sq]) == 0 {
                *opening -= PASSED_PAWN_BONUS[7 - ROW!(sq)];
                *endgame -= PASSED_PAWN_BONUS[7 - ROW!(sq)];
            }
        }

        Piece::DB => {
            let both_units = pos.units(PieceColor::Both);
            *opening -= (attack_info
                .get_bishop_attack(Sq::from_num(sq), both_units)
                .count_ones() as i32
                - BISHOP_UNIT)
                * BISHOP_MOBILITY_BONUS[Phase::Opening as usize];
            *endgame -= (attack_info
                .get_bishop_attack(Sq::from_num(sq), both_units)
                .count_ones() as i32
                - BISHOP_UNIT)
                * BISHOP_MOBILITY_BONUS[Phase::Endgame as usize];
        }

        Piece::DR => {
            if (pos.bitboards[Piece::DP as usize] & mask.file[sq]) == 0 {
                *opening -= SEMI_OPEN_FILE_BONUS;
                *endgame -= SEMI_OPEN_FILE_BONUS;
            }
            if ((pos.bitboards[Piece::LP as usize] | pos.bitboards[Piece::DP as usize]) & mask.file[sq])
                == 0
            {
                *opening -= OPEN_FILE_BONUS;
                *endgame -= OPEN_FILE_BONUS;
            }
        }

        Piece::DQ => {
            let both_units = pos.units(PieceColor::Both);
            *opening -= (attack_info
                .get_queen_attack(Sq::from_num(sq), both_units)
                .count_ones() as i32
                - QUEEN_UNIT)
                * QUEEN_MOBILITY_BONUS[Phase::Opening as usize];
            *endgame -= (attack_info
                .get_queen_attack(Sq::from_num(sq), both_units)
                .count_ones() as i32
                - QUEEN_UNIT)
                * QUEEN_MOBILITY_BONUS[Phase::Endgame as usize];
        }

        Piece::DK => {
            if (pos.bitboards[Piece::DP as usize] & mask.file[sq]) == 0 {
                // The semi open file bonus for the rook is used as a penalty for the king because the king isn't being shielded
                *opening += SEMI_OPEN_FILE_BONUS;
                *endgame += SEMI_OPEN_FILE_BONUS;
            }
            if ((pos.bitboards[Piece::LP as usize] | pos.bitboards[Piece::DP as usize]) & mask.file[sq])
                == 0
            {
                // The open file bonus for the rook is used as a penalty for the king because the king isn't being shielded
                *opening += OPEN_FILE_BONUS;
                *endgame += OPEN_FILE_BONUS;
            }
            // King safety bonus
            let dark_units = pos.units(PieceColor::Dark);
            *opening -= (attack_info.king[sq] & dark_units).count_ones()
                as i32
                * KING_SHIELD_BONUS;
            *endgame -= (attack_info.king[sq] & dark_units).count_ones()
                as i32
                * KING_SHIELD_BONUS;
        }
        _ => {}
    };
}

// ======================== EVALUATION CONSTANTS ========================

pub const OPENING_PHASE_SCORE: i32 = 6192;
pub const ENDGAME_PHASE_SCORE: i32 = 518;

// Mobility bonus
pub const BISHOP_UNIT: i32 = 4;
pub const BISHOP_MOBILITY_BONUS: [i32; 2] = [5, 5];
pub const QUEEN_UNIT: i32 = 9;
pub const QUEEN_MOBILITY_BONUS: [i32; 2] = [1, 2];

// Shield bonus
pub const KING_SHIELD_BONUS: i32 = 5;

// Open(semi-open) file bonus
pub const SEMI_OPEN_FILE_BONUS: i32 = 10;
pub const OPEN_FILE_BONUS: i32 = 15;

// Passed pawns bonus
pub const PASSED_PAWN_BONUS: [i32; 8] = [0, 10, 30, 50, 75, 100, 150, 200];

// Isolated pawn penalty
pub const ISOLATED_PAWN_PENALTY: [i32; 2] = [-5, -10];
// Doubled pawns penalty
pub const DOUBLED_PAWN_PENALTY: [i32; 2] = [-5, -10];

pub const MATERIAL_SCORES: [[i32; 12]; 2] = [
    // opening material score
    [
        82, 337, 365, 477, 1025, 12000, -82, -337, -365, -477, -1025, -12000,
    ],
    // endgame material score
    [
        94, 281, 297, 512, 936, 12000, -94, -281, -297, -512, -936, -12000,
    ],
];

pub const POSITIONAL_SCORES: [[[i32; 64]; 6]; 2] = [
    // Opening positional piece scores //
    [
        [
            0, 0, 0, 0, 0, 0, 0, 0, 98, 134, 61, 95, 68, 126, 34, -11, -6, 7, 26, 31, 65, 56, 25,
            -20, -14, 13, 6, 21, 23, 12, 17, -23, -27, -2, -5, 12, 17, 6, 10, -25, -26, -4, -4,
            -10, 3, 3, 33, -12, -35, -1, -20, -23, -15, 24, 38, -22, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        // knight
        [
            -167, -89, -34, -49, 61, -97, -15, -107, -73, -41, 72, 36, 23, 62, 7, -17, -47, 60, 37,
            65, 84, 129, 73, 44, -9, 17, 19, 53, 37, 69, 18, 22, -13, 4, 16, 13, 28, 19, 21, -8,
            -23, -9, 12, 10, 19, 17, 25, -16, -29, -53, -12, -3, -1, 18, -14, -19, -105, -21, -58,
            -33, -17, -28, -19, -23,
        ],
        // bishop
        [
            -29, 4, -82, -37, -25, -42, 7, -8, -26, 16, -18, -13, 30, 59, 18, -47, -16, 37, 43, 40,
            35, 50, 37, -2, -4, 5, 19, 50, 37, 37, 7, -2, -6, 13, 13, 26, 34, 12, 10, 4, 0, 15, 15,
            15, 14, 27, 18, 10, 4, 15, 16, 0, 7, 21, 33, 1, -33, -3, -14, -21, -13, -12, -39, -21,
        ],
        // rook
        [
            32, 42, 32, 51, 63, 9, 31, 43, 27, 32, 58, 62, 80, 67, 26, 44, -5, 19, 26, 36, 17, 45,
            61, 16, -24, -11, 7, 26, 24, 35, -8, -20, -36, -26, -12, -1, 9, -7, 6, -23, -45, -25,
            -16, -17, 3, 0, -5, -33, -44, -16, -20, -9, -1, 11, -6, -71, -19, -13, 1, 17, 16, 7,
            -37, -26,
        ],
        // queen
        [
            28, 0, 29, 12, 59, 44, 43, 45, -24, -39, -5, 1, -16, 57, 28, 54, -13, -17, 7, 8, 29,
            56, 47, 57, -27, -27, -16, -16, -1, 17, -2, 1, -9, -26, -9, -10, -2, -4, 3, -3, -14, 2,
            -11, -2, -5, 2, 14, 5, -35, -8, 11, 2, 8, 15, -3, 1, -1, -18, -9, 10, -15, -25, -31,
            -50,
        ],
        // king
        [
            -65, 23, 16, -15, -56, -34, 2, 13, 29, -1, -20, -7, -8, -4, -38, -29, -9, 24, 2, -16,
            -20, 6, 22, -22, -17, -20, -12, -27, -30, -25, -14, -36, -49, -1, -27, -39, -46, -44,
            -33, -51, -14, -14, -22, -46, -44, -30, -15, -27, 1, 7, -8, -64, -43, -16, 9, 8, -15,
            36, 12, -54, 8, -28, 24, 14,
        ],
    ],
    // Endgame positional piece scores //
    [
        //pawn
        [
            0, 0, 0, 0, 0, 0, 0, 0, 178, 173, 158, 134, 147, 132, 165, 187, 94, 100, 85, 67, 56,
            53, 82, 84, 32, 24, 13, 5, -2, 4, 17, 17, 13, 9, -3, -7, -7, -8, 3, -1, 4, 7, -6, 1, 0,
            -5, -1, -8, 13, 8, 8, 10, 13, 0, 2, -7, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        // knight
        [
            -58, -38, -13, -28, -31, -27, -63, -99, -25, -8, -25, -2, -9, -25, -24, -52, -24, -20,
            10, 9, -1, -9, -19, -41, -17, 3, 22, 22, 22, 11, 8, -18, -18, -6, 16, 25, 16, 17, 4,
            -18, -23, -3, -1, 15, 10, -3, -20, -22, -42, -20, -10, -5, -2, -20, -23, -44, -29, -51,
            -23, -15, -22, -18, -50, -64,
        ],
        // bishop
        [
            -14, -21, -11, -8, -7, -9, -17, -24, -8, -4, 7, -12, -3, -13, -4, -14, 2, -8, 0, -1,
            -2, 6, 0, 4, -3, 9, 12, 9, 14, 10, 3, 2, -6, 3, 13, 19, 7, 10, -3, -9, -12, -3, 8, 10,
            13, 3, -7, -15, -14, -18, -7, -1, 4, -9, -15, -27, -23, -9, -23, -5, -9, -16, -5, -17,
        ],
        // rook
        [
            13, 10, 18, 15, 12, 12, 8, 5, 11, 13, 13, 11, -3, 3, 8, 3, 7, 7, 7, 5, 4, -3, -5, -3,
            4, 3, 13, 1, 2, 1, -1, 2, 3, 5, 8, 4, -5, -6, -8, -11, -4, 0, -5, -1, -7, -12, -8, -16,
            -6, -6, 0, 2, -9, -9, -11, -3, -9, 2, 3, -1, -5, -13, 4, -20,
        ],
        // queen
        [
            -9, 22, 22, 27, 27, 19, 10, 20, -17, 20, 32, 41, 58, 25, 30, 0, -20, 6, 9, 49, 47, 35,
            19, 9, 3, 22, 24, 45, 57, 40, 57, 36, -18, 28, 19, 47, 31, 34, 39, 23, -16, -27, 15, 6,
            9, 17, 10, 5, -22, -23, -30, -16, -16, -23, -36, -32, -33, -28, -22, -43, -5, -32, -20,
            -41,
        ],
        // king
        [
            -74, -35, -18, -18, -11, 15, 4, -17, -12, 17, 14, 17, 17, 38, 23, 11, 10, 17, 23, 15,
            20, 45, 44, 13, -8, 22, 24, 27, 26, 33, 26, 3, -18, -4, 21, 24, 27, 23, 9, -11, -19,
            -3, 11, 21, 23, 16, 7, -9, -27, -11, 4, 13, 14, 4, -5, -17, -53, -34, -21, -11, -28,
            -14, -24, -43,
        ],
    ],
];
