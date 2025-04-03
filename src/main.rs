use std::collections::HashMap;
use strum::IntoEnumIterator;

use chess::{ChessBoard, Color, Move, PieceType, WinState};
use eframe::{
    egui::{
        self, Color32, ColorImage, Frame, PointerButton, Rect, Sense, TextureHandle,
        TextureOptions, Ui, Vec2,
    },
    CreationContext,
};
use include_dir::{include_dir, Dir};

pub mod chess;

const BOARD_SIZE: usize = 8;
const DEFAULT_ASSETS: &str = "default";
static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/assets");

const DARK_SQUARE: egui::Color32 = egui::Color32::from_rgb(181, 136, 99);
const LIGHT_SQUARE: egui::Color32 = egui::Color32::from_rgb(240, 217, 181);
const SELECTED_SQUARE: egui::Color32 = egui::Color32::from_rgba_premultiplied(115, 154, 222, 128);
const VALID_MOVE: egui::Color32 = egui::Color32::from_rgba_premultiplied(81, 173, 94, 128);

fn load_image_from_memory(image_data: &[u8]) -> ColorImage {
    let image = image::load_from_memory(image_data).expect("Failed to load image");
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    ColorImage::from_rgba_unmultiplied(size, pixels.as_slice())
}

struct ChessApp {
    images: HashMap<(PieceType, Color), TextureHandle>,
    board: ChessBoard,
    selected_piece: Option<(usize, usize)>,
    valid_moves: Vec<Move>,
    win_state: Option<WinState>,
}

impl ChessApp {
    fn new(cc: &CreationContext) -> Self {
        let mut app = Self {
            images: HashMap::new(),
            board: ChessBoard::new(),
            selected_piece: None,
            valid_moves: Vec::new(),
            win_state: None,
        };
        app.load_assets(cc);
        app
    }

    fn load_assets(&mut self, cc: &CreationContext) {
        for piece in PieceType::iter() {
            for color in Color::iter() {
                let path = &format!("{}/{}{}.png", DEFAULT_ASSETS, color, piece);
                if let Some(image) = ASSETS.get_file(path).and_then(|f| Some(f.contents())) {
                    let image = load_image_from_memory(image);
                    self.images.insert(
                        (piece, color),
                        cc.egui_ctx
                            .load_texture("image", image, TextureOptions::default()),
                    );
                } else {
                    panic!("Could not find asset file: {}", path);
                }
            }
        }
    }

    fn get_image(&self, piece: PieceType, color: Color) -> &TextureHandle {
        self.images.get(&(piece, color)).unwrap()
    }

    fn chessboard(&mut self, ui: &mut Ui) -> egui::Response {
        let mut size = ui.available_size_before_wrap();
        size = Vec2::splat(size.x.min(size.y));
        // Placeholder for chessboard drawing logic
        let (response, painter) = ui.allocate_painter(size, Sense::click());

        let square_size = size.x / BOARD_SIZE as f32;

        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                let color = if (row + col) % 2 == 0 {
                    DARK_SQUARE
                } else {
                    LIGHT_SQUARE
                };

                let rect = egui::Rect::from_min_size(
                    response.rect.min
                        + Vec2::new(col as f32 * square_size, row as f32 * square_size),
                    Vec2::splat(square_size),
                );
                painter.rect_filled(rect, 0.0, color);
                if self.selected_piece.is_some_and(|p| p == (col, row)) {
                    painter.rect_filled(rect, 0.0, SELECTED_SQUARE);
                }
            }
        }

        for valid_move in &self.valid_moves {
            let pos =
                Vec2::new(valid_move.target.0 as f32, valid_move.target.1 as f32) * square_size;
            let rect = Rect::from_min_size(response.rect.min + pos, Vec2::splat(square_size));
            painter.rect_filled(rect, 0.0, VALID_MOVE);
        }

        for piece in &self.board.pieces {
            let pos = Vec2::new(piece.pos.0 as f32, piece.pos.1 as f32) * square_size;
            let rect = Rect::from_min_size(response.rect.min + pos, Vec2::splat(square_size));

            egui::Image::new(self.get_image(piece.piece_type, piece.color)).paint_at(ui, rect);
        }

        if self.win_state.is_none() && response.clicked_by(PointerButton::Primary) {
            let pos = response.interact_pointer_pos().unwrap();
            let col = ((pos.x - response.rect.min.x) / square_size).floor() as usize;
            let row = ((pos.y - response.rect.min.y) / square_size).floor() as usize;

            if col < BOARD_SIZE && row < BOARD_SIZE {
                let target_pos = (col, row);
                if self.selected_piece.is_none() {
                    if let Some(piece) = self.board.piece_at(target_pos) {
                        if piece.color == self.board.turn {
                            self.selected_piece = Some((col, row));
                            self.valid_moves = piece.valid_moves(&self.board, false);
                        }
                    }
                } else {
                    if let Some(valid_move) =
                        self.valid_moves.iter().find(|&m| m.target == target_pos)
                    {
                        valid_move.perform(&mut self.board);
                        self.selected_piece = None;
                        self.valid_moves.clear();
                        self.win_state = self.board.win_state();
                    } else {
                        self.selected_piece = None;
                        self.valid_moves.clear();
                    }
                }
            }
        }

        response
    }
}

impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                if let Some(win_state) = &self.win_state {
                    match win_state {
                        WinState::Color(color) => {
                            ui.heading(format!("{} wins!", color.readable()));
                        }
                        WinState::Draw => {
                            ui.heading("Draw!");
                        }
                    }
                } else {
                    ui.heading(format!("{}'s turn", self.board.turn.readable()));
                }

                Frame::canvas(ui.style())
                    .stroke((0_f32, Color32::TRANSPARENT))
                    .fill(Color32::TRANSPARENT)
                    .show(ui, |ui| self.chessboard(ui));
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Chess Game",
        options,
        Box::new(|cc| Ok(Box::new(ChessApp::new(cc)))),
    )
}
