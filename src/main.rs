use game::{ChessGame, HumanPlayer};
use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc, RwLock},
};
use strum::IntoEnumIterator;

use chess::{ChessBoard, Color, Move, MoveType, PieceType, WinState};
use eframe::{
    egui::{
        self, Align2, Area, Color32, ColorImage, Frame, Id, Modal, PointerButton, Pos2, Rect,
        Sense, TextureHandle, TextureOptions, Ui, UiKind, Vec2,
    },
    CreationContext,
};
use include_dir::{include_dir, Dir};

pub mod ai;
pub mod chess;
pub mod game;

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
    board: Arc<RwLock<ChessBoard>>,
    selected_piece: Option<(usize, usize)>,
    valid_moves: Vec<Move>,
    win_state: Option<WinState>,
    restart_modal_closed: bool,
    promoting_piece: Option<(usize, usize)>,
    white_channel: Option<Sender<Move>>,
    black_channel: Option<Sender<Move>>,
    game_thread: Option<std::thread::JoinHandle<WinState>>,
}

impl ChessApp {
    fn new(cc: &CreationContext) -> Self {
        let mut app = Self {
            images: HashMap::new(),
            board: Arc::new(RwLock::new(ChessBoard::new())),
            selected_piece: None,
            valid_moves: Vec::new(),
            win_state: None,
            restart_modal_closed: false,
            promoting_piece: None,
            white_channel: None,
            black_channel: None,
            game_thread: None,
        };
        app.load_assets(cc);
        app.reset();
        app
    }

    fn reset(&mut self) {
        self.selected_piece = None;
        self.valid_moves.clear();
        self.win_state = None;

        let (white_channel, player) = HumanPlayer::new();
        self.white_channel = Some(white_channel);
        let game = ChessGame::new(Box::new(player), Box::new(ai::AI));
        self.board = game.board.clone();
        self.game_thread = Some(game.create_game_thread());
    }

    fn channel(&self, color: Color) -> Option<Sender<Move>> {
        match color {
            Color::White => self.white_channel.clone(),
            Color::Black => self.black_channel.clone(),
        }
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

        let board = self.board.read().unwrap();
        for piece in &board.pieces {
            let pos = Vec2::new(piece.pos.0 as f32, piece.pos.1 as f32) * square_size;
            let rect = Rect::from_min_size(response.rect.min + pos, Vec2::splat(square_size));

            egui::Image::new(self.get_image(piece.piece_type, piece.color)).paint_at(ui, rect);
        }

        if let Some(pos) = self.promoting_piece {
            let options = self
                .valid_moves
                .iter()
                .filter(|m| m.target == pos)
                .filter_map(|m| {
                    if let MoveType::Promotion(p) = m.move_type {
                        Some((p, m))
                    } else {
                        None
                    }
                });

            let target_square = Rect::from_min_size(
                Pos2::new(
                    pos.0 as f32 * square_size + response.rect.min.x,
                    pos.1 as f32 * square_size + response.rect.min.y,
                ),
                Vec2::splat(square_size),
            );

            let mut selected_move = None;

            Area::new(Id::new("Promotion popup"))
                .order(egui::Order::Foreground)
                .pivot(Align2::CENTER_TOP)
                .kind(UiKind::Popup)
                .fixed_pos(target_square.center_top())
                .default_width(square_size)
                .show(ui.ctx(), |ui| {
                    let mut styles = ui.style_mut().clone();
                    styles.spacing.item_spacing =
                        Vec2::splat(styles.visuals.widgets.active.bg_stroke.width);

                    Frame::popup(&styles).show(ui, |ui| {
                        for (i, (piece, mv)) in options.enumerate() {
                            let styles = ui.style_mut();

                            styles.spacing.button_padding = Vec2::ZERO;
                            let color = if i % 2 == 0 {
                                DARK_SQUARE
                            } else {
                                LIGHT_SQUARE
                            };
                            styles.visuals.widgets.inactive.weak_bg_fill = color;
                            styles.visuals.widgets.hovered.weak_bg_fill =
                                color.lerp_to_gamma(Color32::LIGHT_GRAY, 0.25);
                            styles.visuals.widgets.active.weak_bg_fill =
                                color.lerp_to_gamma(Color32::DARK_GRAY, 0.25);
                            let all_widget_stypes = [
                                styles.visuals.widgets.inactive,
                                styles.visuals.widgets.hovered,
                                styles.visuals.widgets.active,
                            ];
                            for mut style in all_widget_stypes {
                                style.expansion = 0.0;
                            }

                            let image = self.get_image(piece, board.turn);
                            let button = ui.add(egui::ImageButton::new(
                                egui::Image::new(image).fit_to_exact_size(Vec2::splat(square_size)),
                            ));
                            if button.clicked() {
                                selected_move = Some(mv);
                            }
                        }
                    })
                });

            if let Some(mv) = selected_move {
                if let Some(channel) = self.channel(board.turn) {
                    channel.send(*mv).unwrap();
                    self.promoting_piece = None;
                    self.selected_piece = None;
                    self.valid_moves.clear();
                    self.win_state = board.win_state();
                }
            }
        } else if self.win_state.is_none() && response.clicked_by(PointerButton::Primary) {
            if let Some(channel) = self.channel(board.turn) {
                let pos = response.interact_pointer_pos().unwrap();
                let col = ((pos.x - response.rect.min.x) / square_size).floor() as usize;
                let row = ((pos.y - response.rect.min.y) / square_size).floor() as usize;

                if col < BOARD_SIZE && row < BOARD_SIZE {
                    let target_pos = (col, row);
                    if self.selected_piece.is_none() {
                        if let Some(piece) = board.piece_at(target_pos) {
                            if piece.color == board.turn {
                                self.selected_piece = Some((col, row));
                                self.valid_moves = piece.valid_moves(&board, false);
                            }
                        }
                    } else {
                        if let Some(valid_move) =
                            self.valid_moves.iter().find(|&m| m.target == target_pos)
                        {
                            if let MoveType::Promotion(_) = valid_move.move_type {
                                self.promoting_piece = Some(valid_move.target);
                            } else {
                                channel.send(*valid_move).unwrap();
                                self.selected_piece = None;
                                self.valid_moves.clear();
                                self.win_state = board.win_state();
                            }
                        } else {
                            self.selected_piece = None;
                            self.valid_moves.clear();
                        }
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
                {
                    ui.heading(format!(
                        "{}'s turn",
                        self.board.read().unwrap().turn.readable()
                    ));
                }

                Frame::canvas(ui.style())
                    .stroke((0_f32, Color32::TRANSPARENT))
                    .fill(Color32::TRANSPARENT)
                    .show(ui, |ui| self.chessboard(ui));

                if !self.restart_modal_closed {
                    if self.win_state.is_some() {
                        Modal::new(Id::new("Winner modal")).show(ui.ctx(), |ui| {
                            ui.set_min_width(200.0);
                            match self.win_state.as_ref().unwrap() {
                                WinState::Checkmate(color) => {
                                    ui.heading(format!("{} wins!", color.readable()));
                                }
                                WinState::Stalemate => {
                                    ui.heading("Draw!");
                                }
                            }
                            let play_again_clicked = egui::Sides::new().show(
                                ui,
                                |ui| ui.button("Play again").clicked(),
                                |ui| ui.button("Close").clicked(),
                            );

                            if play_again_clicked.0 {
                                self.reset();
                                self.restart_modal_closed = true;
                            }
                            if play_again_clicked.1 {
                                self.restart_modal_closed = true;
                            }
                        });
                    }
                }
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
