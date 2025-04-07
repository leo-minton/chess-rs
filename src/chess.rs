use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl PieceType {
    pub fn promotable_to(&self) -> bool {
        match self {
            PieceType::Pawn => false,
            PieceType::King => false,
            _ => true,
        }
    }
}

impl Display for PieceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PieceType::King => write!(f, "K"),
            PieceType::Queen => write!(f, "Q"),
            PieceType::Rook => write!(f, "R"),
            PieceType::Bishop => write!(f, "B"),
            PieceType::Knight => write!(f, "N"),
            PieceType::Pawn => write!(f, "P"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
    pub fn readable(&self) -> &'static str {
        match self {
            Color::White => "White",
            Color::Black => "Black",
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::White => write!(f, "w"),
            Color::Black => write!(f, "b"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChessPiece {
    pub piece_type: PieceType,
    pub pos: (usize, usize),
    pub color: Color,
    pub has_moved: bool,
}

impl ChessPiece {
    pub fn new(piece_type: PieceType, pos: (usize, usize), color: Color) -> Self {
        Self {
            piece_type,
            pos,
            color,
            has_moved: false,
        }
    }

    pub fn move_to(&mut self, target: (usize, usize)) {
        self.pos = target;
        self.has_moved = true;
    }

    fn add_in_dir(
        dir: (isize, isize),
        pos: (usize, usize),
        board: &ChessBoard,
        moves: &mut Vec<Move>,
    ) {
        let mut target = (pos.0 as isize + dir.0, pos.1 as isize + dir.1);
        while (0..8).contains(&(target.0 as usize)) && (0..8).contains(&(target.1 as usize)) {
            moves.push(Move::new(
                pos,
                (target.0 as usize, target.1 as usize),
                MoveType::Normal,
            ));
            if board
                .piece_at((target.0 as usize, target.1 as usize))
                .is_some()
            {
                break;
            }
            target = (target.0 + dir.0, target.1 + dir.1);
        }
    }

    pub fn valid_moves(&self, board: &ChessBoard, ignore_check: bool) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        match self.piece_type {
            PieceType::King => {
                if !ignore_check && !board.is_in_check(self.color) && !self.has_moved {
                    for rook in board.pieces.iter().filter(|p| {
                        p.piece_type == PieceType::Rook && p.color == self.color && !p.has_moved
                    }) {
                        let direction = (rook.pos.0 as isize - self.pos.0 as isize).signum();
                        if (1..(rook.pos.0 as isize - self.pos.0 as isize).abs()).all(|i| {
                            board
                                .piece_at((
                                    (self.pos.0 as isize + i * direction) as usize,
                                    self.pos.1,
                                ))
                                .is_none()
                        }) && !board.is_pos_attacked(
                            ((self.pos.0 as isize + direction) as usize, self.pos.1),
                            self.color.opposite(),
                            true,
                        ) {
                            moves.push(Move::new_with_isize(
                                self.pos,
                                (self.pos.0 as isize + 2 * direction, self.pos.1 as isize),
                                MoveType::Castling {
                                    rook: rook.pos,
                                    direction,
                                },
                            ));
                        }
                    }
                }

                for (dx, dy) in [-1, 0, 1]
                    .iter()
                    .flat_map(|&dx| [-1, 0, 1].iter().map(move |&dy| (dx, dy)))
                    .filter(|&(dx, dy)| dx != 0 || dy != 0)
                {
                    moves.push(Move::new_with_isize(
                        self.pos,
                        (self.pos.0 as isize + dx, self.pos.1 as isize + dy),
                        MoveType::Normal,
                    ));
                }
            }
            PieceType::Queen | PieceType::Rook | PieceType::Bishop => {
                let directions = match self.piece_type {
                    PieceType::Queen => [-1, 0, 1]
                        .iter()
                        .flat_map(|&dx| [-1, 0, 1].iter().map(move |&dy| (dx, dy)))
                        .filter(|&(dx, dy)| dx != 0 || dy != 0)
                        .collect::<Vec<_>>(),
                    PieceType::Rook => [-1, 0, 1]
                        .iter()
                        .flat_map(|&dx| [-1, 0, 1].iter().map(move |&dy| (dx, dy)))
                        .filter(|&(dx, dy)| dx == 0 || dy == 0)
                        .collect::<Vec<_>>(),
                    PieceType::Bishop => [-1, 0, 1]
                        .iter()
                        .flat_map(|&dx| [-1, 0, 1].iter().map(move |&dy| (dx, dy)))
                        .filter(|&(dx, dy)| dx != 0 && dy != 0)
                        .collect::<Vec<_>>(),
                    _ => unreachable!(),
                };
                for &dir in &directions {
                    Self::add_in_dir(dir, self.pos, board, &mut moves);
                }
            }
            PieceType::Knight => {
                for &(dx, dy) in &[
                    (2, 1),
                    (2, -1),
                    (-2, 1),
                    (-2, -1),
                    (1, 2),
                    (1, -2),
                    (-1, 2),
                    (-1, -2),
                ] {
                    moves.push(Move::new_with_isize(
                        self.pos,
                        (self.pos.0 as isize + dx, self.pos.1 as isize + dy),
                        MoveType::Normal,
                    ));
                }
            }
            PieceType::Pawn => {
                let direction = if self.color == Color::White { -1 } else { 1 };
                let target_row = (self.pos.1 as isize + direction) as usize;

                if board.piece_at((self.pos.0, target_row)).is_none() {
                    if target_row == 0 || target_row == 7 {
                        moves.extend(PieceType::iter().filter(|p| p.promotable_to()).map(
                            |piece| {
                                Move::new(
                                    self.pos,
                                    (self.pos.0, target_row),
                                    MoveType::Promotion(piece),
                                )
                            },
                        ));
                    } else {
                        moves.push(Move::new(
                            self.pos,
                            (self.pos.0, target_row),
                            MoveType::Normal,
                        ));
                    }
                    if !self.has_moved {
                        let double_target_row = (self.pos.1 as isize + 2 * direction) as usize;
                        if board.piece_at((self.pos.0, double_target_row)).is_none() {
                            moves.push(Move::new(
                                self.pos,
                                (self.pos.0, double_target_row),
                                MoveType::Normal,
                            ));
                        }
                    }
                }

                for &(dx, dy) in &[(-1, direction), (1, direction)] {
                    let target = (self.pos.0 as isize + dx, self.pos.1 as isize + dy);
                    if (0..8).contains(&target.0) && (0..8).contains(&target.1) {
                        if let Some(target_piece) =
                            board.piece_at((target.0 as usize, target.1 as usize))
                        {
                            if target_piece.color != self.color {
                                moves.push(Move::new(
                                    self.pos,
                                    (target.0 as usize, target.1 as usize),
                                    MoveType::Normal,
                                ));
                            }
                        }
                    }
                }
            }
        }
        moves
            .into_iter()
            .filter(|m| m.is_valid(board, ignore_check))
            .collect()
    }
}

pub enum WinState {
    Checkmate(Color),
    Stalemate,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]

pub enum MoveType {
    Normal,
    Castling {
        rook: (usize, usize),
        direction: isize,
    },
    EnPassant,
    Promotion(PieceType),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Move {
    pub original: (usize, usize),
    pub target: (usize, usize),
    pub move_type: MoveType,
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} -> {}",
            pos_to_notation(self.original),
            pos_to_notation(self.target),
        )
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Move {
    pub fn new(original: (usize, usize), target: (usize, usize), move_type: MoveType) -> Self {
        Self {
            original,
            target,
            move_type,
        }
    }

    pub fn new_with_isize(
        original: (usize, usize),
        target: (isize, isize),
        move_type: MoveType,
    ) -> Self {
        if target.0 < 0 || target.1 < 0 {
            return Self {
                original,
                target: (usize::MAX, usize::MAX),
                move_type,
            };
        }
        Self {
            original,
            target: (target.0 as usize, target.1 as usize),
            move_type,
        }
    }

    pub fn is_valid(&self, board: &ChessBoard, ignore_check: bool) -> bool {
        if self.target.0 >= 8 || self.target.1 >= 8 {
            return false;
        }
        if let Some(piece) = board.piece_at(self.original) {
            if let Some(target_piece) = board.piece_at(self.target) {
                if piece.color == target_piece.color {
                    return false;
                }
            }
        } else {
            return false;
        }
        if !ignore_check {
            let mut temp_board = board.clone();
            if let Some(piece) = board.piece_at(self.original) {
                self.perform(&mut temp_board);
                if temp_board.is_in_check(piece.color) {
                    return false;
                }
            }
        }
        true
    }

    pub fn perform(&self, board: &mut ChessBoard) {
        board.pieces.retain(|p| p.pos != self.target);
        if let Some(piece) = board.piece_at_mut(self.original) {
            piece.move_to(self.target);
            match self.move_type {
                MoveType::Castling { rook, direction } => {
                    if let Some(rook_piece) = board.piece_at_mut(rook) {
                        let target = ((self.target.0 as isize - direction) as usize, self.target.1);
                        rook_piece.move_to(target);
                    }
                }
                MoveType::Promotion(piece_type) => {
                    piece.piece_type = piece_type;
                }
                MoveType::EnPassant => {
                    let target = (self.target.0, self.original.1);
                    board.pieces.retain(|p| p.pos != target);
                }
                MoveType::Normal => {}
            }
        }
        board.turn = board.turn.opposite();
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChessBoard {
    pub pieces: Vec<ChessPiece>,
    pub turn: Color,
}

impl Default for ChessBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl ChessBoard {
    pub fn new() -> Self {
        let mut board = ChessBoard {
            pieces: Vec::new(),
            turn: Color::White,
        };
        board.initialize_pieces();
        board
    }

    fn initialize_pieces(&mut self) {
        let initial_positions: HashMap<PieceType, Vec<&str>> = HashMap::from([
            (
                PieceType::Pawn,
                vec!["a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2"],
            ),
            (PieceType::Rook, vec!["a1", "h1"]),
            (PieceType::Knight, vec!["b1", "g1"]),
            (PieceType::Bishop, vec!["c1", "f1"]),
            (PieceType::Queen, vec!["d1"]),
            (PieceType::King, vec!["e1"]),
        ]);

        for (piece, positions) in initial_positions.iter() {
            for &pos in positions {
                if let Some((x, y)) = notation_to_pos(pos) {
                    self.pieces
                        .push(ChessPiece::new(*piece, (x, y), Color::White));
                    self.pieces
                        .push(ChessPiece::new(*piece, (x, 7 - y), Color::Black));
                }
            }
        }
    }

    pub fn piece_at(&self, pos: (usize, usize)) -> Option<&ChessPiece> {
        let pos = (pos.0 as usize, pos.1 as usize);
        self.pieces.iter().find(|p| p.pos == pos)
    }

    pub fn piece_at_mut(&mut self, pos: (usize, usize)) -> Option<&mut ChessPiece> {
        let pos = (pos.0 as usize, pos.1 as usize);
        self.pieces.iter_mut().find(|p| p.pos == pos)
    }

    pub fn valid_moves<'a>(
        &'a self,
        ignore_check: bool,
        color: Color,
    ) -> impl Iterator<Item = Move> + 'a {
        self.pieces
            .iter()
            .filter(move |piece| piece.color == color)
            .flat_map(move |piece| piece.valid_moves(self, ignore_check))
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        let moves = self.valid_moves(true, color.opposite());
        for valid_move in moves {
            let target_piece = self.piece_at(valid_move.target);
            if let Some(piece) = target_piece {
                if piece.piece_type == PieceType::King && piece.color == color {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_pos_attacked(
        &self,
        pos: (usize, usize),
        attacking_color: Color,
        ignore_check: bool,
    ) -> bool {
        let moves = self.valid_moves(ignore_check, attacking_color);
        for valid_move in moves {
            if valid_move.target == pos {
                return true;
            }
        }
        false
    }

    pub fn win_state(&self) -> Option<WinState> {
        if self.valid_moves(false, self.turn).next().is_none() {
            if self.is_in_check(self.turn) {
                return Some(WinState::Checkmate(self.turn.opposite()));
            } else {
                return Some(WinState::Stalemate);
            }
        }
        None
    }
}

pub fn notation_to_pos(notation: &str) -> Option<(usize, usize)> {
    if notation.len() != 2 {
        return None;
    }
    let chars: Vec<char> = notation.chars().collect();
    let x = chars[0] as usize - 'a' as usize;
    let y = 8 - chars[1].to_digit(10)? as usize;
    Some((x, y))
}

pub fn pos_to_notation(pos: (usize, usize)) -> String {
    let x = (pos.0 as u8 + b'a') as char;
    let y = (8 - pos.1).to_string();
    format!("{}{}", x, y)
}
