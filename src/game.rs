use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, RwLock,
};

use crate::chess::{ChessBoard, Color, WinState};

pub struct ChessGame {
    pub board: Arc<RwLock<ChessBoard>>,
    pub white_player: Box<dyn Player>,
    pub black_player: Box<dyn Player>,
}

impl ChessGame {
    pub fn new(white_player: Box<dyn Player>, black_player: Box<dyn Player>) -> Self {
        Self {
            board: Arc::new(RwLock::new(ChessBoard::new())),
            white_player,
            black_player,
        }
    }

    pub fn create_game_thread(mut self) -> std::thread::JoinHandle<WinState> {
        std::thread::spawn(move || self.play())
    }

    pub fn play(&mut self) -> WinState {
        loop {
            let current_player = {
                let board = self.board.read().unwrap();
                board.turn
            };
            let current_player = self.get_player(current_player);
            let chess_move = current_player.get_move(self.board.clone());

            let mut board = self.board.write().unwrap();

            chess_move.perform(&mut board);

            if let Some(win_state) = board.win_state() {
                return win_state;
            }
        }
    }

    pub fn get_player(&self, color: Color) -> &dyn Player {
        match color {
            Color::White => self.white_player.as_ref(),
            Color::Black => self.black_player.as_ref(),
        }
    }
}

pub trait Player: Send {
    fn get_move(&self, board: Arc<RwLock<ChessBoard>>) -> crate::chess::Move;
}

pub struct HumanPlayer {
    pub move_channel: Receiver<crate::chess::Move>,
}

impl HumanPlayer {
    pub fn new() -> (Sender<crate::chess::Move>, Self) {
        let (tx, rx) = mpsc::channel();
        (tx, Self { move_channel: rx })
    }
}

impl Player for HumanPlayer {
    fn get_move(&self, _board: Arc<RwLock<ChessBoard>>) -> crate::chess::Move {
        self.move_channel.recv().unwrap()
    }
}
