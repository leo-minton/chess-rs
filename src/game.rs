use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, RwLock,
};

use crate::chess::{ChessBoard, Color, WinState};

pub struct ChessGame {
    pub board: Arc<RwLock<ChessBoard>>,
    pub white_player: Box<dyn Player>,
    pub black_player: Box<dyn Player>,
    pub on_update_func: Box<dyn Fn() + Send + 'static>,
}

impl ChessGame {
    pub fn new(
        white_player: Box<dyn Player>,
        black_player: Box<dyn Player>,
        on_update_func: impl Fn() + Send + 'static,
    ) -> Self {
        Self {
            board: Arc::new(RwLock::new(ChessBoard::new())),
            white_player,
            black_player,
            on_update_func: Box::new(on_update_func),
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
            let new_ref = self.board.clone();
            let current_player = self.get_player(current_player);
            let chess_move = current_player.get_move(new_ref);

            let mut board = self.board.write().unwrap();

            chess_move.perform(&mut board);

            (self.on_update_func)();

            if let Some(win_state) = board.win_state() {
                return win_state;
            }
        }
    }

    pub fn get_player(&mut self, color: Color) -> &mut dyn Player {
        match color {
            Color::White => self.white_player.as_mut(),
            Color::Black => self.black_player.as_mut(),
        }
    }
}

pub trait Player: Send {
    fn get_move(&mut self, board: Arc<RwLock<ChessBoard>>) -> crate::chess::Move;
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
    fn get_move(&mut self, _board: Arc<RwLock<ChessBoard>>) -> crate::chess::Move {
        self.move_channel.recv().unwrap_or_else(|_| {
            std::process::exit(0);
        })
    }
}
