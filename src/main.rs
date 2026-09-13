use chess_engine::board::Board;
use chess_engine::perft;
use chess_engine::uci;

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 {
        match args[1].as_str() {
            "perft" => {
                let depth: u32 = args
                    .get(2)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(5);
                let mut b = Board::startpos();
                let n = perft::perft(&mut b, depth);
                println!("perft depth {}: {}", depth, n);
                return;
            }
            "divide" => {
                let depth: u32 = args
                    .get(2)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(2);
                let mut b = Board::startpos();
                perft::divide(&mut b, depth);
                return;
            }
            "perftfen" => {
                let depth: u32 = args.pop().and_then(|s| s.parse().ok()).unwrap_or(4);
                let fen = args[2..].join(" ");
                let mut b = match Board::from_fen(&fen) {
                    Ok(b) => b,
                    Err(e) => {
                        println!("bad fen: {}", e);
                        return;
                    }
                };
                let n = perft::perft(&mut b, depth);
                println!("perft depth {}: {}", depth, n);
                return;
            }
            _ => {}
        }
    }
    uci::run();
}