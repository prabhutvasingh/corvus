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

pub const NO_PIECE: usize = 6;

pub const M_NORMAL: u8 = 0;
pub const M_CASTLE: u8 = 1;
pub const M_EP: u8 = 2;
pub const M_PROMO: u8 = 4;

pub const WK: u16 = 1;
pub const WQ: u16 = 2;
pub const BK: u16 = 4;
pub const BQ: u16 = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move {
    pub from: usize,
    pub to: usize,
    pub piece: usize,
    pub captured: usize,
    pub promo: usize,
    pub flags: u8,
}

impl Move {
    pub fn normal(from: usize, to: usize, piece: usize, captured: usize) -> Move {
        Move {
            from,
            to,
            piece,
            captured,
            promo: NO_PIECE,
            flags: M_NORMAL,
        }
    }

    pub fn castle(from: usize, to: usize) -> Move {
        Move {
            from,
            to,
            piece: KING,
            captured: NO_PIECE,
            promo: NO_PIECE,
            flags: M_CASTLE,
        }
    }

    pub fn ep(from: usize, to: usize) -> Move {
        Move {
            from,
            to,
            piece: PAWN,
            captured: PAWN,
            promo: NO_PIECE,
            flags: M_EP,
        }
    }

    pub fn promo(from: usize, to: usize, promo_piece: usize) -> Move {
        Move {
            from,
            to,
            piece: PAWN,
            captured: NO_PIECE,
            promo: promo_piece,
            flags: M_PROMO,
        }
    }

    pub fn pack(&self) -> u32 {
        (self.from as u32)
            | ((self.to as u32) << 6)
            | ((self.piece as u32) << 12)
            | ((self.captured as u32) << 15)
            | ((self.promo as u32) << 18)
            | ((self.flags as u32) << 21)
    }

    pub fn unpack(v: u32) -> Move {
        Move {
            from: (v & 63) as usize,
            to: ((v >> 6) & 63) as usize,
            piece: ((v >> 12) & 7) as usize,
            captured: ((v >> 15) & 7) as usize,
            promo: ((v >> 18) & 7) as usize,
            flags: ((v >> 21) & 7) as u8,
        }
    }

    pub fn uci(&self) -> String {
        let promo_char = match self.promo {
            KNIGHT => "n",
            BISHOP => "b",
            ROOK => "r",
            QUEEN => "q",
            _ => "",
        };
        let mut s = String::with_capacity(6);
        s.push_str(SQUARE_NAMES[self.from]);
        s.push_str(SQUARE_NAMES[self.to]);
        s.push_str(promo_char);
        s
    }

    pub fn is_tactical(&self) -> bool {
        self.captured != NO_PIECE || self.promo != NO_PIECE
    }
}

pub const fn splitmix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

fn piece_key(sq: usize, color: usize, piece: usize) -> u64 {
    static Z: [u64; 768] = {
        let mut a = [0u64; 768];
        let mut i = 0;
        while i < 768 {
            a[i] = splitmix(0xAABBCCDD11223344 ^ (i as u64).wrapping_mul(0x9E3779B97F4A7C15));
            i += 1;
        }
        a
    };
    Z[sq * 12 + color * 6 + piece]
}

fn castle_key(c: u16) -> u64 {
    static Z: [u64; 16] = {
        let mut a = [0u64; 16];
        let mut i = 0;
        while i < 16 {
            a[i] = splitmix(0xDEADBEEFCAFEBABE ^ (i as u64).wrapping_mul(0xC2B2AE3D27D4EB4F));
            i += 1;
        }
        a
    };
    Z[(c & 15) as usize]
}

pub fn ep_key(sq: usize) -> u64 {
    static Z: [u64; 8] = {
        let mut a = [0u64; 8];
        let mut i = 0;
        while i < 8 {
            a[i] = splitmix(0xFEEDFACE5EED5EED ^ (i as u64).wrapping_mul(0x94D049BB133111EB));
            i += 1;
        }
        a
    };
    Z[file_of(sq)]
}

pub fn side_key() -> u64 {
    splitmix(0xB0AB_BABE_F00D_CAFE)
}

pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Clone, Debug)]
pub struct Undo {
    pub castle: u16,
    pub ep: Option<usize>,
    pub half: u32,
    pub fullmove: u32,
    pub key: u64,
    pub cap_pt: usize,
    pub cap_sq: usize,
}

#[derive(Clone)]
pub struct Board {
    pub pieces: [[u64; 6]; 2],
    pub occ: [u64; 2],
    pub all: u64,
    pub side: usize,
    pub kingsq: [usize; 2],
    pub castle: u16,
    pub ep: Option<usize>,
    pub half: u32,
    pub fullmove: u32,
    pub key: u64,
    pub history: Vec<Undo>,
}

impl Board {
    pub fn empty() -> Board {
        Board {
            pieces: [[0; 6]; 2],
            occ: [0; 2],
            all: 0,
            side: WHITE,
            kingsq: [0; 2],
            castle: 0,
            ep: None,
            half: 0,
            fullmove: 1,
            key: 0,
            history: Vec::new(),
        }
    }

    pub fn startpos() -> Board {
        Board::from_fen(START_FEN).expect("valid start position")
    }

    pub fn from_fen(fen: &str) -> Result<Board, String> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() < 4 {
            return Err(format!("too few FEN fields: '{}'", fen));
        }

        let mut b = Board::empty();

        let rows: Vec<&str> = parts[0].split('/').collect();
        if rows.len() != 8 {
            return Err("expected 8 ranks in piece placement".to_string());
        }
        for (ri, row) in rows.iter().enumerate() {
            let rank = 7 - ri;
            let mut file = 0usize;
            for ch in row.chars() {
                if ch.is_digit(10) {
                    file += ch.to_digit(10).unwrap() as usize;
                } else {
                    if file >= 8 {
                        return Err("rank overflow".to_string());
                    }
                    let sq = rank * 8 + file;
                    let color = if ch.is_ascii_uppercase() { WHITE } else { BLACK };
                    let piece = match ch.to_ascii_lowercase() {
                        'p' => PAWN,
                        'n' => KNIGHT,
                        'b' => BISHOP,
                        'r' => ROOK,
                        'q' => QUEEN,
                        'k' => KING,
                        _ => return Err(format!("bad piece char '{}'", ch)),
                    };
                    b.pieces[color][piece] |= bit(sq);
                    b.occ[color] |= bit(sq);
                    b.all |= bit(sq);
                    file += 1;
                }
            }
            if file != 8 {
                return Err(format!("rank '{}' does not fill a full rank", row));
            }
        }

        b.side = match parts[1] {
            "w" => WHITE,
            "b" => BLACK,
            _ => return Err("side to move must be 'w' or 'b'".to_string()),
        };

        let mut castle = 0u16;
        for ch in parts[2].chars() {
            match ch {
                'K' => castle |= WK,
                'Q' => castle |= WQ,
                'k' => castle |= BK,
                'q' => castle |= BQ,
                '-' => {}
                _ => return Err(format!("bad castling char '{}'", ch)),
            }
        }
        if castle == 0 && parts[2] == "-" {
            castle = 0;
        }
        b.castle = castle;

        b.ep = if parts[3] == "-" {
            None
        } else {
            let bytes = parts[3].as_bytes();
            if bytes.len() != 2 {
                return Err("bad en passant square".to_string());
            }
            let f = bytes[0] as usize;
            if !(b'a' as usize..=b'h' as usize).contains(&f) {
                return Err("bad en passant file".to_string());
            }
            let r = (bytes[1] as char).to_digit(10).ok_or("bad en passant rank")? as usize;
            if !(1..=8).contains(&r) {
                return Err("bad en passant rank".to_string());
            }
            Some((r - 1) * 8 + (f - b'a' as usize))
        };

        if parts.len() > 4 {
            b.half = parts[4].parse().unwrap_or(0);
        }
        if parts.len() > 5 {
            b.fullmove = parts[5].parse().unwrap_or(1);
        }

        for c in 0..2 {
            let k = b.pieces[c][KING];
            if k != 0 && k.is_power_of_two() {
                b.kingsq[c] = k.trailing_zeros() as usize;
            } else if k != 0 {
                return Err("position has more than one king".to_string());
            }
        }

        b.key = b.compute_key();
        Ok(b)
    }

    pub fn compute_key(&self) -> u64 {
        let mut k = 0u64;
        for c in 0..2 {
            for pc in 0..6 {
                let mut bb = self.pieces[c][pc];
                while bb != 0 {
                    let sq = bb.trailing_zeros() as usize;
                    bb &= bb - 1;
                    k ^= piece_key(sq, c, pc);
                }
            }
        }
        k ^= castle_key(self.castle);
        if let Some(e) = self.ep {
            k ^= ep_key(e);
        }
        if self.side == BLACK {
            k ^= side_key();
        }
        k
    }

    pub fn piece_on(&self, sq: usize) -> usize {
        for c in 0..2 {
            for pc in 0..6 {
                if self.pieces[c][pc] & bit(sq) != 0 {
                    return pc;
                }
            }
        }
        NO_PIECE
    }

    pub fn color_on(&self, sq: usize) -> Option<usize> {
        if self.occ[WHITE] & bit(sq) != 0 {
            Some(WHITE)
        } else if self.occ[BLACK] & bit(sq) != 0 {
            Some(BLACK)
        } else {
            None
        }
    }

    pub fn make(&mut self, m: Move) -> bool {
        let us = self.side;
        let them = us ^ 1;
        let from = m.from;
        let to = m.to;
        let pt = m.piece;

        self.history.push(Undo {
            castle: self.castle,
            ep: self.ep,
            half: self.half,
            fullmove: self.fullmove,
            key: self.key,
            cap_pt: NO_PIECE,
            cap_sq: to,
        });

        self.key ^= piece_key(from, us, pt);
        self.pieces[us][pt] &= !bit(from);
        self.occ[us] &= !bit(from);
        self.all &= !bit(from);

        let mut cap_pt = NO_PIECE;
        let mut cap_sq = to;
        if m.flags & M_EP != 0 {
            let esq = if us == WHITE { to - 8 } else { to + 8 };
            if self.pieces[them][PAWN] & bit(esq) != 0 {
                cap_pt = PAWN;
                cap_sq = esq;
            }
        } else if self.occ[them] & bit(to) != 0 {
            cap_pt = self.piece_on(to);
            cap_sq = to;
        }

        if cap_pt != NO_PIECE {
            let h = self.history.len() - 1;
            self.history[h].cap_pt = cap_pt;
            self.history[h].cap_sq = cap_sq;
            self.key ^= piece_key(cap_sq, them, cap_pt);
            self.pieces[them][cap_pt] &= !bit(cap_sq);
            self.occ[them] &= !bit(cap_sq);
            self.all &= !bit(cap_sq);
        }

        if m.flags & M_CASTLE != 0 {
            let (rk_f, rk_t) = if from < to { (from + 3, to - 1) } else { (from - 4, to + 1) };
            self.key ^= piece_key(rk_f, us, ROOK);
            self.key ^= piece_key(rk_t, us, ROOK);
            self.pieces[us][ROOK] = (self.pieces[us][ROOK] & !bit(rk_f)) | bit(rk_t);
            self.occ[us] = (self.occ[us] & !bit(rk_f)) | bit(rk_t);
            self.all = (self.all & !bit(rk_f)) | bit(rk_t);
        }

        let place = if m.flags & M_PROMO != 0 { m.promo } else { pt };
        self.key ^= piece_key(to, us, place);
        self.pieces[us][place] |= bit(to);
        self.occ[us] |= bit(to);
        self.all |= bit(to);

        if pt == KING {
            self.kingsq[us] = to;
        }

        let old_castle = self.castle;
        if pt == KING {
            if us == WHITE {
                self.castle &= !(WK | WQ);
            } else {
                self.castle &= !(BK | BQ);
            }
        }
        if pt == ROOK {
            if from == 7 {
                self.castle &= !WK;
            }
            if from == 0 {
                self.castle &= !WQ;
            }
            if from == 63 {
                self.castle &= !BK;
            }
            if from == 56 {
                self.castle &= !BQ;
            }
        }
        if cap_pt == ROOK {
            if cap_sq == 7 {
                self.castle &= !WK;
            }
            if cap_sq == 0 {
                self.castle &= !WQ;
            }
            if cap_sq == 63 {
                self.castle &= !BK;
            }
            if cap_sq == 56 {
                self.castle &= !BQ;
            }
        }
        if self.castle != old_castle {
            self.key ^= castle_key(old_castle);
            self.key ^= castle_key(self.castle);
        }

        let is_double = pt == PAWN && (to as i32 - from as i32).abs() == 16;
        let new_ep = if is_double {
            Some(if us == WHITE { to - 8 } else { to + 8 })
        } else {
            None
        };
        if let Some(e) = self.ep {
            self.key ^= ep_key(e);
        }
        self.ep = new_ep;
        if let Some(e) = self.ep {
            self.key ^= ep_key(e);
        }

        if pt == PAWN || cap_pt != NO_PIECE {
            self.half = 0;
        } else {
            self.half += 1;
        }

        self.side = them;
        self.key ^= side_key();

        !self.attacked(them, self.kingsq[us])
    }

    pub fn unmake(&mut self, m: Move) {
        let undo = self.history.pop().expect("unmake without a matching make");
        let us = self.side ^ 1;
        let from = m.from;
        let to = m.to;
        let pt = m.piece;

        self.key = undo.key;
        self.castle = undo.castle;
        self.ep = undo.ep;
        self.half = undo.half;
        self.fullmove = undo.fullmove;

        let place = if m.flags & M_PROMO != 0 { m.promo } else { pt };
        self.pieces[us][place] &= !bit(to);
        self.occ[us] &= !bit(to);
        self.all &= !bit(to);

        if m.flags & M_CASTLE != 0 {
            let (rk_f, rk_t) = if from < to { (from + 3, to - 1) } else { (from - 4, to + 1) };
            self.pieces[us][ROOK] = (self.pieces[us][ROOK] & !bit(rk_t)) | bit(rk_f);
            self.occ[us] = (self.occ[us] & !bit(rk_t)) | bit(rk_f);
            self.all = (self.all & !bit(rk_t)) | bit(rk_f);
        }

        if undo.cap_pt != NO_PIECE {
            let them = us ^ 1;
            self.pieces[them][undo.cap_pt] |= bit(undo.cap_sq);
            self.occ[them] |= bit(undo.cap_sq);
            self.all |= bit(undo.cap_sq);
        }

        self.pieces[us][pt] |= bit(from);
        self.occ[us] |= bit(from);
        self.all |= bit(from);

        if pt == KING {
            self.kingsq[us] = from;
        }

        self.side = us;
    }

    pub fn attacked(&self, by: usize, sq: usize) -> bool {
        if self.pieces[by][PAWN] & PAWN_ATTACKS[by ^ 1][sq] != 0 {
            return true;
        }
        if self.pieces[by][KNIGHT] & KNIGHT_ATTACKS[sq] != 0 {
            return true;
        }
        if self.pieces[by][KING] & KING_ATTACKS[sq] != 0 {
            return true;
        }
        let rooks = self.pieces[by][ROOK] | self.pieces[by][QUEEN];
        let bishops = self.pieces[by][BISHOP] | self.pieces[by][QUEEN];
        if rooks & rook_attacks(self.all, sq) != 0 {
            return true;
        }
        if bishops & bishop_attacks(self.all, sq) != 0 {
            return true;
        }
        false
    }

    pub fn in_check(&self, side: usize) -> bool {
        self.attacked(side ^ 1, self.kingsq[side])
    }

    pub fn uci_move(&self, s: &str) -> Option<Move> {
        let b = s.as_bytes();
        if b.len() < 4 {
            return None;
        }
        let f0 = b[0] as usize;
        let r0 = b[1] as usize;
        let f1 = b[2] as usize;
        let r1 = b[3] as usize;
        if f0 < b'a' as usize || f0 > b'h' as usize {
            return None;
        }
        if f1 < b'a' as usize || f1 > b'h' as usize {
            return None;
        }
        if r0 < b'1' as usize || r0 > b'8' as usize {
            return None;
        }
        if r1 < b'1' as usize || r1 > b'8' as usize {
            return None;
        }
        let from = (r0 - b'1' as usize) * 8 + (f0 - b'a' as usize);
        let to = (r1 - b'1' as usize) * 8 + (f1 - b'a' as usize);

        let promo = if b.len() > 4 {
            match b[4] as char {
                'n' => KNIGHT,
                'b' => BISHOP,
                'r' => ROOK,
                'q' => QUEEN,
                _ => return None,
            }
        } else {
            NO_PIECE
        };

        let piece = self.piece_on(from);
        if piece == NO_PIECE {
            return None;
        }

        let flags = if piece == KING && (to as i32 - from as i32) == 2 {
            M_CASTLE
        } else if piece == KING && (from as i32 - to as i32) == 2 {
            M_CASTLE
        } else if piece == PAWN && self.ep == Some(to) {
            M_EP
        } else if promo != NO_PIECE {
            M_PROMO
        } else {
            M_NORMAL
        };

        Some(Move {
            from,
            to,
            piece,
            captured: self.piece_on(to),
            promo,
            flags,
        })
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        for r in (0..8).rev() {
            let mut empty = 0;
            let mut rank = String::new();
            for f in 0..8 {
                let sq = r * 8 + f;
                let pc = self.piece_on(sq);
                if pc == NO_PIECE {
                    empty += 1;
                } else {
                    if empty > 0 {
                        rank.push_str(&empty.to_string());
                        empty = 0;
                    }
                    let c = self.color_on(sq).unwrap();
                    let ch = match pc {
                        PAWN => 'p',
                        KNIGHT => 'n',
                        BISHOP => 'b',
                        ROOK => 'r',
                        QUEEN => 'q',
                        _ => 'k',
                    };
                    if c == WHITE {
                        rank.push(ch.to_ascii_uppercase());
                    } else {
                        rank.push(ch);
                    }
                }
            }
            if empty > 0 {
                rank.push_str(&empty.to_string());
            }
            fen.push_str(&rank);
            if r > 0 {
                fen.push('/');
            }
        }
        fen.push(' ');
        fen.push(if self.side == WHITE { 'w' } else { 'b' });
        fen.push(' ');
        let castle = self.castle;
        if castle == 0 {
            fen.push('-');
        } else {
            if castle & WK != 0 {
                fen.push('K');
            }
            if castle & WQ != 0 {
                fen.push('Q');
            }
            if castle & BK != 0 {
                fen.push('k');
            }
            if castle & BQ != 0 {
                fen.push('q');
            }
        }
        fen.push(' ');
        if let Some(e) = self.ep {
            fen.push_str(SQUARE_NAMES[e]);
        } else {
            fen.push('-');
        }
        fen.push(' ');
        fen.push_str(&self.half.to_string());
        fen.push(' ');
        fen.push_str(&self.fullmove.to_string());
        fen
    }

    pub fn display(&self) -> String {
        let mut s = String::new();
        for r in (0..8).rev() {
            s.push_str(&format!("{} ", r + 1));
            for f in 0..8 {
                let sq = r * 8 + f;
                let mut ch = ' ';
                for c in 0..2 {
                    for pc in 0..6 {
                        if self.pieces[c][pc] & bit(sq) != 0 {
                            ch = match pc {
                                PAWN => 'p',
                                KNIGHT => 'n',
                                BISHOP => 'b',
                                ROOK => 'r',
                                QUEEN => 'q',
                                _ => 'k',
                            };
                            if c == WHITE {
                                ch = ch.to_ascii_uppercase();
                            }
                        }
                    }
                }
                if ch == ' ' {
                    ch = '.';
                }
                s.push(ch);
                s.push(' ');
            }
            s.push('\n');
        }
        s.push_str("  a b c d e f g h\n");
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fen_roundtrip() {
        let b = Board::startpos();
        assert_eq!(b.to_fen(), "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    }

    #[test]
    fn make_unmake_hash_consistency() {
        let mut b = Board::startpos();
        let urandom: Vec<u64> = (0..200)
            .map(|i| splitmix((i as u64).wrapping_mul(0x9E3779B97F4A7C15)))
            .collect();
        for _ in 0..200 {
            let moves = crate::movegen::generate_legal(&mut b);
            if moves.is_empty() {
                break;
            }
            let m = moves[(urandom[moves.len()] as usize) % moves.len()];
            let _ = b.make(m);
            assert_eq!(b.key, b.compute_key(), "hash invariant broken after {}", m.uci());
            b.unmake(m);
            assert_eq!(b.key, b.compute_key(), "hash invariant broken after unmake {}", m.uci());
        }
    }

    #[test]
    fn castling_rights_cleared() {
        let mut b = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
        let m = b.uci_move("e1g1").unwrap();
        let legal = b.make(m);
        assert!(legal);
        assert_eq!(b.castle & (WK | WQ), 0);
        b.unmake(m);
        assert_eq!(b.castle & WK, WK);

        let m = b.uci_move("a1a2").unwrap();
        let _ = b.make(m);
        assert_eq!(b.castle & WQ, 0);
        b.unmake(m);
        assert_eq!(b.castle & WQ, WQ);
    }

    #[test]
    fn en_passant_works() {
        let mut b = Board::from_fen("8/8/8/Pp6/8/8/8/K6k w - b6 0 1").unwrap();
        let m = b.uci_move("a5b6").unwrap();
        let legal = b.make(m);
        assert!(legal);
        assert_eq!(b.piece_on(b5()), NO_PIECE);
        assert_eq!(b.piece_on(b6()), PAWN);
        b.unmake(m);
        assert_eq!(b.piece_on(b5()), PAWN);
    }

    #[test]
    fn promotions_roundtrip() {
        let mut b = Board::from_fen("8/P6k/8/8/8/8/7K/8 w - - 0 1").unwrap();
        let m = b.uci_move("a7a8q").unwrap();
        assert_eq!(m.promo, QUEEN);
        let _ = b.make(m);
        assert_eq!(b.piece_on(a8()), QUEEN);
        assert_eq!(b.piece_on(a7()), NO_PIECE);
        b.unmake(m);
        assert_eq!(b.piece_on(a7()), PAWN);
    }

    fn a8() -> usize { 56 }
    fn a7() -> usize { 48 }
    fn b5() -> usize { 33 }
    fn b6() -> usize { 41 }
}