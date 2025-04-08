use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::{
    chess::{ChessBoard, Move, PieceType, WinState},
    game::Player,
};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct BoardNode {
    pub board: ChessBoard,
    pub score: i32,
    pub children: HashMap<Move, BoardNode>,
}

pub struct AI {
    pub tree: BoardNode,
}

impl AI {
    pub fn new() -> Self {
        Self {
            tree: BoardNode {
                board: ChessBoard::new(),
                score: 0,
                children: HashMap::new(),
            },
        }
    }

    pub fn evaluate_tree(tree: &mut BoardNode, depth: usize) {
        if tree.children.is_empty() {
            if let Some(win_state) = tree.board.win_state() {
                tree.score = match win_state {
                    WinState::Checkmate(winner) => {
                        if winner == tree.board.turn {
                            i32::MIN
                        } else {
                            i32::MAX
                        }
                    }
                    WinState::Stalemate => 0,
                };
                return;
            }
            if depth > 0 {
                let valid_moves = tree
                    .board
                    .valid_moves(false, tree.board.turn)
                    .collect::<Vec<_>>();
                for m in valid_moves {
                    let mut new_board = tree.board.clone();
                    m.perform(&mut new_board);
                    let child_node = BoardNode {
                        board: new_board,
                        score: 0,
                        children: HashMap::new(),
                    };
                    tree.children.insert(m, child_node);
                }
            }
        }
        if depth == 0 {
            let mut score = 0;
            for piece in &tree.board.pieces {
                let piece_score = match piece.piece_type {
                    PieceType::Pawn => 1,
                    PieceType::Knight => 3,
                    PieceType::Bishop => 3,
                    PieceType::Rook => 5,
                    PieceType::Queen => 9,
                    PieceType::King => {
                        if piece.has_moved {
                            0
                        } else {
                            2
                        }
                    }
                };
                if piece.color == tree.board.turn {
                    score -= piece_score;
                } else {
                    score += piece_score;
                }
            }
            tree.score = score;
        } else {
            let mut children: Vec<_> = tree.children.values_mut().collect();
            let score = children
                .par_iter_mut()
                .map(|child| {
                    Self::evaluate_tree(child, depth - 1);
                    child.score
                })
                .max()
                .unwrap_or_default();
            tree.score = -score;
        }
    }

    pub fn best_move(&mut self, board: &ChessBoard, depth: usize) -> Move {
        if &self.tree.board != board {
            if self
                .tree
                .children
                .iter()
                .flat_map(|(_, child)| child.children.iter())
                .any(|(_, child)| &child.board == board)
            {
                self.tree = self
                    .tree
                    .clone()
                    .children
                    .into_iter()
                    .flat_map(|(_, child)| child.children.into_iter())
                    .find(|(_, child)| &child.board == board)
                    .unwrap()
                    .1;
            } else {
                self.tree = BoardNode {
                    board: board.clone(),
                    score: 0,
                    children: HashMap::new(),
                };
            }
        }
        Self::evaluate_tree(&mut self.tree, depth);
        self.tree
            .children
            .iter()
            .max_by_key(|(_, child)| child.score)
            .map(|(m, _)| m.clone())
            .expect("Board should always have valid moves")
    }
}

impl Player for AI {
    fn get_move(&mut self, board: Arc<RwLock<ChessBoard>>) -> Move {
        let board = board.read().unwrap();
        return self.best_move(&board, 4);
    }
}
