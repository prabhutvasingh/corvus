use crate::bitboard::*;
use crate::board::*;

pub const MATE: i32 = 32000;

pub const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, 0];

const PST_PAWN: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
    5, 5, 10, 25, 25, 10, 5, 5,
    0, 0, 0, 20, 20, 0, 0, 0,
    5, -5, -10, 0, 0, -10, -5, 5,
    5, 10, 10, -20, -20, 10, 10, 5,
    0, 0, 0, 0, 0, 0, 0, 0,
];

const PST_KNIGHT: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20, 0, 0, 0, 0, -20, -40,
    -30, 0, 10, 15, 15, 10, 0, -30,
    -30, 5, 15, 20, 20, 15, 5, -30,
    -30, 0, 15, 20, 20, 15, 0, -30,
    -30, 5, 10, 15, 15, 10, 5, -30,
    -40, -20, 0, 5, 5, 0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

const PST_BISHOP: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10, 0, 0, 0, 0, 0, 0, -10,
    -10, 0, 5, 10, 10, 5, 0, -10,
    -10, 5, 5, 10, 10, 5, 5, -10,
    -10, 0, 10, 10, 10, 10, 0, -10,
    -10, 10, 10, 10, 10, 10, 10, -10,
    -10, 5, 0, 0, 0, 0, 5, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

const PST_ROOK: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    5, 10, 10, 10, 10, 10, 10, 5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    -5, 0, 0, 0, 0, 0, 0, -5,
    0, 0, 0, 5, 5, 0, 0, 0,
];

const PST_QUEEN: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20,
    -10, 0, 0, 0, 0, 0, 0, -10,
    -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5,
    0, 0, 5, 5, 5, 5, 0, -5,
    -10, 5, 5, 5, 5, 5, 0, -10,
    -10, 0, 5, 0, 0, 0, 0, -10,
    -20, -10, -10, -5, -5, -10, -10, -20,
];

const PST_KING: [i32; 64] = [
    20, 30, 10, 0, 0, 10, 30, 20,
    20, 20, 0, 0, 0, 0, 20, 20,
    -10, -20, -20, -20, -20, -20, -20, -10,
    -20, -30, -30, -40, -40, -30, -30, -20,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
];

const PST_KING_EG: [i32; 64] = [
    -30, -20, -10, -10, -10, -10, -20, -30,
    -20, -10, 0, 0, 0, 0, -10, -20,
    -10, 0, 10, 15, 15, 10, 0, -10,
    -10, 5, 15, 20, 20, 15, 5, -10,
    -10, 0, 15, 20, 20, 15, 0, -10,
    -10, 5, 10, 15, 15, 10, 5, -10,
    -20, -10, 0, 5, 5, 0, -10, -20,
    -30, -20, -10, -10, -10, -10, -20, -30,
];

const PST: [[i32; 64]; 6] = [
    PST_PAWN, PST_KNIGHT, PST_BISHOP, PST_ROOK, PST_QUEEN, PST_KING,
];

const EG_PASSED: i32 = 28;
const KING_PROX: i32 = 10;

const MOB_WEIGHT: [i32; 6] = [0, 4, 4, 3, 2, 0];
const BISHOP_PAIR: i32 = 32;
const DOUBLED_PAWN: i32 = 10;
const ISOLATED_PAWN: i32 = 14;
const KING_SHIELD: i32 = 12;
const KING_EXPOSED: i32 = 45;
const ROOK_SEMI_OPEN: i32 = 10;
const ROOK_OPEN: i32 = 20;
const DEVELOPED_MINOR: i32 = 10;
const KING_ATK_COV: i32 = 3;
const KING_ATK_TWO: i32 = 18;
const KING_ATK_THREE: i32 = 30;

fn ahead_mask(color: usize, rank: usize, file: usize) -> u64 {
    let fm = FILE_MASK[file];
    let adj = fm | FILE_MASK[file.saturating_sub(1)] | FILE_MASK[(file + 1).min(7)];
    if color == WHITE {
        let hi = !(bit(8 * (rank + 1)) - 1);
        adj & hi
    } else {
        let lo = bit(8 * rank) - 1;
        adj & lo
    }
}

pub fn evaluate(b: &Board) -> i32 {
    let mut score = 0i32;

    let queens = b.pieces[0][QUEEN] | b.pieces[1][QUEEN];
    let rooks_all = b.pieces[0][ROOK] | b.pieces[1][ROOK];
    let eg_active = queens == 0 && rooks_all.count_ones() <= 1;

    for c in 0..2 {
        let mut side = 0i32;
        let own = b.occ[c];
        let mirror = c == BLACK;
        let ering = KING_ATTACKS[b.kingsq[c ^ 1]] | bit(b.kingsq[c ^ 1]);
        let mut attackers = 0;

        for pc in 0..5 {
            let mut bb = b.pieces[c][pc];
            while bb != 0 {
                let sq = bb.trailing_zeros() as usize;
                bb &= bb - 1;
                let mapped = if mirror { sq ^ 56 } else { sq };
                side += PIECE_VALUES[pc] + PST[pc][mapped];
                if pc == KNIGHT || pc == BISHOP || pc == ROOK || pc == QUEEN {
                    let attacks = match pc {
                        KNIGHT => KNIGHT_ATTACKS[sq],
                        BISHOP => bishop_attacks(b.all, sq),
                        ROOK => rook_attacks(b.all, sq),
                        _ => queen_attacks(b.all, sq),
                    };
                    side += MOB_WEIGHT[pc] * (attacks & !own).count_ones() as i32;
                    let on_ring = attacks & ering;
                    side += KING_ATK_COV * on_ring.count_ones() as i32;
                    if on_ring != 0 {
                        attackers += 1;
                    }
                }
            }
        }

        if attackers >= 3 {
            side += KING_ATK_THREE;
        } else if attackers >= 2 {
            side += KING_ATK_TWO;
        }

        if b.pieces[c][BISHOP].count_ones() >= 2 {
            side += BISHOP_PAIR;
        }

        let home_minors = if c == WHITE {
            bit(1) | bit(6) | bit(2) | bit(5)
        } else {
            bit(57) | bit(62) | bit(58) | bit(61)
        };
        side += DEVELOPED_MINOR * ((b.pieces[c][KNIGHT] | b.pieces[c][BISHOP]) & !home_minors).count_ones() as i32;

        let pawns = b.pieces[c][PAWN];
        let mut bb = pawns;
        while bb != 0 {
            let sq = bb.trailing_zeros() as usize;
            bb &= bb - 1;
            let f = file_of(sq);
            let r = rank_of(sq);

            if (pawns & FILE_MASK[f]).count_ones() > 1 {
                side -= DOUBLED_PAWN;
            }

            let adj_files = FILE_MASK[f.saturating_sub(1)] | FILE_MASK[(f + 1).min(7)];
            if pawns & adj_files == 0 {
                side -= ISOLATED_PAWN;
            }

            if b.pieces[c ^ 1][PAWN] & ahead_mask(c, r, f) == 0 {
                let progress = if c == WHITE { r as i32 } else { 7 - r as i32 };
                side += 14 + 12 * progress;
                if eg_active {
                    side += EG_PASSED + 8 * progress;
                }
            }
        }

        let mut rooks = b.pieces[c][ROOK];
        while rooks != 0 {
            let sq = rooks.trailing_zeros() as usize;
            rooks &= rooks - 1;
            let f = file_of(sq);
            let own_p = pawns & FILE_MASK[f];
            let opp_p = b.pieces[c ^ 1][PAWN] & FILE_MASK[f];
            if own_p == 0 {
                side += if opp_p == 0 { ROOK_OPEN } else { ROOK_SEMI_OPEN };
            }
        }

        let ks = b.kingsq[c];
        let (kf, kr) = (file_of(ks), rank_of(ks));
        let sr = if c == WHITE { kr + 1 } else { kr.saturating_sub(1) };
        if sr < 8 {
            for df in -1i32..=1i32 {
                let sf = kf as i32 + df;
                if (0..8).contains(&sf) && pawns & bit(sr * 8 + sf as usize) != 0 {
                    side += KING_SHIELD;
                }
            }
        }
        let queens = b.pieces[0][QUEEN] | b.pieces[1][QUEEN];
        let rooks_all = b.pieces[0][ROOK] | b.pieces[1][ROOK];
        let midgame = queens.count_ones() > 0 || rooks_all.count_ones() >= 3;
        let home_rank = if c == WHITE { 0 } else { 7 };
        if midgame && kr as i32 != home_rank {
            side -= KING_EXPOSED;
        }

        let mapped = if mirror { ks ^ 56 } else { ks };
        let king_pst = if eg_active { PST_KING_EG[mapped] } else { PST_KING[mapped] };
        score += if c == WHITE { side + king_pst } else { -(side + king_pst) };
    }

    if eg_active {
        let cd = ((file_of(b.kingsq[0]) as i32 - file_of(b.kingsq[1]) as i32).abs())
            .max((rank_of(b.kingsq[0]) as i32 - rank_of(b.kingsq[1]) as i32).abs());
        score += (7 - cd) * KING_PROX;
    }

    if b.side == WHITE {
        score + 12
    } else {
        -score + 12
    }
}