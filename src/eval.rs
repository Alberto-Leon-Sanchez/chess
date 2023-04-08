use serde::__private::de;
use tch::nn::ModuleT;

use crate::api::board120_to_board64;
use crate::attack_gen;
use crate::game;
use crate::game::Color;
use crate::game::GameInfo;
use crate::model;
use crate::move_gen;
use crate::move_gen::move_gen;
use crate::piece;
use crate::piece::Piece;

const WK: usize = 0;
const WQ: usize = 1;
const WR: usize = 2;
const WN: usize = 3;
const WB: usize = 4;
const WP: usize = 5;
const BK: usize = 6;
const BQ: usize = 7;
const BR: usize = 8;
const BN: usize = 9;
const BB: usize = 10;
const BP: usize = 11;

const MAT: [[i32; 12]; 2] = [
    [
        10000, 2521, 1270, 817, 836, 198, 10000, 2521, 1270, 817, 836, 198,
    ],
    [
        10000, 2558, 1278, 846, 857, 258, 10000, 2558, 1278, 846, 857, 258,
    ],
];

const MAX_POSITIONAL_SCORE: i32 = 150;

const Q_PHASE_CONTRIBUTION: i32 = 4;
const R_PHASE_CONTRIBUTION: i32 = 2;
const B_PHASE_CONTRIBUTION: i32 = 1;
const N_PHASE_CONTRIBUTION: i32 = 1;
const P_PHASE_CONTRIBUTION: i32 = 0;

const MOBILITY_MULTIPLIERS: [i32; 2] = [0, 1];

const MAX_PHASE: i32 = Q_PHASE_CONTRIBUTION * 2
    + R_PHASE_CONTRIBUTION * 4
    + B_PHASE_CONTRIBUTION * 4
    + N_PHASE_CONTRIBUTION * 4
    + P_PHASE_CONTRIBUTION * 16;

const PAWN_PCSQ_MULTIPLIERS: [i32; 2] = [1, 2];
const PAWN_PCSQ: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 1, 3, 2, 4, 4, 2, 3, 1, 2, 6, 4, 8, 8, 4, 6,
    2, 3, 9, 6, 12, 12, 6, 9, 3, 4, 12, 8, 16, 16, 8, 12, 4, 5, 15, 10, 20, 20, 10, 15, 5, 0, 0, 0,
    0, 0, 0, 0, 0,
];

pub const KNIGHT_PCSQ: [[i32; 64]; 2] = [
    [
        -144, -109, -85, -73, -73, -85, -109, -144, -88, -43, -19, -7, -7, -19, -43, -88, -69, -24,
        0, 12, 12, 0, -24, -69, -28, 17, 41, 53, 53, 41, 17, -28, -30, 15, 39, 51, 51, 39, 15, -30,
        -10, 35, 59, 71, 71, 59, 35, -10, -64, -19, 5, 17, 17, 5, -19, -64, -200, -65, -41, -29,
        -29, -41, -65, -200,
    ],
    [
        -98, -83, -51, -16, -16, -51, -83, -98, -68, -53, -21, 14, 14, -21, -53, -68, -53, -38, -6,
        29, 29, -6, -38, -53, -42, -27, 5, 40, 40, 5, -27, -42, -42, -27, 5, 40, 40, 5, -27, -42,
        -53, -38, -6, 29, 29, -6, -38, -53, -68, -53, -21, 14, 14, -21, -53, -68, -98, -83, -51,
        -16, -16, -51, -83, -98,
    ],
];

pub const KNIGHT_MOBILITY: [[i32; 9]; 2] = [
    [-65, -42, -9, 3, 15, 27, 37, 42, 44],
    [-50, -30, -10, 0, 10, 20, 28, 31, 33],
];

pub const BISHOP_PCSQ: [[i32; 64]; 2] = [
    [
        -54, -27, -34, -43, -43, -34, -27, -54, -29, 8, 1, -8, -8, 1, 8, -29, -20, 17, 10, 1, 1,
        10, 17, -20, -19, 18, 11, 2, 2, 11, 18, -19, -22, 15, 8, -1, -1, 8, 15, -22, -28, 9, 2, -7,
        -7, 2, 9, -28, -32, 5, -2, -11, -11, -2, 5, -32, -49, -22, -29, -38, -38, -29, -22, -49,
    ],
    [
        -65, -42, -44, -26, -26, -44, -42, -65, -43, -20, -22, -4, -4, -22, -20, -43, -33, -10,
        -12, 6, 6, -12, -10, -33, -35, -12, -14, 4, 4, -14, -12, -35, -35, -12, -14, 4, 4, -14,
        -12, -35, -33, -10, -12, 6, 6, -12, -10, -33, -43, -20, -22, -4, -4, -22, -20, -43, -65,
        -42, -44, -26, -26, -44, -42, -65,
    ],
];

pub const BISHOP_MOBILITY: [[i32; 14]; 2] = [
    [-52, -28, 6, 20, 34, 48, 60, 68, 74, 77, 80, 82, 84, 86],
    [-47, -23, 1, 15, 29, 43, 55, 63, 68, 72, 75, 77, 84, 86],
];

pub const BISHOP_PAIR_BONUS: [i32; 2] = [50, 75];

pub const ROOK_PCSQ: [[i32; 64]; 2] = [
    [
        -22, -17, -12, -8, -8, -12, -17, -22, -22, -7, -2, 2, 2, -2, -7, -22, -22, -7, -2, 2, 2,
        -2, -7, -22, -22, -7, -2, 2, 2, -2, -7, -22, -22, -7, -2, 2, 2, -2, -7, -22, -22, -7, -2,
        2, 2, -2, -7, -22, -11, 4, 9, 13, 13, 9, 4, -11, -22, -17, -12, -8, -8, -12, -17, -22,
    ],
    [
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3,
    ],
];

pub const ROOK_MOBILITY: [[i32; 15]; 2] = [
    [-47, -31, -5, 1, 7, 13, 18, 22, 26, 29, 31, 33, 35, 36, 37],
    [
        -53, -26, 0, 16, 32, 48, 64, 80, 96, 109, 115, 119, 122, 123, 124,
    ],
];

pub const QUEEN_PCSQ_MULTIPLIERS: [f32; 2] = [5.0, 5.0];

pub const QUEEN_PCSQ: [[i32; 64]; 2] = [
    [
        -2, -2, -2, -2, -2, -2, -2, -2, -2, 8, 8, 8, 8, 8, 8, -2, -2, 8, 8, 8, 8, 8, 8, -2, -2, 8,
        8, 8, 8, 8, 8, -2, -2, 8, 8, 8, 8, 8, 8, -2, -2, 8, 8, 8, 8, 8, 8, -2, -2, 8, 8, 8, 8, 8,
        8, -2, -2, -2, -2, -2, -2, -2, -2, -2,
    ],
    [
        -80, -54, -42, -30, -30, -42, -54, -80, -54, -30, -18, -6, -6, -18, -30, -54, -42, -18, -6,
        6, 6, -6, -18, -42, -30, -6, 6, 18, 18, 6, -6, -30, -30, -6, 6, 18, 18, 6, -6, -30, -42,
        -18, -6, 6, 6, -6, -18, -42, -54, -30, -18, -6, -6, -18, -30, -54, -80, -54, -42, -30, -30,
        -42, -54, -80,
    ],
];

pub const QUEEN_MOBILITY: [[i32; 28]; 2] = [
    [
        -42, -28, -5, 0, 6, 11, 13, 18, 20, 21, 22, 22, 22, 23, 24, 25, 25, 25, 25, 25, 25, 25, 25,
        25, 25, 25, 25, 25,
    ],
    [
        -40, -23, -7, 0, 10, 19, 29, 38, 40, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41, 41,
        41, 41, 41, 41, 41, 41,
    ],
];

pub const KING_PCSQ: [[i32; 64]; 2] = [
    [
        298, 332, 273, 225, 225, 273, 332, 298, 287, 321, 262, 214, 214, 262, 321, 287, 224, 258,
        199, 151, 151, 199, 258, 224, 196, 230, 171, 123, 123, 171, 230, 196, 173, 207, 148, 100,
        100, 148, 207, 173, 146, 180, 121, 73, 73, 121, 180, 146, 119, 153, 94, 46, 46, 94, 153,
        119, 98, 132, 73, 25, 25, 73, 132, 98,
    ],
    [
        27, 81, 108, 116, 116, 108, 81, 27, 74, 128, 155, 163, 163, 155, 128, 74, 111, 165, 192,
        200, 200, 192, 165, 111, 135, 189, 216, 224, 224, 216, 189, 135, 135, 189, 216, 224, 224,
        216, 189, 135, 111, 165, 192, 200, 200, 192, 165, 111, 74, 128, 155, 163, 163, 155, 128,
        74, 27, 81, 108, 116, 116, 108, 81, 27,
    ],
];

pub const SIDE_TO_MOVE_BONUS: i32 = 14;

const FLIP: [usize; 64] = [
    56, 57, 58, 59, 60, 61, 62, 63, 48, 49, 50, 51, 52, 53, 54, 55, 40, 41, 42, 43, 44, 45, 46, 47,
    32, 33, 34, 35, 36, 37, 38, 39, 24, 25, 26, 27, 28, 29, 30, 31, 16, 17, 18, 19, 20, 21, 22, 23,
    8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7,
];


pub fn order_moves(moves: &mut Vec<move_gen::Move>) {
    let mut index: usize = 0;

    for movement in 0..moves.len() {
        if moves[movement].destiny_piece != piece::Piece::Empty {
            moves.swap(index, movement);
            index += 1;
        }
    }
}

pub fn check(game: &mut game::GameInfo, color: game::Color) -> bool {
    let (_, attacks) = attack_gen::attack_gen(game, Some(&color));

    attacks != 0
}

pub fn net_eval(game: &mut game::GameInfo, net: &model::Net, device: &tch::Device) -> f64 {
    tch::no_grad(|| {
        net.forward_t(&model::pre_proccess(game, device), false)
            .double_value(&[0])
    })
}

pub fn net_eval_tch(game: &mut game::GameInfo, net: &model::Net, device: &tch::Device) -> tch::Tensor {
    tch::no_grad(|| net.forward_t(&model::pre_proccess(game, device), false))
}

fn scale_phase(opening_score: i32, endgame_score: i32, phase: i32) -> i32 {
    let diff = opening_score - endgame_score;

    endgame_score + diff * phase / MAX_PHASE
}

pub fn evaluate_material(game: &GameInfo) -> f64 {
    let mut ret: i32 = 0;

    let wqcount = game.white_pieces.queens.len() as i32;
    let bqcount = game.black_pieces.queens.len() as i32;
    let wrcount = game.white_pieces.rooks.len() as i32;
    let brcount = game.black_pieces.rooks.len() as i32;
    let wbcount = game.white_pieces.bishops.len() as i32;
    let bbcount = game.black_pieces.bishops.len() as i32;
    let wncount = game.white_pieces.knights.len() as i32;
    let bncount = game.black_pieces.knights.len() as i32;
    let wpcount = game.white_pieces.pawns.len() as i32;
    let bpcount = game.black_pieces.pawns.len() as i32;

    let mut phase = wqcount * Q_PHASE_CONTRIBUTION
        + bqcount * Q_PHASE_CONTRIBUTION
        + wrcount * R_PHASE_CONTRIBUTION
        + brcount * R_PHASE_CONTRIBUTION
        + wbcount * B_PHASE_CONTRIBUTION
        + bbcount * B_PHASE_CONTRIBUTION
        + wncount * N_PHASE_CONTRIBUTION
        + bncount * N_PHASE_CONTRIBUTION
        + wpcount * P_PHASE_CONTRIBUTION
        + bpcount * P_PHASE_CONTRIBUTION;

    if phase > MAX_PHASE {
        phase = MAX_PHASE;
    }

    ret += wqcount * scale_phase(MAT[0][WQ], MAT[1][WQ], phase.try_into().unwrap());
    ret -= bqcount * scale_phase(MAT[0][BQ], MAT[1][BQ], phase.try_into().unwrap());
    ret += wrcount * scale_phase(MAT[0][WR], MAT[1][WR], phase.try_into().unwrap());
    ret -= brcount * scale_phase(MAT[0][BR], MAT[1][BR], phase.try_into().unwrap());
    ret += wbcount * scale_phase(MAT[0][WB], MAT[1][WB], phase.try_into().unwrap());
    ret -= bbcount * scale_phase(MAT[0][BB], MAT[1][BB], phase.try_into().unwrap());
    ret += wncount * scale_phase(MAT[0][WN], MAT[1][WN], phase.try_into().unwrap());
    ret -= bncount * scale_phase(MAT[0][BN], MAT[1][BN], phase.try_into().unwrap());
    ret += wpcount * scale_phase(MAT[0][WP], MAT[1][WP], phase.try_into().unwrap());
    ret -= bpcount * scale_phase(MAT[0][BP], MAT[1][BP], phase.try_into().unwrap());

    (1e-3 * ret as f64).tanh()
}

fn evaluate_pawns(game: &GameInfo, phase: i32) -> i32 {
    let mut ret = 0;

    for white in &game.white_pieces.pawns {
        let index = board120_to_board64(*white) as usize;
        ret += scale_phase(
            PAWN_PCSQ[index] * PAWN_PCSQ_MULTIPLIERS[0],
            PAWN_PCSQ[index] * PAWN_PCSQ_MULTIPLIERS[1],
            phase,
        );
    }

    for black in &game.black_pieces.pawns {
        let index = FLIP[board120_to_board64(*black) as usize];
        ret -= scale_phase(
            PAWN_PCSQ[index] * PAWN_PCSQ_MULTIPLIERS[0],
            PAWN_PCSQ[index] * PAWN_PCSQ_MULTIPLIERS[1],
            phase,
        );
    }

    ret
}

fn evaluate_knights(
    game: &GameInfo,
    phase: i32,
    white_mobility: Vec<i32>,
    black_mobility: Vec<i32>,
) -> i32 {
    let mut ret: i32 = 0;

    for knight in &game.white_pieces.knights {
        let index = board120_to_board64(*knight) as usize;
        let mobility = white_mobility[*knight as usize] as usize;
        ret += scale_phase(
            KNIGHT_MOBILITY[0][mobility] * MOBILITY_MULTIPLIERS[0],
            KNIGHT_MOBILITY[1][mobility] * MOBILITY_MULTIPLIERS[1],
            phase,
        );
        ret += scale_phase(KNIGHT_PCSQ[0][index], KNIGHT_PCSQ[1][index], phase);
    }

    for knight in &game.black_pieces.knights {
        let index = FLIP[board120_to_board64(*knight) as usize];
        let mobility = black_mobility[*knight as usize] as usize;
        ret -= scale_phase(
            KNIGHT_MOBILITY[0][mobility] * MOBILITY_MULTIPLIERS[0],
            KNIGHT_MOBILITY[1][mobility] * MOBILITY_MULTIPLIERS[1],
            phase,
        );
        ret -= scale_phase(KNIGHT_PCSQ[0][index], KNIGHT_PCSQ[1][index], phase);
    }

    ret
}

fn evaluate_bishops(
    game: &GameInfo,
    phase: i32,
    white_mobility: Vec<i32>,
    black_mobility: Vec<i32>,
) -> i32 {
    let mut ret = 0;

    if game.white_pieces.bishops.len() >= 2 {
        ret += scale_phase(BISHOP_PAIR_BONUS[0], BISHOP_PAIR_BONUS[1], phase);
    }

    if game.black_pieces.bishops.len() >= 2 {
        ret -= scale_phase(BISHOP_PAIR_BONUS[0], BISHOP_PAIR_BONUS[1], phase);
    }

    for bishop in &game.white_pieces.bishops {
        let index = board120_to_board64(*bishop) as usize;
        let mobility = white_mobility[*bishop as usize] as usize;
        ret += scale_phase(
            BISHOP_MOBILITY[0][mobility] * MOBILITY_MULTIPLIERS[0],
            BISHOP_MOBILITY[1][mobility] * MOBILITY_MULTIPLIERS[1],
            phase,
        );
        ret += scale_phase(BISHOP_PCSQ[0][index], BISHOP_PCSQ[1][index], phase);
    }

    for bishop in &game.black_pieces.bishops {
        let index = FLIP[board120_to_board64(*bishop) as usize];
        let mobility = black_mobility[*bishop as usize] as usize;
        ret -= scale_phase(
            BISHOP_MOBILITY[0][mobility] * MOBILITY_MULTIPLIERS[0],
            BISHOP_MOBILITY[1][mobility] * MOBILITY_MULTIPLIERS[1],
            phase,
        );
        ret -= scale_phase(BISHOP_PCSQ[0][index], BISHOP_PCSQ[1][index], phase);
    }

    ret
}

fn evaluate_rooks(
    game: &GameInfo,
    phase: i32,
    white_mobility: Vec<i32>,
    black_mobility: Vec<i32>,
) -> i32 {
    let mut ret = 0;

    for rook in &game.white_pieces.rooks {
        let index = board120_to_board64(*rook) as usize;
        let mobility = white_mobility[*rook as usize] as usize;
        ret += scale_phase(
            ROOK_MOBILITY[0][mobility] * MOBILITY_MULTIPLIERS[0],
            ROOK_MOBILITY[1][mobility] * MOBILITY_MULTIPLIERS[1],
            phase,
        );
        ret += scale_phase(ROOK_PCSQ[0][index], ROOK_PCSQ[1][index], phase);
    }

    for rook in &game.black_pieces.rooks {
        let index = FLIP[board120_to_board64(*rook) as usize];
        let mobility = black_mobility[*rook as usize] as usize;
        ret -= scale_phase(
            ROOK_MOBILITY[0][mobility] * MOBILITY_MULTIPLIERS[0],
            ROOK_MOBILITY[1][mobility] * MOBILITY_MULTIPLIERS[1],
            phase,
        );
        ret -= scale_phase(ROOK_PCSQ[0][index], ROOK_PCSQ[1][index], phase);
    }

    ret
}

fn evaluate_queens(
    game: &GameInfo,
    phase: i32,
    white_mobility: Vec<i32>,
    black_mobility: Vec<i32>,
) -> i32 {
    let mut ret = 0;

    for queen in &game.white_pieces.queens {
        let index = board120_to_board64(*queen) as usize;
        let mobility = white_mobility[*queen as usize] as usize;
        ret += scale_phase(
            QUEEN_MOBILITY[0][mobility] * MOBILITY_MULTIPLIERS[0],
            QUEEN_MOBILITY[1][mobility] * MOBILITY_MULTIPLIERS[1],
            phase,
        );
        ret += scale_phase(QUEEN_PCSQ[0][index], QUEEN_PCSQ[1][index], phase);
    }

    for queen in &game.black_pieces.queens {
        let index = FLIP[board120_to_board64(*queen) as usize];
        let mobility = black_mobility[*queen as usize] as usize;
        ret -= scale_phase(
            QUEEN_MOBILITY[0][mobility] * MOBILITY_MULTIPLIERS[0],
            QUEEN_MOBILITY[1][mobility] * MOBILITY_MULTIPLIERS[1],
            phase,
        );
        ret -= scale_phase(QUEEN_PCSQ[0][index], QUEEN_PCSQ[1][index], phase);
    }

    ret
}

fn evaluate_kings(game: &GameInfo, phase: i32) -> i32 {
    let mut ret = 0;

    let index = board120_to_board64(game.white_pieces.kings[0]) as usize;
    ret += scale_phase(KING_PCSQ[0][index], KING_PCSQ[1][index], phase);

    let index = FLIP[board120_to_board64(game.black_pieces.kings[0]) as usize];
    ret -= scale_phase(KING_PCSQ[0][index], KING_PCSQ[1][index], phase);

    ret
}

pub fn static_evaluate(game: &mut GameInfo) -> f64 {
    
    if move_gen::move_gen(game).len() == 0{
        if game.turn == game::Color::White{
            if check(game, game::Color::Black){
                return -1.0
            }else{
                return 0.0
            }
        }else{
            if check(game, game::Color::White){
                return 1.0
            }else{
                return 0.0
            }
        }
    }

    let mut ret = 0;

    let wqcount = game.white_pieces.queens.len() as i32;
    let bqcount = game.black_pieces.queens.len() as i32;
    let wrcount = game.white_pieces.rooks.len() as i32;
    let brcount = game.black_pieces.rooks.len() as i32;
    let wbcount = game.white_pieces.bishops.len() as i32;
    let bbcount = game.black_pieces.bishops.len() as i32;
    let wncount = game.white_pieces.knights.len() as i32;
    let bncount = game.black_pieces.knights.len() as i32;
    let wpcount = game.white_pieces.pawns.len() as i32;
    let bpcount = game.black_pieces.pawns.len() as i32;

    let mut phase = wqcount * Q_PHASE_CONTRIBUTION
        + bqcount * Q_PHASE_CONTRIBUTION
        + wrcount * R_PHASE_CONTRIBUTION
        + brcount * R_PHASE_CONTRIBUTION
        + wbcount * B_PHASE_CONTRIBUTION
        + bbcount * B_PHASE_CONTRIBUTION
        + wncount * N_PHASE_CONTRIBUTION
        + bncount * N_PHASE_CONTRIBUTION
        + wpcount * P_PHASE_CONTRIBUTION
        + bpcount * P_PHASE_CONTRIBUTION;

    if phase > MAX_PHASE {
        phase = MAX_PHASE;
    }

    ret += wqcount * scale_phase(MAT[0][WQ], MAT[1][WQ], phase.try_into().unwrap());
    ret -= bqcount * scale_phase(MAT[0][BQ], MAT[1][BQ], phase.try_into().unwrap());
    ret += wrcount * scale_phase(MAT[0][WR], MAT[1][WR], phase.try_into().unwrap());
    ret -= brcount * scale_phase(MAT[0][BR], MAT[1][BR], phase.try_into().unwrap());
    ret += wbcount * scale_phase(MAT[0][WB], MAT[1][WB], phase.try_into().unwrap());
    ret -= bbcount * scale_phase(MAT[0][BB], MAT[1][BB], phase.try_into().unwrap());
    ret += wncount * scale_phase(MAT[0][WN], MAT[1][WN], phase.try_into().unwrap());
    ret -= bncount * scale_phase(MAT[0][BN], MAT[1][BN], phase.try_into().unwrap());
    ret += wpcount * scale_phase(MAT[0][WP], MAT[1][WP], phase.try_into().unwrap());
    ret -= bpcount * scale_phase(MAT[0][BP], MAT[1][BP], phase.try_into().unwrap());

    let white_moves;
    let black_moves;

    if game.turn == game::Color::White {
        white_moves = move_gen(game);
        game.turn = game::Color::Black;
        black_moves = move_gen(game);
        game.turn = game::Color::White;
    } else {
        black_moves = move_gen(game);
        game.turn = game::Color::White;
        white_moves = move_gen(game);
        game.turn = game::Color::Black;
    }

    let mut white_pawns_attacks: Vec<i8> = Vec::new();
    let mut black_pawns_attacks: Vec<i8> = Vec::new();

    let mut white_safe_destinations = vec![true; 120];
    let mut black_safe_destinations = vec![true; 120];

    for movement in &white_moves {
        match game.board[movement.origin as usize] {
            Piece::White(piece::PieceType::Pawn) => {
                if (movement.destiny - movement.origin) % 10 != 0{
                    white_pawns_attacks.push(movement.destiny);
                }
            }
            _ => (),
        }
    }

    for movement in &black_moves {
        match game.board[movement.origin as usize] {
            Piece::Black(piece::PieceType::Pawn) => {
                if (movement.origin - movement.destiny) % 10 != 0{
                    black_pawns_attacks.push(movement.destiny);
                }
            }
            _ => (),
        }
    }

    for attack in &black_pawns_attacks {
        white_safe_destinations[*attack as usize] = false;
    }

    for attack in &white_pawns_attacks {
        black_safe_destinations[*attack as usize] = false;
    }

    let mut white_bishop_mobility = vec![0; 120];
    let mut black_bishop_mobility = vec![0; 120];

    let mut white_knight_mobility = vec![0; 120];
    let mut black_knight_mobility = vec![0; 120];

    let mut white_rook_mobility = vec![0; 120];
    let mut black_rook_mobility = vec![0; 120];

    let mut white_queen_mobility = vec![0; 120];
    let mut black_queen_mobility = vec![0; 120];

    for movement in white_moves {
        match game.board[movement.origin as usize] {
            Piece::White(piece::PieceType::Bishop) => {
                if white_safe_destinations[movement.destiny as usize] {
                    white_bishop_mobility[movement.origin as usize] += 1;
                }
            }
            Piece::White(piece::PieceType::Knight) => {
                if white_safe_destinations[movement.destiny as usize] {
                    white_knight_mobility[movement.origin as usize] += 1;
                }
            }
            Piece::White(piece::PieceType::Rook) => {
                if white_safe_destinations[movement.destiny as usize] {
                    white_rook_mobility[movement.origin as usize] += 1;
                }
            }
            Piece::White(piece::PieceType::Queen) => {
                if white_safe_destinations[movement.destiny as usize] {
                    white_queen_mobility[movement.origin as usize] += 1;
                }
            }
            _ => (),
        }
    }

    for movement in black_moves {
        match game.board[movement.origin as usize] {
            Piece::Black(piece::PieceType::Bishop) => {
                if black_safe_destinations[movement.destiny as usize] {
                    black_bishop_mobility[movement.origin as usize] += 1;
                }
            }
            Piece::Black(piece::PieceType::Knight) => {
                if black_safe_destinations[movement.destiny as usize] {
                    black_knight_mobility[movement.origin as usize] += 1;
                }
            }
            Piece::Black(piece::PieceType::Rook) => {
                if black_safe_destinations[movement.destiny as usize] {
                    black_rook_mobility[movement.origin as usize] += 1;
                }
            }
            Piece::Black(piece::PieceType::Queen) => {
                if black_safe_destinations[movement.destiny as usize] {
                    black_queen_mobility[movement.origin as usize] += 1;
                }
            }
            _ => (),
        }
    }

    ret += evaluate_pawns(game, phase);

    ret += evaluate_knights(game, phase, white_knight_mobility, black_knight_mobility);

    ret += evaluate_bishops(game, phase, white_bishop_mobility, black_bishop_mobility);

    ret += evaluate_rooks(game, phase, white_rook_mobility, black_rook_mobility);

    ret += evaluate_queens(game, phase, white_queen_mobility, black_queen_mobility);

    ret += evaluate_kings(game, phase);

    ret += if game.turn == Color::White {
        SIDE_TO_MOVE_BONUS
    } else {
        -SIDE_TO_MOVE_BONUS
    };

    (1e-3 * ret as f64).tanh()
}

