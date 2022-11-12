// SPDX-License-Identifier: ISC

use curl::easy::{Handler, WriteError};
use fen::{BoardState, Color, PieceKind};
use notcurses::{Channel, Plane, Position, Rgb};
use rand::Rng;
use serde::Deserialize;
use unicode_segmentation::UnicodeSegmentation;

struct PositionOffset {
    row: u32,
    column: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PlayerKind {
    Black,
    White,
}

// Feed responses are adjacently tagged ndjson
#[derive(Deserialize, Debug)]
#[serde(tag = "t", content = "d")]
pub enum FeaturedTVGameFeed {
    #[serde(rename = "featured")]
    FeaturedTVGameSummary(FeaturedTVGameSummary),
    #[serde(rename = "fen")]
    FeaturedTVGameUpdate(FeaturedTVGameUpdate),
}

#[derive(Deserialize, Debug)]
pub struct FeaturedTVGameSummary {
    id: String,
    orientation: PlayerKind,
    players: Vec<Player>,
    fen: String,
}

#[derive(Deserialize, Debug)]
pub struct FeaturedTVGameUpdate {
    fen: String,
    #[serde(rename = "lm")]
    last_move: String,
    #[serde(rename = "wc")]
    white_clock: u32,
    #[serde(rename = "bc")]
    black_clock: u32,
}

#[derive(Deserialize, Debug)]
pub struct Player {
    color: PlayerKind,
    user: User,
    rating: u32,
    seconds: u32,
}

#[derive(Deserialize, Debug)]
pub struct User {
    name: String,
    title: Option<String>,
    id: String,
}

#[derive(Debug)]
pub struct LichessTV<'a> {
    players: Vec<Player>,
    last_move: String,
    board: BoardState,
    board_orientation: PlayerKind,
    white_clock: u32,
    black_clock: u32,
    nc_board_plane: &'a mut Plane,
}

impl<'a> LichessTV<'a> {
    pub fn new(plane: &mut Plane) -> LichessTV {
        LichessTV {
            board: BoardState::from_fen(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            )
            .unwrap(),
            last_move: String::from(""),
            white_clock: 0,
            black_clock: 0,
            players: Vec::new(),
            board_orientation: PlayerKind::White,
            nc_board_plane: plane,
        }
    }

    fn draw_chess_board(&mut self) {
        let mut pieces_board = self.nc_board_plane.new_child().unwrap();

        pieces_board.move_to(Position::from_xy(0, 0)).unwrap();

        let plane_size = self.nc_board_plane.size();
        let mut position = PositionOffset {
            row: plane_size.1 / 2 - 4,
            column: plane_size.0 / 2 - 12,
        };

        for (n, a_piece) in self.board.pieces.iter().enumerate() {
            if n % 8 == 0 {
                position.row = position.row + 1;
                position.column = plane_size.0 / 2 - 12;
            } else {
                position.column = position.column + 3;
            }

            let piece_character = match a_piece {
                Some(piece) => match piece.kind {
                    PieceKind::Pawn => match piece.color {
                        Color::White => " ♙ ",
                        Color::Black => "♟  ",
                    },
                    PieceKind::Knight => match piece.color {
                        Color::White => " ♘ ",
                        Color::Black => "♞  ",
                    },
                    PieceKind::Bishop => match piece.color {
                        Color::White => " ♗ ",
                        Color::Black => "♝  ",
                    },
                    PieceKind::Rook => match piece.color {
                        Color::White => " ♖ ",
                        Color::Black => "♜  ",
                    },
                    PieceKind::Queen => match piece.color {
                        Color::White => " ♕ ",
                        Color::Black => "♛  ",
                    },
                    PieceKind::King => match piece.color {
                        Color::White => " ♔ ",
                        Color::Black => "♚  ",
                    },
                    _ => "   ",
                },
                None => "   ",
            };

            let channel = match (n + (n / 8)) & 1 == 0 {
                true => Channel::from_rgb(Rgb::new(195, 160, 130)),
                false => Channel::from_rgb(Rgb::new(242, 225, 195)),
            };

            pieces_board.set_bg(channel);

            pieces_board
                .putstr_at_xy(
                    Some(position.column),
                    Some(position.row),
                    &piece_character,
                )
                .unwrap();
        }

        pieces_board.render().unwrap();
        self.nc_board_plane.render().unwrap();
    }
}

impl<'a> Handler for LichessTV<'a> {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        let json_data = std::str::from_utf8(data).unwrap();

        let featured_game: FeaturedTVGameFeed =
            serde_json::from_str(json_data).unwrap();

        match featured_game {
            FeaturedTVGameFeed::FeaturedTVGameSummary(summary) => {
                let mut patched_fen = String::from(summary.fen);
                patched_fen.push_str(" w c - 1 1");
                self.board = BoardState::from_fen(&patched_fen).unwrap();
                self.board_orientation = summary.orientation;
                self.players = summary.players;
            }
            FeaturedTVGameFeed::FeaturedTVGameUpdate(update) => {
                let mut patched_fen = String::from(update.fen);
                patched_fen.push_str(" c - 1 1");
                self.board = BoardState::from_fen(&patched_fen).unwrap();
                self.last_move = update.last_move;
                self.white_clock = update.white_clock;
                self.black_clock = update.black_clock;
            }
        }

        self.nc_board_plane.into_ref_mut().erase();
        self.draw_chess_board();

        Ok(data.len())
    }
}
