use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    fmt::{Debug, Display},
    str::FromStr,
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
            PieceType::King => write!(f, "k"),
            PieceType::Queen => write!(f, "q"),
            PieceType::Rook => write!(f, "r"),
            PieceType::Bishop => write!(f, "b"),
            PieceType::Knight => write!(f, "n"),
            PieceType::Pawn => write!(f, "p"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParsePieceError;

impl FromStr for PieceType {
    type Err = ParsePieceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            return Err(ParsePieceError);
        }
        match s.chars().next().unwrap().to_ascii_lowercase() {
            'k' => Ok(PieceType::King),
            'q' => Ok(PieceType::Queen),
            'r' => Ok(PieceType::Rook),
            'b' => Ok(PieceType::Bishop),
            'n' => Ok(PieceType::Knight),
            'p' => Ok(PieceType::Pawn),
            _ => Err(ParsePieceError),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter, Hash)]
pub enum PieceColor {
    White,
    Black,
}

impl PieceColor {
    pub fn opposite(&self) -> PieceColor {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
    pub fn readable(&self) -> &'static str {
        match self {
            PieceColor::White => "White",
            PieceColor::Black => "Black",
        }
    }
}

impl Display for PieceColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PieceColor::White => write!(f, "w"),
            PieceColor::Black => write!(f, "b"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChessPiece {
    pub piece_type: PieceType,
    pub pos: (usize, usize),
    pub color: PieceColor,
    pub first_move_at: Option<usize>,
}

impl ChessPiece {
    pub fn new(piece_type: PieceType, pos: (usize, usize), color: PieceColor) -> Self {
        Self {
            piece_type,
            pos,
            color,
            first_move_at: None,
        }
    }

    pub fn move_to(mut self, target: (usize, usize), first_move_at: usize, board: &mut ChessBoard) {
        self.pos = target;
        self.first_move_at = Some(first_move_at);
        board.pieces[ChessBoard::pos_to_idx(target)] = Some(self);
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

    pub fn valid_moves<'a>(
        &self,
        board: &'a ChessBoard,
        ignore_check: bool,
    ) -> impl Iterator<Item = Move> + 'a {
        let mut moves = Vec::with_capacity(64);
        match self.piece_type {
            PieceType::King => {
                if !ignore_check && !board.is_in_check(self.color) && self.first_move_at.is_none() {
                    for rook in board.pieces.iter().filter_map(|p| {
                        if p.as_ref().is_some_and(|p| {
                            p.piece_type == PieceType::Rook
                                && p.color == self.color
                                && p.first_move_at.is_none()
                        }) {
                            p.as_ref()
                        } else {
                            None
                        }
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
                let direction = if self.color == PieceColor::White {
                    -1
                } else {
                    1
                };
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
                    if self.first_move_at.is_none() {
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
            .filter(move |m| m.is_valid(board, ignore_check))
    }
}

pub enum WinState {
    Checkmate(PieceColor),
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

impl ToString for Move {
    fn to_string(&self) -> String {
        match self.move_type {
            MoveType::Normal | MoveType::EnPassant | MoveType::Castling { .. } => format!(
                "{}{}",
                pos_to_notation(self.original),
                pos_to_notation(self.target)
            ),
            MoveType::Promotion(piece_type) => format!(
                "{}{}{}",
                pos_to_notation(self.original),
                pos_to_notation(self.target),
                piece_type
            ),
        }
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
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

    pub fn from_str(s: &str, board: &ChessBoard) -> Result<Self, ()> {
        match s.len() {
            4 => {
                let original = notation_to_pos(&s[0..2]).ok_or(())?;
                let target = notation_to_pos(&s[2..4]).ok_or(())?;
                let piece = board.piece_at(original).ok_or(())?;
                if piece.piece_type == PieceType::King
                    && (original.0 as isize - target.0 as isize).abs() == 2
                {
                    Ok(Move::new(
                        original,
                        target,
                        MoveType::Castling {
                            rook: (if target.0 < 4 { 0 } else { 7 }, target.1),
                            direction: (target.0 as isize - original.0 as isize).signum(),
                        },
                    ))
                } else {
                    Ok(Move::new(original, target, MoveType::Normal))
                }
            }
            5 => {
                let original = notation_to_pos(&s[0..2]).ok_or(())?;
                let target = notation_to_pos(&s[2..4]).ok_or(())?;
                let piece_type = PieceType::from_str(&s[4..5]).map_err(|_| ())?;
                Ok(Move::new(original, target, MoveType::Promotion(piece_type)))
            }
            _ => Err(()),
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
        let moves_made = board.moves_made;
        if let Some(mut piece) = board.pieces[ChessBoard::pos_to_idx(self.original)].take() {
            match self.move_type {
                MoveType::Castling { rook, direction } => {
                    if let Some(rook_piece) = board.pieces[ChessBoard::pos_to_idx(rook)].take() {
                        let target = ((self.target.0 as isize - direction) as usize, self.target.1);
                        rook_piece.move_to(target, moves_made, board);
                    }
                }
                MoveType::Promotion(piece_type) => {
                    piece.piece_type = piece_type;
                }
                MoveType::EnPassant => {
                    let target = (self.target.0, self.original.1);
                    board.pieces[ChessBoard::pos_to_idx(target)] = None;
                }
                MoveType::Normal => {}
            }
            piece.move_to(self.target, moves_made, board);
        }
        board.turn = board.turn.opposite();
        board.moves_made += 1;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChessBoard {
    pub pieces: [Option<ChessPiece>; 64],
    pub turn: PieceColor,
    pub moves_made: usize,
}

impl Default for ChessBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl ChessBoard {
    pub fn new() -> Self {
        let mut board = ChessBoard {
            pieces: [const { None }; 64],
            turn: PieceColor::White,
            moves_made: 0,
        };
        board.initialize_pieces();
        board
    }

    fn pos_to_idx(pos: (usize, usize)) -> usize {
        pos.0 + pos.1 * 8
    }

    fn initialize_pieces(&mut self) {
        self.set_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR")
    }
    pub fn set_from_fen(&mut self, fen: &str) {
        let lines = fen.split('/');
        let mut pos = (0, 0);
        self.pieces = [const { None }; 64];
        for line in lines {
            for c in line.chars() {
                match c {
                    '1'..='8' => {
                        let empty_squares = c.to_digit(10).unwrap() as usize;
                        pos.0 += empty_squares;
                    }
                    c => {
                        let piece_type = PieceType::from_str(&c.to_string()).unwrap();
                        let color = if c.is_uppercase() {
                            PieceColor::White
                        } else {
                            PieceColor::Black
                        };
                        self.pieces[Self::pos_to_idx(pos)] =
                            Some(ChessPiece::new(piece_type, pos, color));
                        pos.0 += 1;
                    }
                }
            }
            pos.0 = 0;
            pos.1 += 1;
            if pos.1 >= 8 {
                break;
            }
        }
    }

    pub fn piece_at(&self, pos: (usize, usize)) -> Option<&ChessPiece> {
        self.pieces[Self::pos_to_idx(pos)].as_ref()
    }

    pub fn piece_at_mut(&mut self, pos: (usize, usize)) -> Option<&mut ChessPiece> {
        self.pieces[Self::pos_to_idx(pos)].as_mut()
    }

    pub fn valid_moves<'a>(
        &'a self,
        ignore_check: bool,
        color: PieceColor,
    ) -> impl ParallelIterator<Item = Move> + 'a {
        self.pieces
            .par_iter()
            .filter_map(move |piece| {
                piece.as_ref().and_then(|piece| {
                    if piece.color == color {
                        Some(piece)
                    } else {
                        None
                    }
                })
            })
            .flat_map_iter(move |piece| piece.valid_moves(self, ignore_check))
    }

    pub fn is_in_check(&self, color: PieceColor) -> bool {
        self.valid_moves(true, color.opposite()).any(|m| {
            self.piece_at(m.target)
                .map_or(false, |p| p.piece_type == PieceType::King)
        })
    }

    pub fn is_pos_attacked(
        &self,
        pos: (usize, usize),
        attacking_color: PieceColor,
        ignore_check: bool,
    ) -> bool {
        let moves = self.valid_moves(ignore_check, attacking_color);
        return moves.any(|m| m.target == pos);
    }

    pub fn win_state(&self) -> Option<WinState> {
        if self.valid_moves(false, self.turn).all(|_| false) {
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
