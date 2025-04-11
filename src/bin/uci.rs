use std::{io::Stdin, mem, sync::mpsc::Sender};

use chess::{
    ai::AI,
    game::{ChannelPlayer, ChessGame, Player},
    logic::{Move, PieceColor},
};

struct Uci {
    white_channel: Sender<Move>,
    black_channel: Sender<Move>,
    game: ChessGame,
    stdin: Stdin,
    ai: AI,
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
            ai: AI::new(),
        }
    }

    fn reset(&mut self, reset_ai: bool) {
        let mut old = Self::new();
        mem::swap(self, &mut old);
        if !reset_ai {
            self.ai = old.ai;
        }
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
                    self.reset(true);
                }
                "position" => {
                    let mut board = self.game.board.write().unwrap();
                    while let Some(command) = words.next() {
                        match command {
                            "startpos" => {
                                drop(board);
                                self.reset(false);
                                board = self.game.board.write().unwrap();
                            }
                            "fen" => {
                                let fen = words.next().unwrap_or("");
                                board.set_from_fen(fen);
                            }
                            "moves" => {
                                while let Some(word) = words.next() {
                                    if let Ok(mv) = Move::from_str(word, &board) {
                                        mv.perform(&mut board);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "go" => {
                    let mut wtime: usize = 0;
                    let mut btime: usize = 0;
                    let mut winc: usize = 0;
                    let mut binc: usize = 0;
                    while let Some(command) = words.next() {
                        match command {
                            "searchmoves" => {
                                println!("Unimplemented: searchmoves");
                            }
                            "ponder" => {
                                println!("Unimplemented: ponder");
                            }
                            "wtime" => {
                                wtime = words.next().unwrap_or("0").parse().unwrap_or(0);
                            }
                            "btime" => {
                                btime = words.next().unwrap_or("0").parse().unwrap_or(0);
                            }
                            "winc" => {
                                winc = words.next().unwrap_or("0").parse().unwrap_or(0);
                            }
                            "binc" => {
                                binc = words.next().unwrap_or("0").parse().unwrap_or(0);
                            }
                            _ => {}
                        }
                    }
                    let best_move = self.ai.get_move(self.game.board.clone());
                    match self.game.board.read().unwrap().turn {
                        PieceColor::White => {
                            self.white_channel.send(best_move.clone()).unwrap();
                        }
                        PieceColor::Black => {
                            self.black_channel.send(best_move.clone()).unwrap();
                        }
                    }
                    println!("bestmove {}", best_move.to_string());
                }
                _ => {
                    println!("Unknown command: {}", command);
                }
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Uci::new().run()
}
