use std::sync::{Arc, RwLock};

use itertools::Itertools;
use rand::seq::IteratorRandom;

use crate::{
    chess::{ChessBoard, Color, Move, PieceType, WinState},
    game::Player,
};

pub struct AI;

impl AI {
    pub fn piece_value(&self, piece: PieceType) -> i32 {
        match piece {
            PieceType::Pawn => 1,
            PieceType::Knight => 3,
            PieceType::Bishop => 3,
            PieceType::Rook => 5,
            PieceType::Queen => 9,
            PieceType::King => 0,
        }
    }
    pub fn evaluate_board(&self, board: &ChessBoard, color: Color, depth: usize) -> i32 {
        if let Some(win_state) = board.win_state() {
            return match win_state {
                WinState::Checkmate(winner) => {
                    if winner == color {
                        i32::MAX
                    } else {
                        i32::MIN
                    }
                }
                WinState::Stalemate => 0,
            };
        }
        if depth > 0 {
            let mut board = board.clone();
            let valid_moves = board.valid_moves(false, board.turn);
            self.best_move(&board, &valid_moves, depth)
                .perform(&mut board);
            return self.evaluate_board(&board, color, depth - 1);
        }
        let mut score = 0;
        for piece in board.pieces.iter() {
            if piece.color == color {
                score += self.piece_value(piece.piece_type);
            } else {
                score -= self.piece_value(piece.piece_type);
            }
        }
        score
    }

    pub fn best_move(&self, board: &ChessBoard, valid_moves: &[Move], depth: usize) -> Move {
        let options = valid_moves.into_iter().map(|&m| {
            let mut new_board = board.clone();
            m.perform(&mut new_board);
            (m, self.evaluate_board(&new_board, board.turn, depth - 1))
        });
        options
            .max_set_by_key(|&(_, score)| score)
            .into_iter()
            .choose(&mut rand::rng())
            .unwrap()
            .0
    }
}

impl Player for AI {
    fn get_move(&self, board: Arc<RwLock<ChessBoard>>) -> Move {
        let board = board.read().unwrap();
        let valid_moves = board.valid_moves(false, board.turn);
        return self.best_move(&board, &valid_moves, 3);
    }
}
