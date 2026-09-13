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
use crate::eval::{evaluate, MATE, PIECE_VALUES};
use crate::movegen::{generate_captures, generate_legal};
use crate::uci::outln;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub const INF: i32 = MATE + 1000;
pub const DRAW: i32 = 0;

const MAX_PLY: usize = 128;
const TT_BOUND_NONE: u8 = 0;
const TT_BOUND_LOWER: u8 = 1;
const TT_BOUND_UPPER: u8 = 2;
const TT_BOUND_EXACT: u8 = 3;

#[derive(Clone, Copy)]
struct TTEntry {
    key: u64,
    data: u64,
}

fn pack_data(mov: u32, bound: u8, depth: i32, score: i32) -> u64 {
    (mov as u64)
        | ((bound as u64) << 24)
        | (((depth.min(0xff)) as u64) << 26)
        | (((score as i64) as u64) << 34)
}

pub struct TT {
    entries: Vec<TTEntry>,
    mask: usize,
}

impl TT {
    pub fn new(size_pow: u32) -> TT {
        let n = 1usize << size_pow;
        TT {
            entries: vec![TTEntry { key: 0, data: 0 }; n],
            mask: n - 1,
        }
    }

    pub fn clear(&mut self) {
        for e in self.entries.iter_mut() {
            e.key = 0;
            e.data = 0;
        }
    }

    pub fn probe(&self, key: u64) -> Option<(u32, i32, i32, u8)> {
        let e = self.entries[(key as usize) & self.mask];
        if e.key != key {
            return None;
        }
        let mov = (e.data & 0xFFFFFF) as u32;
        let bound = ((e.data >> 24) & 0x3) as u8;
        let depth = ((e.data >> 26) & 0xFF) as i32;
        let score = ((((e.data >> 34) as i64) << 34) >> 34) as i32;
        Some((mov, depth, score, bound))
    }

    pub fn store(&mut self, key: u64, mov: u32, depth: i32, score: i32, bound: u8) {
        if bound == TT_BOUND_NONE {
            return;
        }
        let idx = (key as usize) & self.mask;
        self.entries[idx] = TTEntry {
            key,
            data: pack_data(mov, bound, depth, score),
        };
    }
}

#[derive(Clone, Copy)]
pub struct SearchLimits {
    pub depth: u32,
    pub nodes: u64,
    pub movetime: u64,
    pub wtime: u64,
    pub btime: u64,
    pub winc: u64,
    pub binc: u64,
    pub infinite: bool,
}

impl Default for SearchLimits {
    fn default() -> Self {
        SearchLimits {
            depth: 0,
            nodes: 0,
            movetime: 0,
            wtime: 0,
            btime: 0,
            winc: 0,
            binc: 0,
            infinite: false,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SearchResult {
    pub best: Option<Move>,
    pub score: i32,
    pub nodes: u64,
    pub depth: u32,
}

pub struct Searcher {
    pub board: Board,
    pub tt: TT,
    pub killers: [[Move; 2]; MAX_PLY],
    pub history: [[i32; 4096]; 2],
    pub nodes: u64,
    pub stopped: bool,
    pub quiet: bool,
    pub abort: Arc<AtomicBool>,
    pub deadline: Option<Instant>,
    pub nodes_limit: u64,
    pub key_stack: Vec<u64>,
    pub irr_boundary: Vec<bool>,
    pub null_ep: Vec<Option<usize>>,
}

impl Searcher {
    pub fn new(board: Board) -> Searcher {
        Searcher {
            board,
            tt: TT::new(20),
            killers: [[Move::normal(0, 0, 0, NO_PIECE); 2]; MAX_PLY],
            history: [[0; 4096]; 2],
            nodes: 0,
            stopped: false,
            quiet: false,
            abort: Arc::new(AtomicBool::new(false)),
            deadline: None,
            nodes_limit: u64::MAX,
            key_stack: Vec::new(),
            irr_boundary: Vec::new(),
            null_ep: Vec::new(),
        }
    }

    fn deadline(limits: &SearchLimits, side: usize) -> Option<Instant> {
        if limits.movetime > 0 {
            return Some(Instant::now() + Duration::from_millis(limits.movetime));
        }
        if limits.infinite {
            return None;
        }
        if limits.wtime == 0 && limits.btime == 0 {
            return None;
        }
        let (t, inc) = if side == WHITE {
            (limits.wtime, limits.winc)
        } else {
            (limits.btime, limits.binc)
        };
        if t == 0 {
            return None;
        }
        let budget = (t / 32 + inc * 3 / 4).min(t / 4);
        Some(Instant::now() + Duration::from_millis(budget))
    }

    fn make(&mut self, m: Move) {
        let irr = m.piece == PAWN || m.captured != NO_PIECE;
        self.board.make(m);
        self.key_stack.push(self.board.key);
        self.irr_boundary.push(irr);
    }

    fn unmake(&mut self, m: Move) {
        self.key_stack.pop();
        self.irr_boundary.pop();
        self.board.unmake(m);
    }

    fn make_null(&mut self) {
        self.null_ep.push(self.board.ep);
        connected_to_board::zobrist_ep_clear(&mut self.board);
        self.board.side ^= 1;
        connected_to_board::zobrist_side(&mut self.board);
    }

    fn unmake_null(&mut self) {
        connected_to_board::zobrist_side(&mut self.board);
        self.board.side ^= 1;
        connected_to_board::zobrist_ep_restore(&mut self.board, self.null_ep.pop());
    }

    fn poll_stop(&mut self) {
        if self.stopped {
            return;
        }
        if self.nodes >= self.nodes_limit {
            self.stopped = true;
            return;
        }
        if self.abort.load(Ordering::Relaxed) {
            self.stopped = true;
            return;
        }
        if let Some(d) = self.deadline {
            if Instant::now() >= d {
                self.stopped = true;
            }
        }
    }

    fn rep_occurred(&self) -> bool {
        let n = self.key_stack.len();
        if n == 0 {
            return false;
        }
        let mut i = n;
        while i > 0 {
            i -= 1;
            if self.irr_boundary[i] {
                break;
            }
        }
        let mut j = n - 1;
        while j > i {
            j -= 1;
            if self.key_stack[j] == self.board.key {
                return true;
            }
        }
        false
    }

    fn iter_reset(&mut self) {
        for k in self.killers.iter_mut() {
            k[0] = Move::normal(0, 0, 0, NO_PIECE);
            k[1] = Move::normal(0, 0, 0, NO_PIECE);
        }
        self.key_stack.clear();
        self.irr_boundary.clear();
        self.null_ep.clear();
    }

    fn order_moves(
        &self,
        side: usize,
        moves: Vec<Move>,
        tt_mov: Option<u32>,
        ply: usize,
    ) -> Vec<(Move, i32)> {
        let p = ply.min(MAX_PLY - 1);
        let mut scored: Vec<(Move, i32)> = moves
            .into_iter()
            .map(|m| {
                let mut s = 0i32;
                if let Some(tm) = tt_mov {
                    if tm == m.pack() {
                        s += 1_000_000;
                    }
                }
                if m.promo != NO_PIECE {
                    s += 950_000;
                    if m.captured != NO_PIECE {
                        s += 10 * PIECE_VALUES[m.captured] - PIECE_VALUES[m.piece];
                    }
                } else if m.captured != NO_PIECE {
                    s += 10_000 + 10 * PIECE_VALUES[m.captured] - PIECE_VALUES[m.piece];
                } else if self.killers[p][0] == m {
                    s += 900_000;
                } else if self.killers[p][1] == m {
                    s += 890_000;
                } else {
                    s += self.history[side][m.from * 64 + m.to] / 2;
                }
                (m, s)
            })
            .collect();
        scored.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        scored
    }

    fn store_killer(&mut self, m: Move, ply: usize, side: usize) {
        if m.captured != NO_PIECE || m.promo != NO_PIECE {
            return;
        }
        let p = ply.min(MAX_PLY - 1);
        if self.killers[p][0] != m {
            self.killers[p][1] = self.killers[p][0];
            self.killers[p][0] = m;
        }
        let ks = m.from * 64 + m.to;
        let h = &mut self.history[side][ks];
        *h += 64;
        if *h > 100_000 {
            *h = 100_000;
        }
    }

    pub fn think(&mut self, limits: SearchLimits) -> SearchResult {
        self.deadline = Searcher::deadline(&limits, self.board.side);
        self.nodes_limit = if limits.nodes > 0 { limits.nodes } else { u64::MAX };
        let max_depth = if limits.depth > 0 { limits.depth as usize } else { 128 };
        let start = Instant::now();

        let root_moves = generate_legal(&mut self.board);
        if root_moves.is_empty() {
            return SearchResult {
                best: None,
                score: if self.board.in_check(self.board.side) {
                    -MATE + 1
                } else {
                    DRAW
                },
                nodes: 0,
                depth: 0,
            };
        }

        if let Some(bm) = crate::book::probe(&mut self.board, &root_moves) {
            let sc = evaluate(&self.board);
            if !self.quiet {
                outln(format!("info depth 1 score cp {} pv {}", sc, bm.uci()));
            }
            return SearchResult {
                best: Some(bm),
                score: if self.board.side == BLACK { -sc } else { sc },
                nodes: 0,
                depth: 0,
            };
        }

let mut best: Option<Move> = None;
        let mut score = 0i32;
        let mut last_depth = 0u32;
        let mut depth = 1usize;
        let mut prev = 0i32;

        loop {
            if self.stopped {
                break;
            }
            self.iter_reset();

            let prev_best = best;
            let (mv, sc) = if depth >= 5 && prev > -900 && prev < 900 {
                let mut delta = 26i32;
                let mut out: Option<(Move, i32)> = None;
                while out.is_none() {
                    let alpha = prev - delta;
                    let beta = prev + delta;
                    let (mm, ss) = self.search_root(depth as i32, alpha, beta);
                    if self.stopped || (ss > alpha && ss < beta) {
                        out = Some((mm, ss));
                    } else {
                        delta += delta / 2 + 1;
                        if delta > 300 {
                            out = Some(self.search_root(depth as i32, -INF, INF));
                        }
                    }
                }
                out.unwrap()
            } else {
                self.search_root(depth as i32, -INF, INF)
            };
            if self.stopped {
                break;
            }
            prev = sc;
            best = Some(mv);
            score = if self.board.side == BLACK { -sc } else { sc };
            last_depth = depth as u32;

            let elapsed = start.elapsed().as_millis() as u64;
            let nps = if elapsed > 0 {
                self.nodes * 1000 / elapsed
            } else {
                self.nodes
            };
            let pv = self.pv_line(depth);
            let pv_str: Vec<String> = pv.iter().map(|m| m.uci()).collect();
            if !self.quiet {
                outln(format!(
                    "info depth {} {} nodes {} nps {} time {} pv {}",
                    depth,
                    score_string(score),
                    self.nodes,
                    nps,
                    elapsed,
                    pv_str.join(" ")
                ));
            }

            if depth >= max_depth {
                break;
            }
            if let Some(d) = self.deadline {
                if prev_best == Some(mv) && depth >= 6 {
                    let total = d.duration_since(start).as_millis() as u64;
                    if elapsed >= total * 3 / 5 {
                        break;
                    }
                }
            }
            let over_time = self.deadline.map_or(false, |d| Instant::now() >= d);
            if over_time {
                break;
            }
            depth += 1;
        }

        if best.is_none() {
            if root_moves.is_empty() {
                best = None;
            } else {
                best = Some(root_moves[0]);
                score = DRAW;
            }
        }

        SearchResult {
            best,
            score,
            nodes: self.nodes,
            depth: last_depth,
        }
    }

    fn search_root(&mut self, depth: i32, mut alpha: i32, beta: i32) -> (Move, i32) {
        let moves = generate_legal(&mut self.board);
        let tt_mov = self.tt.probe(self.board.key).map(|e| e.0);
        let ordered = self.order_moves(self.board.side, moves, tt_mov, 0);

        let mut best = -INF;
        let mut best_move = ordered[0].0;
        let mut fail_high = false;

        let mut first = true;
        for &(m, _) in ordered.iter() {
            self.make(m);
            if self.stopped {
                self.unmake(m);
                break;
            }
            let v = if first {
                -self.negamax(depth - 1, -beta, -alpha, 1, true)
            } else {
                let mut v = -self.negamax(depth - 1, -alpha - 1, -alpha, 1, true);
                if v > alpha && v < beta {
                    v = -self.negamax(depth - 1, -beta, -alpha, 1, true);
                }
                v
            };
            self.unmake(m);
            first = false;
            if self.stopped {
                break;
            }
            if v > best {
                best = v;
                best_move = m;
            }
            if best > alpha {
                alpha = best;
            }
            if alpha >= beta {
                fail_high = true;
                break;
            }
        }
        if !self.stopped {
            self.tt.store(
                self.board.key,
                best_move.pack(),
                depth,
                probe_score(best, 0),
                if fail_high { TT_BOUND_LOWER } else { TT_BOUND_EXACT },
            );
        }
        (best_move, best)
    }

    fn negamax(&mut self, depth: i32, mut alpha: i32, beta: i32, ply: usize, allow_null: bool) -> i32 {
        self.nodes += 1;
        if self.nodes & 1023 == 0 {
            self.poll_stop();
        }
        if self.stopped {
            return 0;
        }

        if ply > 0 {
            if self.board.half >= 100 || self.rep_occurred() {
                return DRAW;
            }
        }

        let in_check = self.board.in_check(self.board.side);
        let key = self.board.key;

        let tt = self.tt.probe(key);
        if let Some((_tmov, tdepth, tscore, tbound)) = tt {
            if tdepth >= depth && depth > 0 && tscore > -MATE && tscore < MATE {
                let sc = unprobe_score(tscore, ply);
                if tbound == TT_BOUND_EXACT {
                    return sc;
                }
                if tbound == TT_BOUND_LOWER && sc >= beta {
                    return sc;
                }
                if tbound == TT_BOUND_UPPER && sc <= alpha {
                    return sc;
                }
            }
        }

        let d = if in_check { depth + 1 } else { depth };
        if d <= 0 {
            return self.quiescence(alpha, beta, ply);
        }

        let dd = if depth >= 3 && tt.is_none() { d - 1 } else { d };

        let moves = generate_legal(&mut self.board);
        if moves.is_empty() {
            return if in_check { -(MATE - ply as i32) } else { DRAW };
        }

        if !in_check && alpha + 1 == beta && d <= 6 {
            let stand = evaluate(&self.board);
            if stand >= beta + 60 + 60 * d && stand < MATE - 500 && stand > -MATE + 500 {
                return stand;
            }
        }

        let stm = self.board.side;
        let tt_mov = tt.map(|e| e.0);
        let n = moves.len();
        let ordered = self.order_moves(stm, moves, tt_mov, ply);

        if allow_null && depth >= 3 && !in_check && n >= 2 && self.has_null_material(stm) {
            self.make_null();
            let v = -self.negamax(depth - 3, -beta, -beta + 1, ply + 1, false);
            self.unmake_null();
            if self.stopped {
                return 0;
            }
            if v >= beta {
                return beta;
            }
        }

        let alpha0 = alpha;
        let mut best = -INF;
        let mut best_move = ordered[0].0;
        let mut searched = 0usize;

        let stand = if !in_check && d <= 3 {
            Some(evaluate(&self.board))
        } else {
            None
        };

        for &(m, _) in ordered.iter() {
            let quiet = m.captured == NO_PIECE && m.promo == NO_PIECE;
            if quiet && d <= 3 && searched >= 2 {
                if let Some(st) = stand {
                    let margin = match d {
                        1 => 60,
                        2 => 130,
                        _ => 180,
                    };
                    if st + margin <= alpha && !self.gives_check(m) {
                        continue;
                    }
                }
            }
            let mut reduction = 0i32;
            if quiet && !in_check && d >= 4 && searched >= 4 {
                reduction = (1 + (searched as i32 / 4).min(4)).min(4);
                if self.gives_check(m) {
                    reduction = 0;
                } else {
                    reduction = reduction.min(d - 1);
                }
            }
            let mut v;
            self.make(m);
            if searched == 0 {
                v = -self.negamax(dd - 1, -beta, -alpha, ply + 1, true);
            } else {
                v = -self.negamax(dd - 1 - reduction, -alpha - 1, -alpha, ply + 1, true);
                if v > alpha && v < beta {
                    v = -self.negamax(dd - 1, -beta, -alpha, ply + 1, true);
                }
            }
            self.unmake(m);
            searched += 1;
            if self.stopped {
                return 0;
            }
            if quiet && v <= alpha0 {
                let h = &mut self.history[stm][m.from * 64 + m.to];
                *h = h.saturating_sub(40);
            }
            if v > best {
                best = v;
                best_move = m;
            }
            if v >= beta {
                if quiet {
                    self.store_killer(m, ply, stm);
                }
                break;
            }
            if v > alpha {
                alpha = v;
            }
        }

        let bound = if best <= alpha0 {
            TT_BOUND_UPPER
        } else if best >= beta {
            TT_BOUND_LOWER
        } else {
            TT_BOUND_EXACT
        };
        if !self.stopped {
            let to_score = probe_score(best, ply);
            self.tt.store(key, best_move.pack(), depth, to_score, bound);
        }
        best
    }

    fn quiescence(&mut self, mut alpha: i32, beta: i32, ply: usize) -> i32 {
        if ply >= MAX_PLY {
            return evaluate(&self.board);
        }
        self.nodes += 1;
        if self.nodes & 1023 == 0 {
            self.poll_stop();
        }
        if self.stopped {
            return 0;
        }

        let in_check = self.board.in_check(self.board.side);

        let stand = evaluate(&self.board);
        if !in_check {
            if stand >= beta {
                return stand;
            }
            if stand > alpha {
                alpha = stand;
            }
        }

        let captures = if in_check {
            generate_legal(&mut self.board)
        } else {
            generate_captures(&mut self.board)
        };
        if captures.is_empty() {
            return if in_check { -(MATE - ply as i32) } else { alpha };
        }

        let tt_mov = self.tt.probe(self.board.key).map(|e| e.0);
        let ordered = self.order_moves(self.board.side, captures, tt_mov, ply);

        for &(m, _) in ordered.iter() {
            if !in_check {
                let cap_v = if m.captured != NO_PIECE && m.promo == NO_PIECE {
                    PIECE_VALUES[m.captured]
                } else {
                    0
                };
                let mut skip = stand + cap_v + 200 <= alpha;
                if !skip
                    && m.captured != NO_PIECE
                    && m.promo == NO_PIECE
                    && crate::movegen::see(&self.board, m.to, self.board.side) < 0
                {
                    skip = true;
                }
                if skip && !self.gives_check(m) {
                    continue;
                }
            }
            self.make(m);
            let v = -self.quiescence(-beta, -alpha, ply + 1);
            self.unmake(m);
            if self.stopped {
                return 0;
            }
            if v >= beta {
                return v;
            }
            if v > alpha {
                alpha = v;
            }
        }
        alpha
    }

    fn has_null_material(&self, side: usize) -> bool {
        let major = self.board.pieces[side][QUEEN] | self.board.pieces[side][ROOK];
        let minors = self.board.pieces[side][BISHOP] | self.board.pieces[side][KNIGHT];
        major != 0 || minors.count_ones() >= 2
    }

    fn gives_check(&mut self, m: Move) -> bool {
        self.make(m);
        let c = self.board.in_check(self.board.side);
        self.unmake(m);
        c
    }

    fn pv_line(&mut self, max: usize) -> Vec<Move> {
        let mut out = Vec::new();
        let mut made = Vec::new();
        for _ in 0..max {
            if self.stopped {
                break;
            }
            let key = self.board.key;
            let tm = match self.tt.probe(key) {
                Some((mov, _, _, _)) => {
                    let m = Move::unpack(mov);
                    if self.is_legal(m) {
                        Some(m)
                    } else {
                        None
                    }
                }
                None => None,
            };
            match tm {
                Some(m) => {
                    self.make(m);
                    out.push(m);
                    made.push(m);
                }
                None => break,
            }
        }
        for m in made.iter().rev() {
            self.unmake(*m);
        }
        out
    }

    fn is_legal(&mut self, m: Move) -> bool {
        generate_legal(&mut self.board).contains(&m)
    }
}

fn probe_score(score: i32, ply: usize) -> i32 {
    if score > MATE - 200 {
        score + ply as i32
    } else if score < -(MATE - 200) {
        score - ply as i32
    } else {
        score
    }
}

fn unprobe_score(score: i32, ply: usize) -> i32 {
    if score > MATE - 200 {
        score - ply as i32
    } else if score < -(MATE - 200) {
        score + ply as i32
    } else {
        score
    }
}

fn score_string(score: i32) -> String {
    if score > MATE - 200 {
        let plies = MATE - score;
        let moves = (plies + 1) / 2;
        format!("score mate {}", moves)
    } else if score < -(MATE - 200) {
        let plies = MATE + score;
        let moves = (plies + 1) / 2;
        format!("score mate -{}", moves)
    } else {
        format!("score cp {}", score)
    }
}

mod connected_to_board {
    use super::*;

    pub fn zobrist_side(board: &mut Board) {
        let lhs = crate::board::side_key();
        board.key ^= lhs;
    }

    pub fn zobrist_ep_clear(board: &mut Board) {
        if let Some(e) = board.ep {
            board.key ^= crate::board::ep_key(e);
        }
        board.ep = None;
    }

    pub fn zobrist_ep_restore(board: &mut Board, prev: Option<Option<usize>>) {
        let prev = prev.expect("null ep stack");
        if let Some(e) = prev {
            board.key ^= crate::board::ep_key(e);
        }
        board.ep = prev;
    }
}