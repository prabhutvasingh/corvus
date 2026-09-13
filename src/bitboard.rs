pub const WHITE: usize = 0;
pub const BLACK: usize = 1;

pub const PAWN: usize = 0;
pub const KNIGHT: usize = 1;
pub const BISHOP: usize = 2;
pub const ROOK: usize = 3;
pub const QUEEN: usize = 4;
pub const KING: usize = 5;

pub const fn bit(sq: usize) -> u64 {
    1u64 << sq
}

pub const fn file_of(sq: usize) -> usize {
    sq & 7
}

pub const fn rank_of(sq: usize) -> usize {
    sq >> 3
}

pub const FILE_A: u64 = 0x0101_0101_0101_0101;
pub const FILE_B: u64 = 0x0202_0202_0202_0202;
pub const FILE_C: u64 = 0x0404_0404_0404_0404;
pub const FILE_D: u64 = 0x0808_0808_0808_0808;
pub const FILE_E: u64 = 0x1010_1010_1010_1010;
pub const FILE_F: u64 = 0x2020_2020_2020_2020;
pub const FILE_G: u64 = 0x4040_4040_4040_4040;
pub const FILE_H: u64 = 0x8080_8080_8080_8080;

pub const NOT_A: u64 = !FILE_A;
pub const NOT_H: u64 = !FILE_H;

pub const FILE_MASK: [u64; 8] = [
    FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H,
];

pub const RANK_1: u64 = 0x0000_0000_0000_00FF;
pub const RANK_2: u64 = 0x0000_0000_0000_FF00;
pub const RANK_3: u64 = 0x0000_0000_00FF_0000;
pub const RANK_4: u64 = 0x0000_0000_FF00_0000;
pub const RANK_5: u64 = 0x0000_00FF_0000_0000;
pub const RANK_6: u64 = 0x0000_FF00_0000_0000;
pub const RANK_7: u64 = 0x00FF_0000_0000_0000;
pub const RANK_8: u64 = 0xFF00_0000_0000_0000;

pub const RANK_MASK: [u64; 8] = [
    RANK_1, RANK_2, RANK_3, RANK_4, RANK_5, RANK_6, RANK_7, RANK_8,
];

pub const SQUARE_NAMES: [&str; 64] = {
    let mut a = [""; 64];
    let mut i = 0;
    while i < 64 {
        let f = i & 7;
        let r = i >> 3;
        a[i] = match (f, r) {
            (0, 0) => "a1",
            (1, 0) => "b1",
            (2, 0) => "c1",
            (3, 0) => "d1",
            (4, 0) => "e1",
            (5, 0) => "f1",
            (6, 0) => "g1",
            (7, 0) => "h1",
            (0, 1) => "a2",
            (1, 1) => "b2",
            (2, 1) => "c2",
            (3, 1) => "d2",
            (4, 1) => "e2",
            (5, 1) => "f2",
            (6, 1) => "g2",
            (7, 1) => "h2",
            (0, 2) => "a3",
            (1, 2) => "b3",
            (2, 2) => "c3",
            (3, 2) => "d3",
            (4, 2) => "e3",
            (5, 2) => "f3",
            (6, 2) => "g3",
            (7, 2) => "h3",
            (0, 3) => "a4",
            (1, 3) => "b4",
            (2, 3) => "c4",
            (3, 3) => "d4",
            (4, 3) => "e4",
            (5, 3) => "f4",
            (6, 3) => "g4",
            (7, 3) => "h4",
            (0, 4) => "a5",
            (1, 4) => "b5",
            (2, 4) => "c5",
            (3, 4) => "d5",
            (4, 4) => "e5",
            (5, 4) => "f5",
            (6, 4) => "g5",
            (7, 4) => "h5",
            (0, 5) => "a6",
            (1, 5) => "b6",
            (2, 5) => "c6",
            (3, 5) => "d6",
            (4, 5) => "e6",
            (5, 5) => "f6",
            (6, 5) => "g6",
            (7, 5) => "h6",
            (0, 6) => "a7",
            (1, 6) => "b7",
            (2, 6) => "c7",
            (3, 6) => "d7",
            (4, 6) => "e7",
            (5, 6) => "f7",
            (6, 6) => "g7",
            (7, 6) => "h7",
            (0, 7) => "a8",
            (1, 7) => "b8",
            (2, 7) => "c8",
            (3, 7) => "d8",
            (4, 7) => "e8",
            (5, 7) => "f8",
            (6, 7) => "g8",
            (7, 7) => "h8",
            _ => "??",
        };
        i += 1;
    }
    a
};

pub fn square_name(sq: usize) -> String {
    SQUARE_NAMES[sq].to_string()
}

const DIRS: [(i32, i32); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (-1, -1),
];

const DIRD: [i32; 8] = [8, 9, 1, 7, -8, -9, -1, -7];

const fn build_rays() -> [[u64; 8]; 64] {
    let mut rays = [[0u64; 8]; 64];
    let mut sq = 0;
    while sq < 64 {
        let rf = (sq >> 3) as i32;
        let ff = (sq & 7) as i32;
        let mut d = 0;
        while d < 8 {
            let (dr, df) = DIRS[d];
            let mut cr = rf + dr;
            let mut cf = ff + df;
            let mut m = 0u64;
            while cr >= 0 && cr < 8 && cf >= 0 && cf < 8 {
                m |= 1u64 << (cr * 8 + cf);
                cr += dr;
                cf += df;
            }
            rays[sq][d] = m;
            d += 1;
        }
        sq += 1;
    }
    rays
}

pub const RAY: [[u64; 8]; 64] = build_rays();

const fn build_knight() -> [u64; 64] {
    let mut t = [0u64; 64];
    let mut sq = 0;
    while sq < 64 {
        let rf = (sq >> 3) as i32;
        let ff = (sq & 7) as i32;
        let offs = [-2, -1, 1, 2];
        let mut i = 0;
        while i < 4 {
            let mut j = 0;
            while j < 4 {
                let dr: i32 = offs[i];
                let df: i32 = offs[j];
                if dr.abs() + df.abs() == 3 {
                    let nr = rf + dr;
                    let nf = ff + df;
                    if nr >= 0 && nr < 8 && nf >= 0 && nf < 8 {
                        t[sq] |= 1u64 << (nr * 8 + nf);
                    }
                }
                j += 1;
            }
            i += 1;
        }
        sq += 1;
    }
    t
}

pub const KNIGHT_ATTACKS: [u64; 64] = build_knight();

const fn build_king() -> [u64; 64] {
    let mut t = [0u64; 64];
    let mut sq = 0;
    while sq < 64 {
        let rf = (sq >> 3) as i32;
        let ff = (sq & 7) as i32;
        let mut dr = -1;
        while dr <= 1 {
            let mut df = -1;
            while df <= 1 {
                if dr != 0 || df != 0 {
                    let nr = rf + dr;
                    let nf = ff + df;
                    if nr >= 0 && nr < 8 && nf >= 0 && nf < 8 {
                        t[sq] |= 1u64 << (nr * 8 + nf);
                    }
                }
                df += 1;
            }
            dr += 1;
        }
        sq += 1;
    }
    t
}

pub const KING_ATTACKS: [u64; 64] = build_king();

const fn build_pawn() -> [[u64; 64]; 2] {
    let mut t = [[0u64; 64]; 2];
    let mut sq = 0;
    while sq < 64 {
        let rf = (sq >> 3) as i32;
        let ff = (sq & 7) as i32;
        let mut c = 0;
        while c < 2 {
            let dr = if c == 0 { 1 } else { -1 };
            let mut df = -1;
            while df <= 1 {
                if df != 0 {
                    let nr = rf + dr;
                    let nf = ff + df;
                    if nr >= 0 && nr < 8 && nf >= 0 && nf < 8 {
                        t[c][sq] |= 1u64 << (nr * 8 + nf);
                    }
                }
                df += 2;
            }
            c += 1;
        }
        sq += 1;
    }
    t
}

pub const PAWN_ATTACKS: [[u64; 64]; 2] = build_pawn();

fn slider_attack(occ: u64, sq: usize, dirs_list: &[usize]) -> u64 {
    let mut att = 0;
    for &d in dirs_list {
        let ray = RAY[sq][d];
        let blockers = ray & occ;
        if blockers == 0 {
            att |= ray;
        } else if DIRD[d] > 0 {
            let first = blockers & blockers.wrapping_neg();
            att |= ray & (first | (first - 1));
        } else {
            let last = 1u64 << (63 - blockers.leading_zeros());
            att |= ray & !(last - 1);
        }
    }
    att
}

pub fn rook_attacks(occ: u64, sq: usize) -> u64 {
    slider_attack(occ, sq, &[0, 2, 4, 6])
}

pub fn bishop_attacks(occ: u64, sq: usize) -> u64 {
    slider_attack(occ, sq, &[1, 3, 5, 7])
}

pub fn queen_attacks(occ: u64, sq: usize) -> u64 {
    rook_attacks(occ, sq) | bishop_attacks(occ, sq)
}

pub fn pawn_attacks(color: usize, sq: usize) -> u64 {
    PAWN_ATTACKS[color][sq]
}