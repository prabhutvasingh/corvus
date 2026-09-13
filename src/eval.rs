// Corvus Chess Engine
// Copyright (C) 2026 Avi
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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

const PST_PAWN_EG: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
];

const PST_EG: [[i32; 64]; 6] = [
    PST_PAWN_EG, PST_KNIGHT, PST_BISHOP, PST_ROOK, PST_QUEEN, PST_KING_EG,
];

const EG_PASSED: i32 = 28;
const KING_PROX: i32 = 10;
const PIN_PEN: [i32; 6] = [10, 35, 30, 45, 60, 0];

const MOB_WEIGHT: [i32; 6] = [0, 4, 4, 3, 2, 0];
const BISHOP_PAIR: i32 = 32;
const FORK_BONUS: i32 = 30;
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
const KS_W: [i32; 6] = [0, 35, 35, 55, 70, 0];

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

fn pinned_mask(b: &Board, side: usize) -> u64 {
    let ks = b.kingsq[side];
    let enemy = side ^ 1;
    let own = b.occ[side];
    let occ = b.all;
    let dia = b.pieces[enemy][BISHOP] | b.pieces[enemy][QUEEN];
    let orth = b.pieces[enemy][ROOK] | b.pieces[enemy][QUEEN];

    let dirs: [(i32, i32); 8] = [
        (0, 1), (0, -1), (1, 0), (-1, 0), (1, 1), (1, -1), (-1, 1), (-1, -1),
    ];
    let mut pinned = 0u64;
    for (dr, df) in dirs {
        let diag = dr != 0 && df != 0;
        let sliders = if diag { dia } else { orth };
        let mut kr = rank_of(ks) as i32 + dr;
        let mut kf = file_of(ks) as i32 + df;
        let mut blocker: Option<usize> = None;
        while (0..8).contains(&kr) && (0..8).contains(&kf) {
            let sq = kr as usize * 8 + kf as usize;
            if own & bit(sq) != 0 {
                if blocker.is_some() {
                    break;
                }
                blocker = Some(sq);
            } else if occ & bit(sq) != 0 {
                if sliders & bit(sq) != 0 {
                    if let Some(bs) = blocker {
                        pinned |= bit(bs);
                    }
                }
                break;
            }
            kr += dr;
            kf += df;
        }
    }
    pinned
}

pub fn evaluate(b: &Board) -> i32 {
    let mut score = 0i32;

    let mut mat_phase = 0i32;
    for c in 0..2 {
        mat_phase += 4 * b.pieces[c][QUEEN].count_ones() as i32;
        mat_phase += 2 * b.pieces[c][ROOK].count_ones() as i32;
        mat_phase += (b.pieces[c][KNIGHT].count_ones() + b.pieces[c][BISHOP].count_ones()) as i32;
    }
    let p = (24 - mat_phase).clamp(0, 24);
    let mut attacked_by = [0u64; 2];
    let mut king_cover = [0u64; 2];
    let mut atk_force = [0i32; 2];

    for c in 0..2 {
        let mut side = 0i32;
        let own = b.occ[c];
        let ering = KING_ATTACKS[b.kingsq[c ^ 1]] | bit(b.kingsq[c ^ 1]);
        let mut attackers = 0;
        let pinned = pinned_mask(b, c);
        let enemy_mm = b.pieces[c ^ 1][KNIGHT]
            | b.pieces[c ^ 1][BISHOP]
            | b.pieces[c ^ 1][ROOK]
            | b.pieces[c ^ 1][QUEEN]
            | b.pieces[c ^ 1][KING];

        for pc in 0..5 {
            let mut bb = b.pieces[c][pc];
            while bb != 0 {
                let sq = bb.trailing_zeros() as usize;
                bb &= bb - 1;
                let mapped = if c == BLACK { sq } else { sq ^ 56 };
                let ps = (PST[pc][mapped] * (24 - p) + PST_EG[pc][mapped] * p) / 24;
                side += PIECE_VALUES[pc] + ps;
                if pc == KNIGHT || pc == BISHOP || pc == ROOK || pc == QUEEN {
                    let attacks = match pc {
                        KNIGHT => KNIGHT_ATTACKS[sq],
                        BISHOP => bishop_attacks(b.all, sq),
                        ROOK => rook_attacks(b.all, sq),
                        _ => queen_attacks(b.all, sq),
                    };
                    attacked_by[c] |= attacks;
                    let pin = pinned & bit(sq) != 0;
                    if pin {
                        side -= PIN_PEN[pc];
                    }
                    let mob = (attacks & !own).count_ones() as i32;
                    side += MOB_WEIGHT[pc] * if pin { mob / 2 } else { mob };
                    if pin {
                        continue;
                    }
                    let on_ring = attacks & ering;
                    side += KING_ATK_COV * on_ring.count_ones() as i32;
                    if on_ring != 0 {
                        attackers += 1;
                        king_cover[c ^ 1] |= on_ring;
                        atk_force[c ^ 1] += KS_W[pc];
                    }
                    let fork_hits = attacks & enemy_mm;
                    if fork_hits.count_ones() >= 2 {
                        let mut tot = 0i32;
                        let mut h = fork_hits;
                        while h != 0 {
                            let s2 = h.trailing_zeros() as usize;
                            h &= h - 1;
                            tot += PIECE_VALUES[b.piece_on(s2)];
                        }
                        if tot > PIECE_VALUES[pc] {
                            side += FORK_BONUS;
                        }
                    }
                }
            }
        }
        attacked_by[c] |= KING_ATTACKS[b.kingsq[c]];

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
            attacked_by[c] |= PAWN_ATTACKS[c][sq];
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
                side += (EG_PASSED + 8 * progress) * p / 24;
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

        let mapped = if c == BLACK { ks } else { ks ^ 56 };
        let king_pst = (PST_KING[mapped] * (24 - p) + PST_KING_EG[mapped] * p) / 24;
        score += if c == WHITE { side + king_pst } else { -(side + king_pst) };
    }

    for c in 0..2 {
        let their = attacked_by[c ^ 1];
        let mut pen = 0i32;
        for pc in PAWN..=QUEEN {
            let mut bb = b.pieces[c][pc];
            while bb != 0 {
                let sq = bb.trailing_zeros() as usize;
                bb &= bb - 1;
                if their & bit(sq) == 0 {
                    continue;
                }
                let defended = attacked_by[c] & bit(sq) != 0;
                pen += if defended {
                    PIECE_VALUES[pc] / 12
                } else {
                    PIECE_VALUES[pc] / 3
                };
            }
        }
        score += if c == WHITE { -pen } else { pen };
    }

    for c in 0..2 {
        let cover = king_cover[c].count_ones() as i32;
        let force = atk_force[c];
        let serious = (cover >= 4 && force >= 70) || (cover >= 3 && force >= 120);
        if !serious {
            continue;
        }
        let (kf, kr) = (file_of(b.kingsq[c]), rank_of(b.kingsq[c]));
        let ahead = if c == WHITE { kr + 1 } else { kr.saturating_sub(1) };
        let mut shield = 0;
        if ahead < 8 {
            for df in -1i32..=1i32 {
                let sf = kf as i32 + df;
                if (0..8).contains(&sf) && b.pieces[c][PAWN] & bit(ahead * 8 + sf as usize) != 0 {
                    shield += 1;
                }
            }
        }
        let pen = ((force * cover * (60 - 14 * shield.max(0))) / 12) * p / 24;
        let pen = pen.min(850);
        score += if c == WHITE { -pen } else { pen };
    }

    let cd = ((file_of(b.kingsq[0]) as i32 - file_of(b.kingsq[1]) as i32).abs())
            .max((rank_of(b.kingsq[0]) as i32 - rank_of(b.kingsq[1]) as i32).abs());
        score += (7 - cd) * KING_PROX * p / 24;

    if b.side == WHITE {
        score + 12
    } else {
        -score + 12
    }
}