use std::{io::Stdin, mem, sync::mpsc::Sender};

use chess::{
    game::{self, ChannelPlayer, ChessGame},
    logic::Move,
};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

struct Uci {
    white_channel: Sender<Move>,
    black_channel: Sender<Move>,
    game: ChessGame,
    stdin: Stdin,
}

impl Uci {
    fn new() -> Self {
        let (white_channel, white_player) = ChannelPlayer::new();
        let (black_channel, black_player) = ChannelPlayer::new();

        let game = ChessGame::new(Box::new(white_player), Box::new(black_player), || {});

        Uci {
            white_channel,
            black_channel,
            game,
            stdin: std::io::stdin(),
        }
    }

    fn reset(&mut self) {
        let new = Self::new();
        *self = new;
    }

    fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut input = String::new();
        loop {
            input.clear();
            self.stdin.read_line(&mut input)?;
            let mut words = input.split_whitespace();
            let command = words.next().unwrap_or("");

            match command {
                "uci" => {
                    println!("id name ChessAI");
                    println!("id author Leo Minton");
                    println!("uciok");
                }
                "isready" => {
                    println!("readyok");
                }
                "quit" => {
                    break;
                }
                "ucinewgame" => {
                    self.reset();
                }
                "position" => {
                    let mut moves = Vec::new();
                    if words.next() == Some("startpos") {
                        // Start from the initial position
                    } else {
                        // Parse FEN or moves
                        let fen = words.next().unwrap_or("");
                        if fen != "moves" {
                            self.game.board.write()?.set_from_fen(fen);
                        }
                        for word in words {
                            if let Ok(mv) = Move::from_str(word) {
                                moves.push(mv);
                            }
                        }
                    }
                    self.game.make_moves(&moves);
                }
                _ => {
                    println!("Unknown command: {}", command);
                }
            }
        }
        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    Uci::new().run()
}
