#!/usr/bin/env bash
# Self-play Elo harness: compare OLD vs NEW engine.
# Usage: ./tools/tune.sh <old-binary> <new-binary> [rounds=40] [st_ms=50]
set -euo pipefail
cd "$(dirname "$0")/.."

OLD="$1"; NEW="$2"; ROUNDS="${3:-40}"; ST="${4:-50}"
LOG="tools/tune_log.csv"
WS=/tmp/tune_workspace; mkdir -p "$WS"

test -x "$OLD" || { echo "not executable: $OLD"; exit 1; }
test -x "$NEW" || { echo "not executable: $NEW"; exit 1; }

# Opening suite: (side) FEN
declare -a OPENINGS=(
  "(startpos) rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
  "(Italian) r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1"
  "(Ruy Lopez) r1bqkbnr/1ppp1ppp/p1n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 1 3"
  "(QGD) rnbqkbnr/ppp2ppp/4p3/3p4/2PP4/2N5/PP2PPPP/R1BQKBNR w KQkq - 0 3"
  "(Slav) rn1qkbnr/ppp1pppp/8/3p4/3P4/2N5/PP2PPPP/R1BQKBNR w KQkq - 1 3"
  "(Sicilian) rnbqkb1r/pp3ppp/3ppn2/2p5/2PP4/2N2N2/PP2PPPP/R1BQKB1R w KQkq - 0 4"
  "(King Indian) r1bq1rk1/pp2bppp/2n1pn2/2pp4/3P1B2/2NBPN2/PPP2PPP/R1BQK2R w KQ - 4 7"
  "(Nimzo) 2rqk2r/1p2bppp/p1n1bn2/2pp4/3P1B2/2NBPN2/PPP2PPP/R1BQ1RK1 w k - 7 11"
)

echo "=== $OLD vs $NEW  rounds=$ROUNDS  st=${ST}ms ==="

run_pair() {
  local side1="$1" side2="$2" fen1="$3" fen2="$4" gnum="$5"
  local pgn="$WS/g${gnum}.pgn"
  local name1 name2 cmd1 cmd2 fen1 fen2
  if [[ "$side1" == "Old" ]]; then
    name1=Old; cmd1="$OLD"; name2=New; cmd2="$NEW"
    fen_used="$fen1"
  else
    name1=New; cmd1="$NEW"; name2=Old; cmd2="$OLD"
    fen_used="$fen2"
  fi
  # Run both engines independently, pipe I/O via coproc for the 2-engine match
  # Simpler: cutechess handles the I/O; we just need valid EPDs.
  # Convert bare FEN → proper EPD with a dummy move by appending "bm a1a1"
  local epd
  epd=$(echo "$fen_used" | sed -E 's/ ([0-9]+) ([0-9]+) *$/; bm a1a1; c0 \1 \2;/')
  echo -e "${epd}" > "$WS/cur.epd"
  cutechess-cli \
    -engine name="$name1" cmd="$cmd1" proto=uci \
    -engine name="$name2" cmd="$cmd2" proto=uci \
    -each st="$ST" \
    -openings file="$WS/cur.epd" order=random -repeat -pgnout "$pgn" \
    -concurrency 1 \
    -games 1 2>&1 | sed 's/^/  /' || true
  echo "$pgn"
}

for i in $(seq 1 "$ROUNDS"); do
  idx=$(( (i-1) % ${#OPENINGS[@]} ))
  side=$(( i % 2 ))
  if [[ "$side" -eq 0 ]]; then
    side1="Old"; side2="New"
  else
    side1="New"; side2="Old"
  fi
  run_pair "$side1" "$side2" "${OPENINGS[$idx]}" "${OPENINGS[$(( (idx+1) % ${#OPENINGS[@]} ))]}" "$i" > /dev/null 2>&1 || true
done

# Score all games
python3 - "$WS" "$OLD" "$NEW" "$ST" "$ROUNDS" "$LOG" <<'EOF'
import re, sys, math, glob, os
ws, old, new, st, rounds, log = sys.argv[1:7]
res = {'Old':[0,0], 'New':[0,0]}
seen = 0
for pgn in sorted(glob.glob(f"{ws}/g*.pgn")):
    data = open(pgn).read()
    if '[Result "' not in data: continue
    white = re.search(r'\[White "(\w+)"\]', data).group(1)
    r = re.search(r'\[Result "([^"]+)"\]', data).group(1)
    if r == '1-0':   res[white][0] += 1; seen += 1
    elif r == '0-1': res['New' if white=='Old' else 'Old'][0] += 1; seen += 1
    elif r == '1/2-1/2': res['Old'][1] += 1; res['New'][1] += 1; seen += 1
o, n = res['Old'], res['New']
tot = seen
score = (o[0] + 0.5*o[1]) / max(tot,1)
elo = 400*math.log10(score/(1-score)) if 0 < score < 1 else (float('inf') if score>=1 else float('-inf'))
z = 1.96
S, N = score, max(tot,1)
h = z*math.sqrt((S*(1-S))/N + (z*z)/(4*N*N)); d = 1 + z*z/N
lo = (S + z*z/(2*N) - h)/d; hi = (S + z*z/(2*N) + h)/d
def e(s): return 400*math.log10(s/(1-s)) if 0<s<1 else (float('inf') if s>=1 else float('-inf'))
print(f"\n  games={tot}  Old={o[0]} New={n[0]} Draws={o[1]}")
print(f"  Old score = {S:.3f}   (95% CI {lo:.3f}..{hi:.3f})")
print(f"  Elo diff (Old - New) = {e(S):+.0f}  CI [{e(lo):+.0f}, {e(hi):+.0f}]")
with open(log,'a') as f:
    f.write(f"{__import__('datetime').datetime.now().isoformat()},{old},{new},{st},{rounds},{tot},{o[0]},{n[0]},{o[1]},{S:.3f},{e(S):+.0f}\n")
EOF
rm -f "$WS"/g*.pgn "$WS"/cur.epd
