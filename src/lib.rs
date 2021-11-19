mod utils;

use bitflags::bitflags;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::Display;
use std::ops::{BitOr, BitOrAssign, Index, IndexMut};
use wasm_bindgen::prelude::*;

const BOARD_WIDTH: u8 = 8;
const BOARD_HEIGHT: u8 = 8;
const MOVE_LIST_CAPACITY: usize = 27;
const STARTING_PLACEMENT: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

#[derive(Debug, PartialEq, Eq)]
pub struct ChessError(ChessErrorKind, &'static str);

#[derive(Debug, PartialEq, Eq)]
enum ChessErrorKind {
    InvalidCharacter,
    InvalidString,
    IndexOutOfRange,
    InvalidPromotion,
    TargetIsNone,
    Other,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Color {
    White,
    Black,
}

impl Color {
    fn opponent(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl TryFrom<char> for Color {
    type Error = ChessError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'w' => Ok(Color::White),
            'b' => Ok(Color::Black),
            _ => Err(ChessError(
                ChessErrorKind::InvalidCharacter,
                "A Color could not be derived from the given character.",
            )),
        }
    }
}

impl TryFrom<&str> for Color {
    type Error = ChessError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 1 {
            return Err(ChessError(
                ChessErrorKind::InvalidString,
                "A Color can only be derived from a string that is one character long.",
            ));
        }

        if let Some(character) = value.chars().next() {
            return Color::try_from(character);
        }

        unreachable!()
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl TryFrom<char> for PieceKind {
    type Error = ChessError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        let value = value.to_ascii_lowercase();

        match value {
            'p' => Ok(PieceKind::Pawn),
            'n' => Ok(PieceKind::Knight),
            'b' => Ok(PieceKind::Bishop),
            'r' => Ok(PieceKind::Rook),
            'q' => Ok(PieceKind::Queen),
            'k' => Ok(PieceKind::King),
            _ => Err(ChessError(
                ChessErrorKind::InvalidCharacter,
                "A PieceType could not be derived from the given character.",
            )),
        }
    }
}

impl TryFrom<&str> for PieceKind {
    type Error = ChessError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 1 {
            return Err(ChessError(
                ChessErrorKind::InvalidString,
                "A PieceType can only be derived from a string that is one character long.",
            ));
        }

        if let Some(character) = value.chars().next() {
            return PieceKind::try_from(character);
        }

        unreachable!()
    }
}

impl<'a> From<PieceKind> for &'a str {
    fn from(value: PieceKind) -> &'a str {
        match value {
            PieceKind::Pawn => "p",
            PieceKind::Knight => "n",
            PieceKind::Bishop => "b",
            PieceKind::Rook => "r",
            PieceKind::Queen => "q",
            PieceKind::King => "k",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct Piece(Color, PieceKind);

impl TryFrom<char> for Piece {
    type Error = ChessError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'P' => Ok(Piece(Color::White, PieceKind::Pawn)),
            'N' => Ok(Piece(Color::White, PieceKind::Knight)),
            'B' => Ok(Piece(Color::White, PieceKind::Bishop)),
            'R' => Ok(Piece(Color::White, PieceKind::Rook)),
            'Q' => Ok(Piece(Color::White, PieceKind::Queen)),
            'K' => Ok(Piece(Color::White, PieceKind::King)),
            'p' => Ok(Piece(Color::Black, PieceKind::Pawn)),
            'n' => Ok(Piece(Color::Black, PieceKind::Knight)),
            'b' => Ok(Piece(Color::Black, PieceKind::Bishop)),
            'r' => Ok(Piece(Color::Black, PieceKind::Rook)),
            'q' => Ok(Piece(Color::Black, PieceKind::Queen)),
            'k' => Ok(Piece(Color::Black, PieceKind::King)),
            _ => Err(ChessError(
                ChessErrorKind::InvalidCharacter,
                "A Piece could not be derived from the given character.",
            )),
        }
    }
}

impl From<Piece> for char {
    fn from(value: Piece) -> Self {
        match value {
            Piece(Color::White, PieceKind::Pawn) => 'P',
            Piece(Color::White, PieceKind::Knight) => 'N',
            Piece(Color::White, PieceKind::Bishop) => 'B',
            Piece(Color::White, PieceKind::Rook) => 'R',
            Piece(Color::White, PieceKind::Queen) => 'Q',
            Piece(Color::White, PieceKind::King) => 'K',
            Piece(Color::Black, PieceKind::Pawn) => 'p',
            Piece(Color::Black, PieceKind::Knight) => 'n',
            Piece(Color::Black, PieceKind::Bishop) => 'b',
            Piece(Color::Black, PieceKind::Rook) => 'r',
            Piece(Color::Black, PieceKind::Queen) => 'q',
            Piece(Color::Black, PieceKind::King) => 'k',
        }
    }
}

// TODO(thismarvin): Is there a more idiomatic approach to this?
bitflags! {
    struct CastlingAbility : u8 {
        const WHITE_KINGSIDE = 1 << 0;
        const WHITE_QUEENSIDE = 1 << 1;
        const BLACK_KINGSIDE = 1 << 2;
        const BLACK_QUEENSIDE = 1 << 3;
    }
}

impl TryFrom<char> for CastlingAbility {
    type Error = ChessError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'K' => Ok(CastlingAbility::WHITE_KINGSIDE),
            'Q' => Ok(CastlingAbility::WHITE_QUEENSIDE),
            'k' => Ok(CastlingAbility::BLACK_KINGSIDE),
            'q' => Ok(CastlingAbility::BLACK_QUEENSIDE),
            _ => Err(ChessError(
                ChessErrorKind::InvalidCharacter,
                "A CastlingAbility could not be derived from the given character.",
            )),
        }
    }
}

impl TryFrom<&str> for CastlingAbility {
    type Error = ChessError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() >= 5 {
            return Err(ChessError(
                ChessErrorKind::InvalidString,
                "A CastlingAbility can only be derived from a string that is less than five characters long.",
            ));
        }

        let mut ability: Option<CastlingAbility> = None;

        for character in value.chars() {
            if let Ok(value) = CastlingAbility::try_from(character) {
                ability = if let Some(ability) = ability {
                    Some(ability | value)
                } else {
                    Some(value)
                };
            } else {
                return Err(ChessError(
                    ChessErrorKind::InvalidString,
                    "A CastlingAbility could not be constructed from the given string.",
                ));
            }
        }

        match ability {
            Some(ability) => Ok(ability),
            None => Err(ChessError(
                ChessErrorKind::InvalidString,
                "A CastlingAbility can not be constructed from an empty string.",
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Coordinate {
    A8 = 0,
    B8 = 1,
    C8 = 2,
    D8 = 3,
    E8 = 4,
    F8 = 5,
    G8 = 6,
    H8 = 7,
    A7 = 8,
    B7 = 9,
    C7 = 10,
    D7 = 11,
    E7 = 12,
    F7 = 13,
    G7 = 14,
    H7 = 15,
    A6 = 16,
    B6 = 17,
    C6 = 18,
    D6 = 19,
    E6 = 20,
    F6 = 21,
    G6 = 22,
    H6 = 23,
    A5 = 24,
    B5 = 25,
    C5 = 26,
    D5 = 27,
    E5 = 28,
    F5 = 29,
    G5 = 30,
    H5 = 31,
    A4 = 32,
    B4 = 33,
    C4 = 34,
    D4 = 35,
    E4 = 36,
    F4 = 37,
    G4 = 38,
    H4 = 39,
    A3 = 40,
    B3 = 41,
    C3 = 42,
    D3 = 43,
    E3 = 44,
    F3 = 45,
    G3 = 46,
    H3 = 47,
    A2 = 48,
    B2 = 49,
    C2 = 50,
    D2 = 51,
    E2 = 52,
    F2 = 53,
    G2 = 54,
    H2 = 55,
    A1 = 56,
    B1 = 57,
    C1 = 58,
    D1 = 59,
    E1 = 60,
    F1 = 61,
    G1 = 62,
    H1 = 63,
}

impl TryFrom<u8> for Coordinate {
    type Error = ChessError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Coordinate::A8),
            1 => Ok(Coordinate::B8),
            2 => Ok(Coordinate::C8),
            3 => Ok(Coordinate::D8),
            4 => Ok(Coordinate::E8),
            5 => Ok(Coordinate::F8),
            6 => Ok(Coordinate::G8),
            7 => Ok(Coordinate::H8),
            8 => Ok(Coordinate::A7),
            9 => Ok(Coordinate::B7),
            10 => Ok(Coordinate::C7),
            11 => Ok(Coordinate::D7),
            12 => Ok(Coordinate::E7),
            13 => Ok(Coordinate::F7),
            14 => Ok(Coordinate::G7),
            15 => Ok(Coordinate::H7),
            16 => Ok(Coordinate::A6),
            17 => Ok(Coordinate::B6),
            18 => Ok(Coordinate::C6),
            19 => Ok(Coordinate::D6),
            20 => Ok(Coordinate::E6),
            21 => Ok(Coordinate::F6),
            22 => Ok(Coordinate::G6),
            23 => Ok(Coordinate::H6),
            24 => Ok(Coordinate::A5),
            25 => Ok(Coordinate::B5),
            26 => Ok(Coordinate::C5),
            27 => Ok(Coordinate::D5),
            28 => Ok(Coordinate::E5),
            29 => Ok(Coordinate::F5),
            30 => Ok(Coordinate::G5),
            31 => Ok(Coordinate::H5),
            32 => Ok(Coordinate::A4),
            33 => Ok(Coordinate::B4),
            34 => Ok(Coordinate::C4),
            35 => Ok(Coordinate::D4),
            36 => Ok(Coordinate::E4),
            37 => Ok(Coordinate::F4),
            38 => Ok(Coordinate::G4),
            39 => Ok(Coordinate::H4),
            40 => Ok(Coordinate::A3),
            41 => Ok(Coordinate::B3),
            42 => Ok(Coordinate::C3),
            43 => Ok(Coordinate::D3),
            44 => Ok(Coordinate::E3),
            45 => Ok(Coordinate::F3),
            46 => Ok(Coordinate::G3),
            47 => Ok(Coordinate::H3),
            48 => Ok(Coordinate::A2),
            49 => Ok(Coordinate::B2),
            50 => Ok(Coordinate::C2),
            51 => Ok(Coordinate::D2),
            52 => Ok(Coordinate::E2),
            53 => Ok(Coordinate::F2),
            54 => Ok(Coordinate::G2),
            55 => Ok(Coordinate::H2),
            56 => Ok(Coordinate::A1),
            57 => Ok(Coordinate::B1),
            58 => Ok(Coordinate::C1),
            59 => Ok(Coordinate::D1),
            60 => Ok(Coordinate::E1),
            61 => Ok(Coordinate::F1),
            62 => Ok(Coordinate::G1),
            63 => Ok(Coordinate::H1),
            _ => Err(ChessError(
                ChessErrorKind::IndexOutOfRange,
                "The given index is too big to be a Coordinate.",
            )),
        }
    }
}

impl Coordinate {
    fn x(&self) -> u8 {
        (*self) as u8 % BOARD_WIDTH
    }

    fn y(&self) -> u8 {
        (*self) as u8 / BOARD_WIDTH
    }

    fn try_move(&self, dx: i8, dy: i8) -> Result<Coordinate, ChessError> {
        let x = self.x() as i8 + dx;

        if x < 0 || x >= BOARD_WIDTH as i8 {
            return Err(ChessError(
                ChessErrorKind::IndexOutOfRange,
                "The x coordinate must be within the range [0, 7]",
            ));
        }

        let y = self.y() as i8 - dy;

        if y < 0 || y >= BOARD_HEIGHT as i8 {
            return Err(ChessError(
                ChessErrorKind::IndexOutOfRange,
                "The y coordinate must be within the range [0, 7].",
            ));
        }

        Coordinate::try_from(y as u8 * BOARD_WIDTH + x as u8)
    }
}

impl TryFrom<&str> for Coordinate {
    type Error = ChessError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 2 {
            return Err(ChessError(
                ChessErrorKind::InvalidString,
                "A Coordinate can only be derived from a string that is two characters long.",
            ));
        }

        let value = value.to_lowercase();
        let mut characters = value.chars();

        match (characters.next(), characters.next()) {
            (Some(file), Some(rank)) => {
                if !file.is_ascii_alphabetic() || !rank.is_ascii_digit() {
                    return Err(ChessError(
                        ChessErrorKind::InvalidString,
                        "A Coordinate can only only be derived from a string that consists of an alphabetic ASCII character followed by a numeric ASCII character.",
                    ));
                }

                let x = file as u32 - 'a' as u32;

                if x >= BOARD_WIDTH as u32 {
                    return Err(ChessError(
                        ChessErrorKind::InvalidCharacter,
                        "The first character should be within a-h (inclusive).",
                    ));
                }

                let y = rank.to_digit(10).ok_or(ChessError(
                    ChessErrorKind::InvalidCharacter,
                    "Expected a number.",
                ))?;

                if y == 0 || y > BOARD_HEIGHT as u32 {
                    return Err(ChessError(
                        ChessErrorKind::InvalidCharacter,
                        "The second character should be within 1-8 (inclusive).",
                    ));
                }

                let y = BOARD_HEIGHT as u32 - y;

                let index = (y * BOARD_WIDTH as u32 + x) as u8;

                Ok(Coordinate::try_from(index)?)
            }
            _ => unreachable!(),
        }
    }
}

impl<'a> From<Coordinate> for &'a str {
    fn from(value: Coordinate) -> Self {
        match value {
            Coordinate::A8 => "a8",
            Coordinate::B8 => "b8",
            Coordinate::C8 => "c8",
            Coordinate::D8 => "d8",
            Coordinate::E8 => "e8",
            Coordinate::F8 => "f8",
            Coordinate::G8 => "g8",
            Coordinate::H8 => "h8",
            Coordinate::A7 => "a7",
            Coordinate::B7 => "b7",
            Coordinate::C7 => "c7",
            Coordinate::D7 => "d7",
            Coordinate::E7 => "e7",
            Coordinate::F7 => "f7",
            Coordinate::G7 => "g7",
            Coordinate::H7 => "h7",
            Coordinate::A6 => "a6",
            Coordinate::B6 => "b6",
            Coordinate::C6 => "c6",
            Coordinate::D6 => "d6",
            Coordinate::E6 => "e6",
            Coordinate::F6 => "f6",
            Coordinate::G6 => "g6",
            Coordinate::H6 => "h6",
            Coordinate::A5 => "a5",
            Coordinate::B5 => "b5",
            Coordinate::C5 => "c5",
            Coordinate::D5 => "d5",
            Coordinate::E5 => "e5",
            Coordinate::F5 => "f5",
            Coordinate::G5 => "g5",
            Coordinate::H5 => "h5",
            Coordinate::A4 => "a4",
            Coordinate::B4 => "b4",
            Coordinate::C4 => "c4",
            Coordinate::D4 => "d4",
            Coordinate::E4 => "e4",
            Coordinate::F4 => "f4",
            Coordinate::G4 => "g4",
            Coordinate::H4 => "h4",
            Coordinate::A3 => "a3",
            Coordinate::B3 => "b3",
            Coordinate::C3 => "c3",
            Coordinate::D3 => "d3",
            Coordinate::E3 => "e3",
            Coordinate::F3 => "f3",
            Coordinate::G3 => "g3",
            Coordinate::H3 => "h3",
            Coordinate::A2 => "a2",
            Coordinate::B2 => "b2",
            Coordinate::C2 => "c2",
            Coordinate::D2 => "d2",
            Coordinate::E2 => "e2",
            Coordinate::F2 => "f2",
            Coordinate::G2 => "g2",
            Coordinate::H2 => "h2",
            Coordinate::A1 => "a1",
            Coordinate::B1 => "b1",
            Coordinate::C1 => "c1",
            Coordinate::D1 => "d1",
            Coordinate::E1 => "e1",
            Coordinate::F1 => "f1",
            Coordinate::G1 => "g1",
            Coordinate::H1 => "h1",
        }
    }
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slice: &str = (*self).into();

        write!(f, "{}", slice)
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct Lan {
    start: Coordinate,
    end: Coordinate,
    promotion: Option<PieceKind>,
}

impl TryFrom<&str> for Lan {
    type Error = ChessError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() < 4 || value.len() > 5 {
            return Err(ChessError(
                ChessErrorKind::InvalidString,
                "A LAN can only be created from a string that is four or five characters long.",
            ));
        }

        let value = value.to_lowercase();
        let mut characters = value.chars();

        let start = Coordinate::try_from(
            format!(
                "{}{}",
                characters.next().unwrap_or('_'),
                characters.next().unwrap_or('_')
            )
            .as_str(),
        )?;

        let end = Coordinate::try_from(
            format!(
                "{}{}",
                characters.next().unwrap_or('_'),
                characters.next().unwrap_or('_')
            )
            .as_str(),
        )?;

        let character = characters.next();

        match character {
            Some(character) => match PieceKind::try_from(character) {
                Ok(promotion) => Ok(Lan {
                    start,
                    end,
                    promotion: Some(promotion),
                }),
                Err(error) => Err(error),
            },
            None => Ok(Lan {
                start,
                end,
                promotion: None,
            }),
        }
    }
}

impl Display for Lan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let promotion: &str = match self.promotion {
            Some(promotion) => promotion.into(),
            None => "",
        };

        write!(f, "{}{}{}", self.start, self.end, promotion)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Placement(String);

impl Default for Placement {
    fn default() -> Self {
        Placement(STARTING_PLACEMENT.into())
    }
}

impl TryFrom<&str> for Placement {
    type Error = ChessError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let ranks: Vec<&str> = value.split('/').collect();

        if ranks.len() != BOARD_HEIGHT as usize {
            return Err(ChessError(
                ChessErrorKind::InvalidString,
                "A valid Placement must consist of eight sections separated by a forward slash.",
            ));
        }

        for rank in ranks {
            let characters = rank.chars();
            let mut reach = 0;

            for character in characters {
                if let Some(digit) = character.to_digit(10) {
                    reach += digit as usize;
                    continue;
                }

                Piece::try_from(character)?;
                reach += 1;
            }

            if reach != BOARD_WIDTH as usize {
                return Err(ChessError(
                    ChessErrorKind::InvalidString,
                    "Each section of a valid Placement must add up to eight.",
                ));
            }
        }

        Ok(Placement(String::from(value)))
    }
}

impl From<Board> for Placement {
    fn from(value: Board) -> Placement {
        const LOOKUP: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

        let mut placement =
            String::with_capacity((BOARD_WIDTH * BOARD_HEIGHT + (BOARD_HEIGHT - 1)) as usize);

        let mut index = 0;
        let mut empty = 0;

        let mut sections = 0;

        for piece in value.pieces {
            if let Some(piece) = piece {
                if empty != 0 {
                    placement.push(LOOKUP[empty]);
                    empty = 0;
                }

                placement.push(piece.into());
            } else {
                empty += 1;
            }

            index += 1;

            if index == 8 {
                sections += 1;

                if empty > 0 {
                    placement.push(LOOKUP[empty]);
                    empty = 0;
                }

                if sections != BOARD_HEIGHT {
                    placement.push('/');
                }

                index = 0;
            }
        }

        Placement(placement)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Fen {
    placement: Placement,
    side_to_move: Color,
    castling_ability: Option<CastlingAbility>,
    en_passant_target: Option<Coordinate>,
    half_moves: usize,
    full_moves: usize,
}

impl Fen {
    fn apply_move(&self, lan: Lan) -> Result<Fen, ChessError> {
        let mut board = Board::from(&self.placement);

        let piece = board[lan.start];
        let piece = piece.ok_or(ChessError(
            ChessErrorKind::TargetIsNone,
            "Cannot move a piece that does not exist.",
        ))?;

        let target = board[lan.end];

        let capture = target.is_some();

        let dx = lan.end.x() as i8 - lan.start.x() as i8;
        let dy = lan.end.y() as i8 - lan.start.y() as i8;

        // Setup variables for next FEN.
        let side_to_move = match self.side_to_move {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
        let mut castling_ability = self.castling_ability;
        let mut en_passant_target = None;
        let mut half_moves = self.half_moves + 1;
        let mut full_moves = self.full_moves;

        // Keep castling rights up to date.
        if let Piece(color, PieceKind::King) = piece {
            // If the king castled then make sure to also move the rook.
            if dx.abs() == 2 {
                let y = match color {
                    Color::White => BOARD_HEIGHT - 1,
                    Color::Black => 0,
                };

                let (initial_index, final_index) = match dx.cmp(&0) {
                    // Castling king side.
                    std::cmp::Ordering::Greater => {
                        let x = BOARD_WIDTH - 1;
                        let index = y * BOARD_WIDTH + x;

                        (index, index - 2)
                    }
                    // Castling queen side.
                    std::cmp::Ordering::Less => {
                        let x = 0;
                        let index = y * BOARD_WIDTH + x;

                        (index, index + 3)
                    }
                    _ => unreachable!(),
                };

                let initial_coordinate = Coordinate::try_from(initial_index)?;
                let final_coordinate = Coordinate::try_from(final_index)?;

                board[initial_coordinate] = None;
                board[final_coordinate] = Some(Piece(color, PieceKind::Rook));
            }

            // If the king moves then remove their ability to castle.
            match color {
                Color::White => {
                    if let Some(ability) = castling_ability {
                        castling_ability = Some(
                            ability
                                & (!(CastlingAbility::WHITE_KINGSIDE
                                    | CastlingAbility::WHITE_QUEENSIDE)),
                        );
                    }
                }
                Color::Black => {
                    if let Some(ability) = castling_ability {
                        castling_ability = Some(
                            ability
                                & (!(CastlingAbility::BLACK_KINGSIDE
                                    | CastlingAbility::BLACK_QUEENSIDE)),
                        );
                    }
                }
            }
        }

        {
            let significant_rook_index = |castling_ability: CastlingAbility| {
                let (x, y) = match castling_ability {
                    CastlingAbility::WHITE_KINGSIDE => {
                        let x = BOARD_WIDTH - 1;
                        let y = BOARD_HEIGHT - 1;

                        (x, y)
                    }
                    CastlingAbility::WHITE_QUEENSIDE => {
                        let x = 0;
                        let y = BOARD_HEIGHT - 1;

                        (x, y)
                    }
                    CastlingAbility::BLACK_KINGSIDE => {
                        let x = BOARD_WIDTH - 1;
                        let y = 0;

                        (x, y)
                    }
                    CastlingAbility::BLACK_QUEENSIDE => {
                        let x = 0;
                        let y = 0;

                        (x, y)
                    }
                    _ => unreachable!(),
                };

                y * BOARD_WIDTH + x
            };

            let king_side = match self.side_to_move {
                Color::White => CastlingAbility::WHITE_KINGSIDE,
                Color::Black => CastlingAbility::BLACK_KINGSIDE,
            };
            let queen_side = match self.side_to_move {
                Color::White => CastlingAbility::WHITE_QUEENSIDE,
                Color::Black => CastlingAbility::BLACK_QUEENSIDE,
            };

            let king_side_index = significant_rook_index(king_side);
            let queen_side_index = significant_rook_index(queen_side);

            // Make sure that moving a rook affects the king's ability to castle.
            if piece.1 == PieceKind::Rook {
                if lan.start as u8 == king_side_index {
                    if let Some(ability) = castling_ability {
                        if (ability & king_side) != CastlingAbility::empty() {
                            castling_ability = Some(ability ^ king_side);
                        }
                    }
                } else if lan.start as u8 == queen_side_index {
                    if let Some(ability) = castling_ability {
                        if (ability & queen_side) != CastlingAbility::empty() {
                            castling_ability = Some(ability ^ queen_side);
                        }
                    }
                }
            }

            let king_side = match side_to_move {
                Color::White => CastlingAbility::WHITE_KINGSIDE,
                Color::Black => CastlingAbility::BLACK_KINGSIDE,
            };
            let queen_side = match side_to_move {
                Color::White => CastlingAbility::WHITE_QUEENSIDE,
                Color::Black => CastlingAbility::BLACK_QUEENSIDE,
            };

            let king_side_index = significant_rook_index(king_side);
            let queen_side_index = significant_rook_index(queen_side);

            // Capturing a rook on either corner should disable castling on that side.
            if matches!(target, Some(Piece(_, PieceKind::Rook))) {
                if lan.end as u8 == king_side_index {
                    if let Some(ability) = castling_ability {
                        if (ability & king_side) != CastlingAbility::empty() {
                            castling_ability = Some(ability ^ king_side);
                        }
                    }
                } else if lan.end as u8 == queen_side_index {
                    if let Some(ability) = castling_ability {
                        if (ability & queen_side) != CastlingAbility::empty() {
                            castling_ability = Some(ability ^ queen_side);
                        }
                    }
                }
            }
        }

        // Handle setting up a potential en passant.
        if dy.abs() == 2 && piece.1 == PieceKind::Pawn {
            let direction = -dy.signum();
            let potential_en_passant_target = lan
                .start
                .try_move(0, direction)
                .expect("A pawn that moved two squares should be able to move one.");

            // Only enable en_passant_target if an enemy pawn is in position to capture en passant.
            let mut pawns = 0;

            if let Ok(coordinate) = lan.end.try_move(-1, 0) {
                match board[coordinate] {
                    Some(Piece(color, PieceKind::Pawn)) if color == side_to_move => {
                        en_passant_target = Some(potential_en_passant_target);
                        pawns += 1;
                    }
                    _ => (),
                }
            }
            if let Ok(coordinate) = lan.end.try_move(1, 0) {
                match board[coordinate] {
                    Some(Piece(color, PieceKind::Pawn)) if color == side_to_move => {
                        en_passant_target = Some(potential_en_passant_target);
                        pawns += 1;
                    }
                    _ => (),
                }
            }

            // Taking en passant could lead to a discovered check; we need to make sure that cannot happen.
            if pawns == 1 {
                let mut kings_coordinate: Option<Coordinate> = None;
                let mut rank: [Option<Piece>; BOARD_WIDTH as usize] = [None; BOARD_WIDTH as usize];

                let y = match self.side_to_move {
                    Color::White => 4,
                    Color::Black => 3,
                };

                for x in 0..BOARD_WIDTH {
                    let index = y * BOARD_WIDTH + x;
                    let coordinate = Coordinate::try_from(index)?;
                    let target = board[coordinate];

                    match target {
                        Some(Piece(color, PieceKind::King)) if color == side_to_move => {
                            kings_coordinate = Some(coordinate);
                        }
                        _ => (),
                    }

                    rank[x as usize] = target;
                }

                if let Some(kings_coordinate) = kings_coordinate {
                    // Remove pawn from `rank` (assume opponent took en passant).
                    let x = lan.end.x();

                    if x < BOARD_WIDTH - 1 {
                        let index = x as usize + 1;

                        match rank[index] {
                            Some(Piece(color, PieceKind::Pawn)) if color == side_to_move => {
                                rank[index] = None;
                            }
                            _ => (),
                        }
                    }
                    if x > 0 {
                        let index = x as usize - 1;

                        match rank[index] {
                            Some(Piece(color, PieceKind::Pawn)) if color == side_to_move => {
                                rank[index] = None;
                            }
                            _ => (),
                        }
                    }

                    // Get direction to walk King in.
                    let mut kings_x = kings_coordinate.x() as isize;
                    let dir_x: isize = if x > kings_x as u8 { 1 } else { -1 };

                    kings_x += dir_x;

                    // Walk King and check if a Rook or Queen is in its line of sight.
                    let mut danger = false;

                    while kings_x > -1 && kings_x < BOARD_WIDTH as isize {
                        match rank[kings_x as usize] {
                            Some(Piece(color, kind)) if color == self.side_to_move => {
                                if let PieceKind::Rook | PieceKind::Queen = kind {
                                    danger = true;
                                }

                                break;
                            }
                            Some(Piece(color, _)) if color == side_to_move => {
                                break;
                            }
                            _ => (),
                        }

                        kings_x += dir_x;
                    }

                    // Taking en passant would have resulted in a discovered check; en_passant_target should be disabled.
                    if danger {
                        en_passant_target = None;
                    }
                }
            }
        }

        // Deal with an en passant (Holy hell).
        match (self.en_passant_target, piece) {
            (Some(target), Piece(_, PieceKind::Pawn)) if target == lan.end => {
                let direction: i8 = dy.signum();
                let coordinate = target.try_move(0, direction).expect(
                    "If en_passant_target is Some then there must be a enemy pawn in its position.",
                );

                board[coordinate] = None;
            }
            _ => (),
        }

        if capture || piece.1 == PieceKind::Pawn {
            half_moves = 0;
        }

        if self.side_to_move == Color::Black {
            full_moves += 1;
        }

        // Move the piece.
        board = board.apply_move(lan)?;

        let placement = Placement::from(board);

        Ok(Fen {
            placement,
            side_to_move,
            castling_ability,
            en_passant_target,
            half_moves,
            full_moves,
        })
    }
}

impl Default for Fen {
    fn default() -> Self {
        Fen {
            placement: Default::default(),
            side_to_move: Color::White,
            castling_ability: Some(
                CastlingAbility::WHITE_KINGSIDE
                    | CastlingAbility::WHITE_QUEENSIDE
                    | CastlingAbility::BLACK_KINGSIDE
                    | CastlingAbility::BLACK_QUEENSIDE,
            ),
            en_passant_target: None,
            half_moves: 0,
            full_moves: 1,
        }
    }
}

impl TryFrom<&str> for Fen {
    type Error = ChessError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let sections: Vec<&str> = value.split_whitespace().collect();

        if sections.len() != 6 {
            return Err(ChessError(
                ChessErrorKind::InvalidString,
                "A valid FEN must consist of six sections separated by whitespace.",
            ));
        }

        let placement = sections[0];
        let placement = Placement::try_from(placement)?;

        let side_to_move = sections[1];
        let side_to_move = Color::try_from(side_to_move)?;

        let castling_ability = sections[2];
        let castling_ability = (|| {
            if castling_ability == "-" {
                return Ok(None);
            }

            CastlingAbility::try_from(castling_ability).map(Some)
        })()?;

        let en_passant_target = sections[3];
        let en_passant_target = (|| {
            if en_passant_target == "-" {
                return Ok(None);
            }

            Coordinate::try_from(en_passant_target).map(Some)
        })()?;

        let half_moves = sections[4];
        let half_moves: usize = half_moves
            .parse()
            .map_err(|_| ChessError(ChessErrorKind::InvalidString, "Expected a number."))?;

        let full_moves = sections[5];
        let full_moves: usize = full_moves
            .parse()
            .map_err(|_| ChessError(ChessErrorKind::InvalidString, "Expected a number."))?;

        Ok(Fen {
            placement,
            side_to_move,
            castling_ability,
            en_passant_target,
            half_moves,
            full_moves,
        })
    }
}

struct Board {
    pieces: [Option<Piece>; (BOARD_WIDTH * BOARD_HEIGHT) as usize],
}

impl Board {
    fn apply_move(&self, lan: Lan) -> Result<Board, ChessError> {
        let mut pieces = self.pieces;

        let start = pieces[lan.start as usize];

        match start {
            Some(piece) => {
                if let Some(promotion) = lan.promotion {
                    return if piece.1 == PieceKind::Pawn {
                        pieces[lan.start as usize] = None;
                        pieces[lan.end as usize] = Some(Piece(piece.0, promotion));

                        Ok(Board { pieces })
                    } else {
                        Err(ChessError(
                            ChessErrorKind::InvalidPromotion,
                            "Only pawns can be promoted.",
                        ))
                    };
                }

                pieces[lan.start as usize] = None;
                pieces[lan.end as usize] = start;

                Ok(Board { pieces })
            }
            _ => Err(ChessError(
                ChessErrorKind::TargetIsNone,
                "Cannot move a piece that does not exist.",
            )),
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Board::from(Placement::default())
    }
}

impl<B: Borrow<Placement>> From<B> for Board {
    fn from(value: B) -> Self {
        let value = value.borrow();

        let mut pieces: [Option<Piece>; (BOARD_WIDTH * BOARD_HEIGHT) as usize] =
            [None; (BOARD_WIDTH * BOARD_HEIGHT) as usize];
        let ranks: Vec<&str> = value.0.split('/').collect();

        let mut y = 0;

        for rank in ranks {
            let characters = rank.chars();

            let mut x = 0;

            for character in characters {
                if let Some(delta) = character.to_digit(10) {
                    x += delta as usize;

                    continue;
                }

                pieces[y * BOARD_WIDTH as usize + x] = Piece::try_from(character).ok();

                x += 1;
            }

            y += 1;
        }

        Board { pieces }
    }
}

impl Index<Coordinate> for Board {
    type Output = Option<Piece>;

    fn index(&self, index: Coordinate) -> &Self::Output {
        &self.pieces[index as usize]
    }
}

impl IndexMut<Coordinate> for Board {
    fn index_mut(&mut self, index: Coordinate) -> &mut Self::Output {
        &mut self.pieces[index as usize]
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct Bitboard(u64);

impl Bitboard {
    fn empty() -> Self {
        Default::default()
    }

    fn get(&self, coordinate: Coordinate) -> bool {
        ((1 << coordinate as u64) & self.0) != 0
    }

    fn set(&mut self, coordinate: Coordinate, value: bool) {
        match value {
            true => self.0 |= 1 << coordinate as u64,
            false => self.0 &= !(1 << coordinate as u64),
        }
    }

    fn population_count(&self) -> usize {
        // Derived from "Brian Kernighan's way" mentioned here: https://www.chessprogramming.org/Population_Count
        let mut total = 0;
        let mut contents = self.0;

        while contents != 0 {
            contents &= contents - 1;

            total += 1;
        }

        total as usize
    }
}

impl Default for Bitboard {
    fn default() -> Self {
        Bitboard(0)
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;

    fn bitor(self, rhs: Self) -> Self::Output {
        let data = self.0 | rhs.0;

        Bitboard(data)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl From<Vec<Coordinate>> for Bitboard {
    fn from(value: Vec<Coordinate>) -> Self {
        let mut result = Bitboard::empty();

        for coordinate in value {
            result.set(coordinate, true);
        }

        result
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum KingSafety {
    Safe,
    Check,
    Checkmate,
    Stalemate,
}

struct Analysis {
    moves: Vec<Option<Vec<Lan>>>,
    danger_zone: Bitboard,
    king_location: Coordinate,
    king_safety: KingSafety,
}

#[derive(Default)]
pub struct State {
    fen: Fen,
    board: Board,
}

impl State {
    fn walk(&self, moves: &mut Vec<Lan>, start: Coordinate, opponent: Color, dx: i8, dy: i8) {
        let size = BOARD_WIDTH.max(BOARD_HEIGHT) as i8;

        let mut push_move = |end: Coordinate| {
            moves.push(Lan {
                start,
                end,
                promotion: None,
            });
        };

        for i in 1..size {
            if let Ok(end) = start.try_move(i * dx, i * dy) {
                match self.board[end] {
                    Some(Piece(color, _)) => {
                        if color == opponent {
                            push_move(end);
                        }

                        break;
                    }
                    None => push_move(end),
                }
            } else {
                break;
            }
        }
    }

    fn walk_dangerously(&self, danger_zone: &mut Bitboard, start: Coordinate, dx: i8, dy: i8) {
        let size = BOARD_WIDTH.max(BOARD_HEIGHT) as i8;
        let opponent = self.board[start]
            .expect("The starting Coordinate should always index a Some piece")
            .0
            .opponent();

        for i in 1..size {
            if let Ok(end) = start.try_move(i * dx, i * dy) {
                match self.board[end] {
                    Some(piece) => {
                        danger_zone.set(end, true);

                        match piece {
                            // The king should not be able to block attackers.
                            Piece(color, PieceKind::King) if color == opponent => continue,
                            _ => (),
                        }

                        break;
                    }
                    None => danger_zone.set(end, true),
                }
            } else {
                break;
            }
        }
    }

    fn generate_pseudo_legal_pawn_moves(&self, start: Coordinate) -> Vec<Lan> {
        let mut moves = Vec::with_capacity(MOVE_LIST_CAPACITY);

        let piece = self.board[start];

        let mut register_move = |end: Coordinate| {
            const PROMOTIONS: [PieceKind; 4] = [
                PieceKind::Knight,
                PieceKind::Bishop,
                PieceKind::Rook,
                PieceKind::Queen,
            ];

            if end.y() == 0 || end.y() == BOARD_HEIGHT - 1 {
                for kind in PROMOTIONS {
                    moves.push(Lan {
                        start,
                        end,
                        promotion: Some(kind),
                    });
                }
            } else {
                moves.push(Lan {
                    start,
                    end,
                    promotion: None,
                });
            }
        };

        match piece {
            Some(Piece(Color::White, PieceKind::Pawn)) => {
                // Handle advancing one square.
                if let Ok(end) = start.try_move(0, 1) {
                    if self.board[end].is_none() {
                        register_move(end);
                    }
                }

                // Handle advancing two squares (if the pawn has not moved before).
                if start.y() == BOARD_HEIGHT - 2 {
                    if let (Ok(prerequisite), Ok(end)) =
                        (start.try_move(0, 1), start.try_move(0, 2))
                    {
                        if let (None, None) = (self.board[prerequisite], self.board[end]) {
                            register_move(end);
                        }
                    }
                }

                // Handle capturing to the top left.
                if let Ok(end) = start.try_move(-1, 1) {
                    if let Some(Piece(Color::Black, _)) = self.board[end] {
                        register_move(end);
                    }
                }

                // Handle capturing to the top right.
                if let Ok(end) = start.try_move(1, 1) {
                    if let Some(Piece(Color::Black, _)) = self.board[end] {
                        register_move(end);
                    }
                }

                // Handle en passant.
                if start.y() == 3 {
                    if let Some(en_passant_target) = self.fen.en_passant_target {
                        match start.try_move(-1, 1) {
                            Ok(end) if end == en_passant_target => register_move(end),
                            _ => (),
                        }
                        match start.try_move(1, 1) {
                            Ok(end) if end == en_passant_target => register_move(end),
                            _ => (),
                        }
                    }
                }
            }
            Some(Piece(Color::Black, PieceKind::Pawn)) => {
                // Handle advancing one square.
                if let Ok(end) = start.try_move(0, -1) {
                    if self.board[end].is_none() {
                        register_move(end);
                    }
                }

                // Handle advancing two squares (if the pawn has not moved before).
                if start.y() == 1 {
                    if let (Ok(prerequisite), Ok(end)) =
                        (start.try_move(0, -1), start.try_move(0, -2))
                    {
                        if let (None, None) = (self.board[prerequisite], self.board[end]) {
                            register_move(end);
                        }
                    }
                }

                // Handle capturing to the bottom left.
                if let Ok(end) = start.try_move(-1, -1) {
                    if let Some(Piece(Color::White, _)) = self.board[end] {
                        register_move(end);
                    }
                }

                // Handle capturing to the bottom right.
                if let Ok(end) = start.try_move(1, -1) {
                    if let Some(Piece(Color::White, _)) = self.board[end] {
                        register_move(end);
                    }
                }
                {}
                // Handle en passant.
                if start.y() == BOARD_HEIGHT - 3 - 1 {
                    if let Some(en_passant_target) = self.fen.en_passant_target {
                        match start.try_move(-1, -1) {
                            Ok(end) if end == en_passant_target => register_move(end),
                            _ => (),
                        }
                        match start.try_move(1, -1) {
                            Ok(end) if end == en_passant_target => register_move(end),
                            _ => (),
                        }
                    }
                }
            }
            _ => (),
        }

        moves
    }

    fn generate_pseudo_legal_knight_moves(&self, start: Coordinate) -> Vec<Lan> {
        let mut moves = Vec::with_capacity(MOVE_LIST_CAPACITY);

        if let Some(Piece(color, PieceKind::Knight)) = self.board[start] {
            let opponent = color.opponent();

            let mut try_register_move = |dx: i8, dy: i8| {
                let mut push_move = |end: Coordinate| {
                    moves.push(Lan {
                        start,
                        end,
                        promotion: None,
                    });
                };

                if let Ok(end) = start.try_move(dx, dy) {
                    match self.board[end] {
                        Some(Piece(color, _)) if color == opponent => push_move(end),
                        None => push_move(end),
                        _ => (),
                    }
                }
            };

            try_register_move(1, 2);
            try_register_move(2, 1);
            try_register_move(2, -1);
            try_register_move(1, -2);
            try_register_move(-1, -2);
            try_register_move(-2, -1);
            try_register_move(-2, 1);
            try_register_move(-1, 2);
        }

        moves
    }

    fn generate_pseudo_legal_bishop_moves(&self, start: Coordinate) -> Vec<Lan> {
        let mut moves = Vec::with_capacity(MOVE_LIST_CAPACITY);

        if let Some(Piece(color, PieceKind::Bishop)) = self.board[start] {
            let opponent = color.opponent();

            self.walk(&mut moves, start, opponent, 1, 1);
            self.walk(&mut moves, start, opponent, 1, -1);
            self.walk(&mut moves, start, opponent, -1, -1);
            self.walk(&mut moves, start, opponent, -1, 1);
        }

        moves
    }

    fn generate_pseudo_legal_rook_moves(&self, start: Coordinate) -> Vec<Lan> {
        let mut moves = Vec::with_capacity(MOVE_LIST_CAPACITY);

        if let Some(Piece(color, PieceKind::Rook)) = self.board[start] {
            let opponent = color.opponent();

            self.walk(&mut moves, start, opponent, 0, 1);
            self.walk(&mut moves, start, opponent, 1, 0);
            self.walk(&mut moves, start, opponent, 0, -1);
            self.walk(&mut moves, start, opponent, -1, 0);
        }

        moves
    }

    fn generate_pseudo_legal_queen_moves(&self, start: Coordinate) -> Vec<Lan> {
        let mut moves = Vec::with_capacity(MOVE_LIST_CAPACITY);

        if let Some(Piece(color, PieceKind::Queen)) = self.board[start] {
            let opponent = color.opponent();

            self.walk(&mut moves, start, opponent, 0, 1);
            self.walk(&mut moves, start, opponent, 1, 1);
            self.walk(&mut moves, start, opponent, 1, 0);
            self.walk(&mut moves, start, opponent, 1, -1);
            self.walk(&mut moves, start, opponent, 0, -1);
            self.walk(&mut moves, start, opponent, -1, -1);
            self.walk(&mut moves, start, opponent, -1, 0);
            self.walk(&mut moves, start, opponent, -1, 1);
        }

        moves
    }

    fn generate_pseudo_legal_king_moves(&self, start: Coordinate) -> Vec<Lan> {
        let mut moves = Vec::with_capacity(MOVE_LIST_CAPACITY);

        let mut push_move = |end: Coordinate| {
            moves.push(Lan {
                start,
                end,
                promotion: None,
            });
        };

        if let Some(Piece(color, PieceKind::King)) = self.board[start] {
            let opponent = color.opponent();

            let mut try_register_move = |dx: i8, dy: i8| {
                if let Ok(end) = start.try_move(dx, dy) {
                    match self.board[end] {
                        Some(Piece(color, _)) if color == opponent => push_move(end),
                        None => push_move(end),
                        _ => (),
                    }
                }
            };

            try_register_move(0, 1);
            try_register_move(1, 1);
            try_register_move(1, 0);
            try_register_move(1, -1);
            try_register_move(0, -1);
            try_register_move(-1, -1);
            try_register_move(-1, 0);
            try_register_move(-1, 1);

            let king_side = match color {
                Color::White => CastlingAbility::WHITE_KINGSIDE,
                Color::Black => CastlingAbility::BLACK_KINGSIDE,
            };
            let queen_side = match color {
                Color::White => CastlingAbility::WHITE_QUEENSIDE,
                Color::Black => CastlingAbility::BLACK_QUEENSIDE,
            };

            if let Some(castling_ability) = self.fen.castling_ability {
                if (castling_ability & king_side) != CastlingAbility::empty() {
                    if let (Ok(prerequisite), Ok(end)) =
                        (start.try_move(1, 0), start.try_move(2, 0))
                    {
                        if let (None, None) = (self.board[prerequisite], self.board[end]) {
                            push_move(end);
                        }
                    }
                }

                if (castling_ability & queen_side) != CastlingAbility::empty() {
                    if let (Ok(prerequisite_a), Ok(end), Ok(prerequisite_b)) = (
                        start.try_move(-1, 0),
                        start.try_move(-2, 0),
                        start.try_move(-3, 0),
                    ) {
                        if let (None, None, None) = (
                            self.board[prerequisite_a],
                            self.board[end],
                            self.board[prerequisite_b],
                        ) {
                            push_move(end);
                        }
                    }
                }
            }
        }

        moves
    }

    fn generate_pseudo_legal_moves(&self, color: Color) -> Vec<Option<Vec<Lan>>> {
        let mut moves = vec![None; (BOARD_WIDTH * BOARD_HEIGHT) as usize];

        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                let index = (y * BOARD_WIDTH + x) as usize;
                let start = Coordinate::try_from(index as u8)
                    .expect("The given index should always be within the board's length.");

                match self.board[start] {
                    Some(Piece(temp, kind)) if temp == color => {
                        let move_list = match kind {
                            PieceKind::Pawn => self.generate_pseudo_legal_pawn_moves(start),
                            PieceKind::Knight => self.generate_pseudo_legal_knight_moves(start),
                            PieceKind::Bishop => self.generate_pseudo_legal_bishop_moves(start),
                            PieceKind::Rook => self.generate_pseudo_legal_rook_moves(start),
                            PieceKind::Queen => self.generate_pseudo_legal_queen_moves(start),
                            PieceKind::King => self.generate_pseudo_legal_king_moves(start),
                        };

                        moves[index] = Some(move_list);
                    }
                    _ => (),
                }
            }
        }

        moves
    }

    fn generate_pawn_danger_zone(&self, coordinate: Coordinate) -> Option<Bitboard> {
        match self.board[coordinate] {
            Some(Piece(color, PieceKind::Pawn)) => {
                let mut result = Bitboard::empty();

                let dy = match color {
                    Color::White => 1,
                    Color::Black => -1,
                };

                if let Ok(end) = coordinate.try_move(-1, dy) {
                    result.set(end, true);
                }
                if let Ok(end) = coordinate.try_move(1, dy) {
                    result.set(end, true);
                }

                Some(result)
            }
            _ => None,
        }
    }

    fn generate_knight_danger_zone(&self, coordinate: Coordinate) -> Option<Bitboard> {
        match self.board[coordinate] {
            Some(Piece(_, PieceKind::Knight)) => {
                let mut result = Bitboard::empty();

                let mut try_register_danger = |dx: i8, dy: i8| {
                    if let Ok(end) = coordinate.try_move(dx, dy) {
                        result.set(end, true);
                    }
                };

                try_register_danger(1, 2);
                try_register_danger(2, 1);
                try_register_danger(2, -1);
                try_register_danger(1, -2);
                try_register_danger(-1, -2);
                try_register_danger(-2, -1);
                try_register_danger(-2, 1);
                try_register_danger(-1, 2);

                Some(result)
            }
            _ => None,
        }
    }

    fn generate_bishop_danger_zone(&self, coordinate: Coordinate) -> Option<Bitboard> {
        match self.board[coordinate] {
            Some(Piece(_, PieceKind::Bishop)) => {
                let mut result = Bitboard::empty();

                self.walk_dangerously(&mut result, coordinate, 1, 1);
                self.walk_dangerously(&mut result, coordinate, 1, -1);
                self.walk_dangerously(&mut result, coordinate, -1, -1);
                self.walk_dangerously(&mut result, coordinate, -1, 1);

                Some(result)
            }
            _ => None,
        }
    }

    fn generate_rook_danger_zone(&self, coordinate: Coordinate) -> Option<Bitboard> {
        match self.board[coordinate] {
            Some(Piece(_, PieceKind::Rook)) => {
                let mut result = Bitboard::empty();

                self.walk_dangerously(&mut result, coordinate, 0, 1);
                self.walk_dangerously(&mut result, coordinate, 1, 0);
                self.walk_dangerously(&mut result, coordinate, 0, -1);
                self.walk_dangerously(&mut result, coordinate, -1, 0);

                Some(result)
            }
            _ => None,
        }
    }

    fn generate_queen_danger_zone(&self, coordinate: Coordinate) -> Option<Bitboard> {
        match self.board[coordinate] {
            Some(Piece(_, PieceKind::Queen)) => {
                let mut result = Bitboard::empty();

                self.walk_dangerously(&mut result, coordinate, 0, 1);
                self.walk_dangerously(&mut result, coordinate, 1, 1);
                self.walk_dangerously(&mut result, coordinate, 1, 0);
                self.walk_dangerously(&mut result, coordinate, 1, -1);
                self.walk_dangerously(&mut result, coordinate, 0, -1);
                self.walk_dangerously(&mut result, coordinate, -1, -1);
                self.walk_dangerously(&mut result, coordinate, -1, 0);
                self.walk_dangerously(&mut result, coordinate, -1, 1);

                Some(result)
            }
            _ => None,
        }
    }

    fn generate_king_danger_zone(&self, coordinate: Coordinate) -> Option<Bitboard> {
        match self.board[coordinate] {
            Some(Piece(_, PieceKind::King)) => {
                let mut result = Bitboard::empty();

                let mut try_register_danger = |dx: i8, dy: i8| {
                    if let Ok(end) = coordinate.try_move(dx, dy) {
                        result.set(end, true);
                    }
                };

                try_register_danger(0, 1);
                try_register_danger(1, 1);
                try_register_danger(1, 0);
                try_register_danger(1, -1);
                try_register_danger(0, -1);
                try_register_danger(-1, -1);
                try_register_danger(-1, 0);
                try_register_danger(-1, 1);

                Some(result)
            }
            _ => None,
        }
    }

    fn generate_danger_zone(&self, color: Color) -> Bitboard {
        let mut result = Bitboard::empty();

        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                let coordinate = Coordinate::try_from(y * BOARD_WIDTH + x)
                    .expect("The given index should always be within the board's length.");

                if let Some(piece) = self.board[coordinate] {
                    if piece.0 != color {
                        continue;
                    }

                    result |= match piece.1 {
                        PieceKind::Pawn => self
                            .generate_pawn_danger_zone(coordinate)
                            .unwrap_or_default(),
                        PieceKind::Knight => self
                            .generate_knight_danger_zone(coordinate)
                            .unwrap_or_default(),
                        PieceKind::Bishop => self
                            .generate_bishop_danger_zone(coordinate)
                            .unwrap_or_default(),
                        PieceKind::Rook => self
                            .generate_rook_danger_zone(coordinate)
                            .unwrap_or_default(),
                        PieceKind::Queen => self
                            .generate_queen_danger_zone(coordinate)
                            .unwrap_or_default(),
                        PieceKind::King => self
                            .generate_king_danger_zone(coordinate)
                            .unwrap_or_default(),
                    }
                }
            }
        }

        result
    }

    fn find_king(&self, color: Color) -> Option<Coordinate> {
        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                let coordinate = Coordinate::try_from(y * BOARD_WIDTH + x)
                    .expect("The given index should always be within the board's length.");

                match self.board[coordinate] {
                    Some(Piece(temp, PieceKind::King)) if temp == color => {
                        return Some(coordinate);
                    }
                    _ => (),
                }
            }
        }

        None
    }

    fn find_pins(&self, color: Color) -> Option<Bitboard> {
        let kings_coordinate = self.find_king(color)?;

        let mut result = Bitboard::empty();
        let opponent = color.opponent();

        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                let coordinate = Coordinate::try_from(y * BOARD_WIDTH + x)
                    .expect("The given index should always be within the board's length.");

                if let Some(piece) = self.board[coordinate] {
                    if piece.0 != opponent {
                        continue;
                    }

                    let direction = (|| match piece.1 {
                        PieceKind::Bishop => {
                            if coordinate.x() == kings_coordinate.x()
                                || coordinate.y() == kings_coordinate.y()
                            {
                                return None;
                            }

                            let difference_x = kings_coordinate.x() as i8 - coordinate.x() as i8;
                            let difference_y = kings_coordinate.y() as i8 - coordinate.y() as i8;

                            if difference_x.abs() != difference_y.abs() {
                                return None;
                            }

                            let x = -(coordinate.x() as i8 - kings_coordinate.x() as i8).signum();
                            let y = (coordinate.y() as i8 - kings_coordinate.y() as i8).signum();

                            Some((x, y))
                        }
                        PieceKind::Rook => {
                            if coordinate.x() != kings_coordinate.x()
                                && coordinate.y() != kings_coordinate.y()
                            {
                                return None;
                            }

                            let x = if coordinate.y() != kings_coordinate.y() {
                                0
                            } else {
                                -(coordinate.x() as i8 - kings_coordinate.x() as i8).signum()
                            };
                            let y = if coordinate.x() != kings_coordinate.x() {
                                0
                            } else {
                                (coordinate.y() as i8 - kings_coordinate.y() as i8).signum()
                            };

                            Some((x, y))
                        }
                        PieceKind::Queen => {
                            let x = if coordinate.y() != kings_coordinate.y() {
                                0
                            } else {
                                -(coordinate.x() as i8 - kings_coordinate.x() as i8).signum()
                            };
                            let y = if coordinate.x() != kings_coordinate.x() {
                                0
                            } else {
                                (coordinate.y() as i8 - kings_coordinate.y() as i8).signum()
                            };

                            if coordinate.x() != kings_coordinate.x()
                                && coordinate.y() != kings_coordinate.y()
                            {
                                let difference_x =
                                    kings_coordinate.x() as i8 - coordinate.x() as i8;
                                let difference_y =
                                    kings_coordinate.y() as i8 - coordinate.y() as i8;

                                if difference_x.abs() != difference_y.abs() {
                                    return None;
                                }

                                let x =
                                    -(coordinate.x() as i8 - kings_coordinate.x() as i8).signum();
                                let y =
                                    (coordinate.y() as i8 - kings_coordinate.y() as i8).signum();

                                return Some((x, y));
                            }

                            Some((x, y))
                        }
                        _ => None,
                    })();

                    if let Some((dx, dy)) = direction {
                        let mut has_line_of_sight = false;
                        let mut potential_pin: Option<Coordinate> = None;

                        let mut temp = coordinate.try_move(dx, dy);

                        while let Ok(coordinate) = temp {
                            temp = coordinate.try_move(dx, dy);

                            match self.board[coordinate] {
                                Some(Piece(temp, kind)) if temp == color => {
                                    if kind == PieceKind::King {
                                        has_line_of_sight = true;

                                        break;
                                    }

                                    if potential_pin.is_none() {
                                        potential_pin = Some(coordinate);

                                        continue;
                                    }

                                    if potential_pin.is_some() {
                                        break;
                                    }
                                }
                                Some(Piece(color, _)) if color == opponent => {
                                    break;
                                }
                                _ => {
                                    if (dx > 0 && coordinate.x() > kings_coordinate.x())
                                        || (dx < 0 && coordinate.x() < kings_coordinate.x())
                                        || (dy > 0 && coordinate.y() < kings_coordinate.y())
                                        || (dy < 0 && coordinate.y() > kings_coordinate.y())
                                    {
                                        break;
                                    }
                                }
                            }
                        }

                        if has_line_of_sight {
                            if let Some(coordinate) = potential_pin {
                                result.set(coordinate, true);
                            }
                        }
                    }
                }
            }
        }

        Some(result)
    }

    fn find_attackers(&self, target: Coordinate) -> Option<(Bitboard, Bitboard)> {
        let piece = self.board[target]?;
        let opponent = piece.0.opponent();

        let mut attackers = Vec::with_capacity(2);

        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                let current = Coordinate::try_from(y * BOARD_WIDTH + x)
                    .expect("The given index should always be within the board's length.");

                match self.board[current] {
                    Some(Piece(color, kind)) if color == opponent => {
                        let move_list = (|| match kind {
                            PieceKind::Pawn => {
                                if current.x() == target.x() {
                                    return None;
                                }

                                if (target.x() as i8 - current.x() as i8).abs() > 1 {
                                    return None;
                                }

                                let dy: i8 = if color == Color::White { 1 } else { -1 };

                                if let Ok(coordinate) = current.try_move(0, dy) {
                                    if coordinate.y() != target.y() {
                                        return None;
                                    }
                                }

                                Some(self.generate_pseudo_legal_pawn_moves(current))
                            }
                            PieceKind::Knight => {
                                Some(self.generate_pseudo_legal_knight_moves(current))
                            }
                            PieceKind::Bishop => {
                                let difference_x = target.x() as i8 - current.x() as i8;
                                let difference_y = target.y() as i8 - current.y() as i8;

                                if difference_x.abs() != difference_y.abs() {
                                    return None;
                                }

                                Some(self.generate_pseudo_legal_bishop_moves(current))
                            }
                            PieceKind::Rook => {
                                if current.x() != target.x() && current.y() != target.y() {
                                    return None;
                                }

                                Some(self.generate_pseudo_legal_rook_moves(current))
                            }
                            PieceKind::Queen => {
                                let difference_x = target.x() as i8 - current.x() as i8;
                                let difference_y = target.y() as i8 - current.y() as i8;

                                if difference_x.abs() == difference_y.abs()
                                    || current.x() == target.x()
                                    || current.y() == target.y()
                                {
                                    return Some(self.generate_pseudo_legal_queen_moves(current));
                                }

                                None
                            }
                            PieceKind::King => Some(self.generate_pseudo_legal_king_moves(current)),
                        })();

                        if let Some(move_list) = move_list {
                            for lan in move_list {
                                if lan.end == target {
                                    attackers.push(lan.start);
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
        }

        let mut coordinates = Bitboard::empty();
        let mut line_of_sight = Bitboard::empty();

        for coordinate in attackers {
            coordinates.set(coordinate, true);

            if let Some(Piece(_, kind)) = self.board[coordinate] {
                let direction = (|| match kind {
                    PieceKind::Bishop => {
                        let x = -(coordinate.x() as i8 - target.x() as i8).signum();
                        let y = (coordinate.y() as i8 - target.y() as i8).signum();

                        Some((x, y))
                    }
                    PieceKind::Rook => {
                        let x = if coordinate.y() != target.y() {
                            0
                        } else {
                            -(coordinate.x() as i8 - target.x() as i8).signum()
                        };

                        let y = if coordinate.x() != target.x() {
                            0
                        } else {
                            (coordinate.y() as i8 - target.y() as i8).signum()
                        };

                        Some((x, y))
                    }
                    PieceKind::Queen => {
                        if coordinate.x() != target.x() && coordinate.y() != target.y() {
                            let x = -(coordinate.x() as i8 - target.x() as i8).signum();
                            let y = (coordinate.y() as i8 - target.y() as i8).signum();

                            return Some((x, y));
                        }

                        let x = if coordinate.y() != target.y() {
                            0
                        } else {
                            -(coordinate.x() as i8 - target.x() as i8).signum()
                        };

                        let y = if coordinate.x() != target.x() {
                            0
                        } else {
                            (coordinate.y() as i8 - target.y() as i8).signum()
                        };

                        Some((x, y))
                    }
                    _ => None,
                })();

                if let Some(direction) = direction {
                    let mut temp = coordinate.try_move(direction.0, direction.1);

                    while let Ok(coordinate) = temp {
                        temp = coordinate.try_move(direction.0, direction.1);

                        if self.board[coordinate].is_none() {
                            line_of_sight.set(coordinate, true);
                        }

                        if coordinate.x() == target.x() && coordinate.y() == target.y() {
                            break;
                        }
                    }
                }
            }
        }

        Some((coordinates, line_of_sight))
    }

    fn sanitize_pinned_pawn(
        &self,
        move_list: &mut Vec<Lan>,
        kings_coordinate: Coordinate,
        coordinate: Coordinate,
    ) {
        match self.board[coordinate] {
            Some(Piece(color, kind)) if kind == PieceKind::Pawn => {
                // If a pinned pawn is diagonal to the king then its only move is capturing the attacker
                // that is pinning it.
                if coordinate.x() != kings_coordinate.x() && coordinate.y() != kings_coordinate.y()
                {
                    let ordering = match color {
                        Color::White => Ordering::Greater,
                        Color::Black => Ordering::Less,
                    };

                    // If a white pawn is pinned and below the king or a black pawn is pinned and above the
                    // king then it is not possible for it to capture the attacker that is pinning it.
                    if ordering == coordinate.y().cmp(&kings_coordinate.y()) {
                        move_list.clear();

                        return;
                    }

                    let dx = (coordinate.x() as i8 - kings_coordinate.x() as i8).signum();
                    let dy = match color {
                        Color::White => 1,
                        Color::Black => -1,
                    };

                    if let Ok(target) = coordinate.try_move(dx, dy) {
                        match self.board[target] {
                            Some(Piece(temp, _)) if temp == color.opponent() => {
                                for i in (0..move_list.len()).rev() {
                                    if move_list[i].end != target {
                                        move_list.remove(i);
                                    }
                                }
                            }
                            _ => move_list.clear(),
                        }
                    }

                    return;
                }

                // If a pinned pawn is in the same rank as the king then it cannot move.
                if coordinate.y() == kings_coordinate.y() {
                    move_list.clear();

                    return;
                }

                // If a pinned pawn is in the same file as the king then it can only move along the file.
                for i in (0..move_list.len()).rev() {
                    if move_list[i].end.x() != coordinate.x() {
                        move_list.remove(i);
                    }
                }
            }
            _ => panic!("Expected a pawn."),
        }
    }

    fn sanitize_pinned_bishop(
        &self,
        move_list: &mut Vec<Lan>,
        kings_coordinate: Coordinate,
        coordinate: Coordinate,
    ) {
        match self.board[coordinate] {
            Some(Piece(_, kind)) if kind == PieceKind::Bishop => {
                // If a pinned bishop is on the same file or rank as the king then it cannot move.
                if coordinate.x() == kings_coordinate.x() || coordinate.y() == kings_coordinate.y()
                {
                    move_list.clear();

                    return;
                }

                let dx = (coordinate.x() as i8 - kings_coordinate.x() as i8).signum();
                let dy = -(coordinate.y() as i8 - kings_coordinate.y() as i8).signum();

                let mut bitboard = Bitboard::empty();
                let mut temp = Ok(kings_coordinate);

                while let Ok(target) = temp {
                    temp = target.try_move(dx, dy);

                    bitboard.set(target, true);
                }

                // Discard any moves that are outside of the king's diagonal.
                for i in (0..move_list.len()).rev() {
                    if !bitboard.get(move_list[i].end) {
                        move_list.remove(i);
                    }
                }
            }
            _ => panic!("Expected a bishop."),
        }
    }

    fn sanitize_pinned_rook(
        &self,
        move_list: &mut Vec<Lan>,
        kings_coordinate: Coordinate,
        coordinate: Coordinate,
    ) {
        match self.board[coordinate] {
            Some(Piece(_, kind)) if kind == PieceKind::Rook => {
                // If a pinned rook is not in the same file or rank as the king then it cannot
                // move.
                if coordinate.x() != kings_coordinate.x() && coordinate.y() != kings_coordinate.y()
                {
                    move_list.clear();

                    return;
                }

                // Discard any moves that are not in the same file or rank as the king.
                for i in (0..move_list.len()).rev() {
                    let lan = move_list[i];

                    if (coordinate.x() == kings_coordinate.x()
                        && lan.end.x() != kings_coordinate.x())
                        || (coordinate.y() == kings_coordinate.y()
                            && lan.end.y() != kings_coordinate.y())
                    {
                        move_list.remove(i);
                    }
                }
            }
            _ => panic!("Expected a rook."),
        }
    }

    fn sanitize_pinned_queen(
        &self,
        move_list: &mut Vec<Lan>,
        kings_coordinate: Coordinate,
        coordinate: Coordinate,
    ) {
        match self.board[coordinate] {
            Some(Piece(_, kind)) if kind == PieceKind::Queen => {
                // If a pinned queen is in the same file or rank as the king then discard any moves
                // that are not in said file or rank.
                if coordinate.x() == kings_coordinate.x() || coordinate.y() == kings_coordinate.y()
                {
                    for i in (0..move_list.len()).rev() {
                        let lan = move_list[i];

                        if (coordinate.x() == kings_coordinate.x()
                            && lan.end.x() != kings_coordinate.x())
                            || (coordinate.y() == kings_coordinate.y()
                                && lan.end.y() != kings_coordinate.y())
                        {
                            move_list.remove(i);
                        }
                    }

                    return;
                }

                let dx = (coordinate.x() as i8 - kings_coordinate.x() as i8).signum();
                let dy = -(coordinate.y() as i8 - kings_coordinate.y() as i8).signum();

                let mut bitboard = Bitboard::empty();
                let mut temp = Ok(kings_coordinate);

                while let Ok(target) = temp {
                    temp = target.try_move(dx, dy);

                    bitboard.set(target, true);
                }

                // Discard any moves that are outside of the king's diagonal.
                for i in (0..move_list.len()).rev() {
                    if !bitboard.get(move_list[i].end) {
                        move_list.remove(i);
                    }
                }
            }
            _ => panic!("Expected a queen."),
        }
    }

    fn analyze(&self, color: Color) -> Option<Analysis> {
        let kings_coordinate = self.find_king(color)?;
        let opponent = color.opponent();

        let mut moves = self.generate_pseudo_legal_moves(color);
        let danger_zone = self.generate_danger_zone(opponent);
        let pins = self.find_pins(color)?;
        let attackers = self.find_attackers(kings_coordinate)?;

        let mut can_move = false;

        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                let index = (y * BOARD_WIDTH + x) as usize;
                let coordinate = Coordinate::try_from(index as u8)
                    .expect("The given index should always be within the board's length.");

                match self.board[coordinate] {
                    Some(Piece(temp, kind)) if temp == color => {
                        let mut move_list = moves[index]
                            .as_mut()
                            .expect("A Some piece should always have a move list.");

                        // Deal with pins.
                        if pins.get(coordinate) {
                            // If the king is under attack then a pinned piece cannot move.
                            if danger_zone.get(kings_coordinate) {
                                move_list.clear();

                                continue;
                            }

                            match kind {
                                PieceKind::Pawn => self.sanitize_pinned_pawn(
                                    &mut move_list,
                                    kings_coordinate,
                                    coordinate,
                                ),
                                PieceKind::Knight => move_list.clear(),
                                PieceKind::Bishop => self.sanitize_pinned_bishop(
                                    &mut move_list,
                                    kings_coordinate,
                                    coordinate,
                                ),
                                PieceKind::Rook => self.sanitize_pinned_rook(
                                    &mut move_list,
                                    kings_coordinate,
                                    coordinate,
                                ),
                                PieceKind::Queen => self.sanitize_pinned_queen(
                                    &mut move_list,
                                    kings_coordinate,
                                    coordinate,
                                ),
                                PieceKind::King => {
                                    panic!("It should not be possible for a king to be pinned.")
                                }
                            };

                            continue;
                        }

                        match kind {
                            PieceKind::King => {
                                const WHITE_KINGSIDE_LAN: Lan = Lan {
                                    start: Coordinate::E1,
                                    end: Coordinate::G1,
                                    promotion: None,
                                };
                                const WHITE_QUEENSIDE_LAN: Lan = Lan {
                                    start: Coordinate::E1,
                                    end: Coordinate::C1,
                                    promotion: None,
                                };
                                const BLACK_KINGSIDE_LAN: Lan = Lan {
                                    start: Coordinate::E8,
                                    end: Coordinate::G8,
                                    promotion: None,
                                };
                                const BLACK_QUEENSIDE_LAN: Lan = Lan {
                                    start: Coordinate::E8,
                                    end: Coordinate::C8,
                                    promotion: None,
                                };

                                for i in (0..move_list.len()).rev() {
                                    let lan = move_list[i];

                                    match lan {
                                        Lan {
                                            start,
                                            end,
                                            promotion: None,
                                        } if (start == Coordinate::E1
                                            && (end == Coordinate::G1
                                                || end == Coordinate::C1))
                                            || (start == Coordinate::E8
                                                && (end == Coordinate::G8
                                                    || end == Coordinate::C8)) =>
                                        {
                                            // If the king is under attack then it should not be
                                            // able to castle.
                                            if danger_zone.get(coordinate) {
                                                move_list.remove(i);

                                                continue;
                                            }

                                            // Make sure the king cannot castle through a check.
                                            let king_side = match color {
                                                Color::White => CastlingAbility::WHITE_KINGSIDE,
                                                Color::Black => CastlingAbility::BLACK_KINGSIDE,
                                            };
                                            let queen_side = match color {
                                                Color::White => CastlingAbility::WHITE_QUEENSIDE,
                                                Color::Black => CastlingAbility::BLACK_QUEENSIDE,
                                            };

                                            let king_side_lan = match color {
                                                Color::White => WHITE_KINGSIDE_LAN,
                                                Color::Black => BLACK_KINGSIDE_LAN,
                                            };
                                            let queen_side_lan = match color {
                                                Color::White => WHITE_QUEENSIDE_LAN,
                                                Color::Black => BLACK_QUEENSIDE_LAN,
                                            };

                                            if let Some(castling_ability) =
                                                self.fen.castling_ability
                                            {
                                                if (castling_ability & king_side)
                                                    != CastlingAbility::empty()
                                                    && lan == king_side_lan
                                                {
                                                    if let (Ok(one), Ok(two)) = (
                                                        coordinate.try_move(1, 0),
                                                        coordinate.try_move(2, 0),
                                                    ) {
                                                        if danger_zone.get(one)
                                                            || danger_zone.get(two)
                                                        {
                                                            move_list.remove(i);
                                                        }
                                                    }
                                                }

                                                if (castling_ability & queen_side)
                                                    != CastlingAbility::empty()
                                                    && lan == queen_side_lan
                                                {
                                                    if let (Ok(one), Ok(two)) = (
                                                        coordinate.try_move(-1, 0),
                                                        coordinate.try_move(-2, 0),
                                                    ) {
                                                        if danger_zone.get(one)
                                                            || danger_zone.get(two)
                                                        {
                                                            move_list.remove(i);
                                                        }
                                                    }
                                                }

                                                continue;
                                            }
                                        }
                                        _ => (),
                                    }

                                    // The king should not be able to walk into an attack.
                                    if danger_zone.get(lan.end) {
                                        move_list.remove(i);
                                    }
                                }
                            }
                            _ => {
                                // Moves only need to be sanitized if the king is under attack.
                                if !danger_zone.get(kings_coordinate) {
                                    can_move = true;

                                    continue;
                                }

                                // The only response to a double check is moving the king.
                                if attackers.0.population_count() >= 2 {
                                    move_list.clear();

                                    continue;
                                }

                                for i in (0..move_list.len()).rev() {
                                    let lan = move_list[i];

                                    // If the king is under attack then the only valid move is
                                    // either capturing the attacker or blocking the attacker's
                                    // line of sight towards the king.
                                    if attackers.0.get(lan.end) || attackers.1.get(lan.end) {
                                        continue;
                                    }

                                    // Check if capturing en passant captures an attacker.
                                    if let Some(en_passant_target) = self.fen.en_passant_target {
                                        if kind == PieceKind::Pawn && lan.end == en_passant_target {
                                            let dy = match color {
                                                Color::White => -1,
                                                Color::Black => 1,
                                            };

                                            if let Ok(coordinate) =
                                                en_passant_target.try_move(0, dy)
                                            {
                                                if attackers.0.get(coordinate) {
                                                    match self.board[coordinate] {
                                                        Some(Piece(temp, PieceKind::Pawn))
                                                            if temp == opponent =>
                                                        {
                                                            continue;
                                                        }
                                                        _ => (),
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    move_list.remove(i);
                                }
                            }
                        }

                        if !can_move && !move_list.is_empty() {
                            can_move = true;
                        }
                    }
                    _ => (),
                }
            }
        }

        let king_safety = {
            if danger_zone.get(kings_coordinate) {
                if can_move {
                    KingSafety::Check
                } else {
                    KingSafety::Checkmate
                }
            } else if !can_move {
                KingSafety::Stalemate
            } else {
                KingSafety::Safe
            }
        };

        Some(Analysis {
            moves,
            danger_zone,
            king_location: kings_coordinate,
            king_safety,
        })
    }
}

impl From<Fen> for State {
    fn from(value: Fen) -> Self {
        let board = Board::from(&value.placement);

        State { fen: value, board }
    }
}

pub struct Engine;

impl Engine {
    pub fn perft(state: &State, depth: u8) -> Result<u128, ChessError> {
        if depth == 0 {
            return Ok(1);
        }

        let mut total = 0;

        let analysis = state.analyze(state.fen.side_to_move).ok_or(ChessError(
            ChessErrorKind::Other,
            "Could not analyze current state.",
        ))?;

        for move_list in analysis.moves.into_iter().flatten() {
            // if let Some(move_list) = move_list {
            for lan in move_list {
                let fen = state.fen.apply_move(lan)?;
                let state = State::from(fen);

                total += Engine::perft(&state, depth - 1)?;
            }
            // }
        }

        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_from_str() {
        let coordinate = Coordinate::try_from("a1a");
        assert!(coordinate.is_err());

        let coordinate = Coordinate::try_from("a0");
        assert!(coordinate.is_err());

        let coordinate = Coordinate::try_from("a9");
        assert!(coordinate.is_err());

        let coordinate = Coordinate::try_from("m1");
        assert!(coordinate.is_err());

        let coordinate = Coordinate::try_from("_1");
        assert!(coordinate.is_err());

        let coordinate = Coordinate::try_from("holy hell");
        assert!(coordinate.is_err());

        let coordinate = Coordinate::try_from("a8");
        assert_eq!(coordinate, Ok(Coordinate::A8));

        let coordinate = Coordinate::try_from("e4");
        assert_eq!(coordinate, Ok(Coordinate::E4));

        let coordinate = Coordinate::try_from("h1");
        assert_eq!(coordinate, Ok(Coordinate::H1));
    }

    #[test]
    fn test_coordinate_try_move() -> Result<(), ChessError> {
        let coordinate = Coordinate::E2;

        let result = coordinate.try_move(4, 0);
        assert!(result.is_err());

        let result = coordinate.try_move(-7, 0);
        assert!(result.is_err());

        let result = coordinate.try_move(0, -2);
        assert!(result.is_err());

        let result = coordinate.try_move(0, -8);
        assert!(result.is_err());

        let result = coordinate.try_move(0, 2);
        assert_eq!(result, Ok(Coordinate::E4));

        let result = coordinate.try_move(0, -1);
        assert_eq!(result, Ok(Coordinate::E1));

        let result = coordinate.try_move(-1, 0);
        assert_eq!(result, Ok(Coordinate::D2));

        let result = coordinate.try_move(2, 0);
        assert_eq!(result, Ok(Coordinate::G2));

        let result = coordinate.try_move(3, 3);
        assert_eq!(result, Ok(Coordinate::H5));

        Ok(())
    }

    #[test]
    fn test_lan_from_str() -> Result<(), ChessError> {
        let lan = Lan::try_from("a1a9");
        assert!(lan.is_err());

        let lan = Lan::try_from("e2e1m");
        assert!(lan.is_err());

        let lan = Lan::try_from("a1a2");
        assert_eq!(
            lan,
            Ok(Lan {
                start: Coordinate::try_from("a1")?,
                end: Coordinate::try_from("a2")?,
                promotion: None,
            })
        );

        let lan = Lan::try_from("e7e8q");
        assert_eq!(
            lan,
            Ok(Lan {
                start: Coordinate::try_from("e7")?,
                end: Coordinate::try_from("e8")?,
                promotion: Some(PieceKind::Queen),
            })
        );

        Ok(())
    }

    #[test]
    fn test_placement_from_str() {
        let placement = Placement::try_from("what is this really called?");
        assert!(placement.is_err());

        let placement = Placement::try_from("rnbqkbnr /pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        assert!(placement.is_err());

        let placement = Placement::try_from("rnbqkbnrr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        assert!(placement.is_err());

        let placement = Placement::try_from("rnbqkbnr/pppppppp/9/8/8/8/PPPPPPPP/RNBQKBNR");
        assert!(placement.is_err());

        let placement = Placement::try_from("rnbqkbnr/pppppppp/8/8/4P4/8/PPPP1PPP/RNBQKBNR");
        assert!(placement.is_err());

        let placement = Placement::try_from("rnbq1bnr/ppppkppp/8/4p3/4P3/8/PPPPKPPP/RNBQ1BNR");
        assert!(placement.is_ok());

        let placement = Placement::try_from("rnbqkb1r/1p2pppp/p2p1n2/8/3NP3/2N5/PPP2PPP/R1BQKB1R");
        assert!(placement.is_ok());

        let placement =
            Placement::try_from("r1bqk2r/1pppbppp/p1n2n2/4p3/B3P3/5N2/PPPP1PPP/RNBQ1RK1");
        assert!(placement.is_ok());
    }

    #[test]
    fn test_fen_from_str() -> Result<(), ChessError> {
        let fen = Fen::try_from("what is a fen string for?");
        assert!(fen.is_err());

        let fen = Fen::try_from("rnbqkbnr /pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert!(fen.is_err());

        let fen = Fen::try_from("rnbqkbnrr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert!(fen.is_err());

        let fen = Fen::try_from("rnbqkbnr/pppppppp/9/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert!(fen.is_err());

        let fen = Fen::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR m KQkq - 0 1");
        assert!(fen.is_err());

        let fen = Fen::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w king - 0 1");
        assert!(fen.is_err());

        let fen = Fen::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq m1 0 1");
        assert!(fen.is_err());

        let fen = Fen::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - a 1");
        assert!(fen.is_err());

        let fen = Fen::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 a");
        assert!(fen.is_err());

        let fen = Fen::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(fen, Ok(Fen::default()));

        let fen = Fen::try_from("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
        assert_eq!(
            fen,
            Ok(Fen {
                placement: Placement("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR".into()),
                side_to_move: Color::Black,
                castling_ability: Some(
                    CastlingAbility::WHITE_KINGSIDE
                        | CastlingAbility::WHITE_QUEENSIDE
                        | CastlingAbility::BLACK_KINGSIDE
                        | CastlingAbility::BLACK_QUEENSIDE
                ),
                en_passant_target: Some(Coordinate::try_from("e3")?),
                half_moves: 0,
                full_moves: 1,
            })
        );

        let fen =
            Fen::try_from("r2qkbnr/pp1n1ppp/2p1p3/3pPb2/3P4/5N2/PPP1BPPP/RNBQ1RK1 b kq - 3 6 ");
        assert_eq!(
            fen,
            Ok(Fen {
                placement: Placement(
                    "r2qkbnr/pp1n1ppp/2p1p3/3pPb2/3P4/5N2/PPP1BPPP/RNBQ1RK1".into()
                ),
                side_to_move: Color::Black,
                castling_ability: Some(
                    CastlingAbility::BLACK_KINGSIDE | CastlingAbility::BLACK_QUEENSIDE
                ),
                en_passant_target: None,
                half_moves: 3,
                full_moves: 6,
            })
        );

        let fen =
            Fen::try_from("r4rk1/2qn1pb1/1p2p1np/3pPb2/8/1N1N2B1/PPP1B1PP/R2Q1RK1 w - - 3 17");
        assert_eq!(
            fen,
            Ok(Fen {
                placement: Placement(
                    "r4rk1/2qn1pb1/1p2p1np/3pPb2/8/1N1N2B1/PPP1B1PP/R2Q1RK1".into()
                ),
                side_to_move: Color::White,
                castling_ability: None,
                en_passant_target: None,
                half_moves: 3,
                full_moves: 17,
            })
        );

        Ok(())
    }

    #[test]
    fn test_board_from_placement() -> Result<(), ChessError> {
        let board = Board::from(Placement(
            "rnbq1bnr/ppppkppp/8/4p3/4P3/8/PPPPKPPP/RNBQ1BNR".into(),
        ));

        assert_eq!(
            board[Coordinate::E7],
            Some(Piece(Color::Black, PieceKind::King))
        );
        assert_eq!(board[Coordinate::E1], None);

        Ok(())
    }

    #[test]
    fn test_board_apply_move() -> Result<(), ChessError> {
        let board = Board::default();
        let lan = Lan::try_from("e3e4")?;
        let result = board.apply_move(lan);
        assert!(result.is_err());

        let board = Board::from(Placement("1k6/6R1/1K6/8/8/8/8/8".into()));
        let lan = Lan::try_from("g7g8q")?;
        let result = board.apply_move(lan);
        assert!(result.is_err());

        let board = Board::default();
        let lan = Lan::try_from("e2e4")?;
        let result = board.apply_move(lan);
        assert!(result.is_ok());
        let result = result?;
        assert_eq!(result[Coordinate::E2], None);
        assert_eq!(
            result[Coordinate::E4],
            Some(Piece(Color::White, PieceKind::Pawn))
        );

        let board = Board::from(Placement("8/2k1PK2/8/8/8/8/8/8".into()));
        let lan = Lan::try_from("e7e8q")?;
        let result = board.apply_move(lan);
        assert!(result.is_ok());
        let result = result?;
        assert_eq!(result[Coordinate::E7], None);
        assert_eq!(
            result[Coordinate::E8],
            Some(Piece(Color::White, PieceKind::Queen))
        );

        Ok(())
    }

    #[test]
    fn test_placement_from_board() -> Result<(), ChessError> {
        let initial = Board::default();

        let board = initial.apply_move(Lan::try_from("e2e4")?)?;
        let placement = Placement::from(board);
        assert_eq!(
            placement,
            Placement("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR".into())
        );

        let board = initial.apply_move(Lan::try_from("e2e4")?)?;
        let board = board.apply_move(Lan::try_from("c7c5")?)?;
        let board = board.apply_move(Lan::try_from("g1f3")?)?;
        let board = board.apply_move(Lan::try_from("d7d6")?)?;
        let placement = Placement::from(board);
        assert_eq!(
            placement,
            Placement("rnbqkbnr/pp2pppp/3p4/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R".into())
        );

        Ok(())
    }

    #[test]
    fn test_fen_apply_move() -> Result<(), ChessError> {
        // Advance a pawn two squares; the enemy is not in a position to take en passant.
        let fen = Fen::default();
        let result = fen.apply_move(Lan::try_from("e2e4")?);
        assert_eq!(
            result,
            Ok(Fen::try_from(
                "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
            )?)
        );

        // Advance a pawn two squares; the enemy is in a position to take en passant.
        let fen = Fen::try_from("rnbqkbnr/ppp1pppp/8/8/3p4/8/PPPPPPPP/RNBQKBNR w KQkq - 0 3")?;
        let result = fen.apply_move(Lan::try_from("e2e4")?);
        assert_eq!(
            result,
            Ok(Fen::try_from(
                "rnbqkbnr/ppp1pppp/8/8/3pP3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 3"
            )?)
        );

        // Taking en passant results in check.
        let fen = Fen::try_from("8/8/8/8/1k3p1R/8/4P3/4K3 w - - 0 1")?;
        let result = fen.apply_move(Lan::try_from("e2e4")?);
        assert_eq!(
            result,
            Ok(Fen::try_from("8/8/8/8/1k2Pp1R/8/8/4K3 b - - 0 1")?)
        );

        // Castle kingside.
        let fen =
            Fen::try_from("r1bqkbnr/pp1npppp/3p4/1Bp5/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 2 4")?;
        let result = fen.apply_move(Lan::try_from("e1g1")?);
        assert_eq!(
            result,
            Ok(Fen::try_from(
                "r1bqkbnr/pp1npppp/3p4/1Bp5/4P3/5N2/PPPP1PPP/RNBQ1RK1 b kq - 3 4"
            )?)
        );

        // The kingside rook moves; the king can no longer castle king side.
        let fen =
            Fen::try_from("r1bqkbnr/pp1npppp/3p4/1Bp5/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 2 4")?;
        let result = fen.apply_move(Lan::try_from("h1f1")?);
        assert_eq!(
            result,
            Ok(Fen::try_from(
                "r1bqkbnr/pp1npppp/3p4/1Bp5/4P3/5N2/PPPP1PPP/RNBQKR2 b Qkq - 3 4"
            )?)
        );

        // The kingside rook is captured; the king can no longer castle king side.
        let fen = Fen::try_from("rnbqkb1r/pppppppp/8/8/8/6n1/PPPPPPPP/RNBQKBNR b KQkq - 7 4")?;
        let result = fen.apply_move(Lan::try_from("g3h1")?);
        assert_eq!(
            result,
            Ok(Fen::try_from(
                "rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNn w Qkq - 0 5"
            )?)
        );

        // Promote a pawn to a queen.
        let fen = Fen::try_from("rnbqkbnr/ppppppPp/8/8/8/8/PPPPPPP1/RNBQKBNR w KQkq - 1 5")?;
        let result = fen.apply_move(Lan::try_from("g7h8q")?);
        assert_eq!(
            result,
            Ok(Fen::try_from(
                "rnbqkbnQ/pppppp1p/8/8/8/8/PPPPPPP1/RNBQKBNR b KQq - 0 5"
            )?)
        );

        Ok(())
    }

    #[test]
    fn test_state_generate_pseudo_legal_pawn_moves() -> Result<(), ChessError> {
        // Moving None should return an empty move list.
        let state = State::default();
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::E1);
        assert_eq!(move_list, vec![]);

        // A pawn that hasn't moved should be able to advance one or two squares.
        let state = State::default();
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::E2);
        assert_eq!(
            move_list,
            vec![Lan::try_from("e2e3")?, Lan::try_from("e2e4")?]
        );

        let fen = Fen::try_from("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::E7);
        assert_eq!(
            move_list,
            vec![Lan::try_from("e7e6")?, Lan::try_from("e7e5")?]
        );

        // A pawn that has already moved should only be able to advance one square.
        let fen = Fen::try_from("rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 1 2")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::E4);
        assert_eq!(move_list, vec![Lan::try_from("e4e5")?]);

        let fen = Fen::try_from("rnbqkbnr/pppp1ppp/8/4p3/8/8/PPPPPPPP/RNBQKBNR b KQkq - 1 2")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::E5);
        assert_eq!(move_list, vec![Lan::try_from("e5e4")?]);

        // Test capturing to the top left.
        let fen = Fen::try_from("r1bqkb1r/pppppppp/2n2n2/3P4/8/8/PPP1PPPP/RNBQKBNR w KQkq - 1 3")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::D5);
        assert_eq!(
            move_list,
            vec![Lan::try_from("d5d6")?, Lan::try_from("d5c6")?]
        );

        // Test capturing to the top right.
        let fen = Fen::try_from("r1bqkb1r/pppppppp/2n2n2/4P3/8/8/PPPP1PPP/RNBQKBNR w KQkq - 1 3")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::E5);
        assert_eq!(
            move_list,
            vec![Lan::try_from("e5e6")?, Lan::try_from("e5f6")?]
        );

        // Test capturing to the bottom left.
        let fen =
            Fen::try_from("rnbqkb1r/pppp1ppp/5n2/4p3/2PP4/5N2/PP2PPPP/RNBQKB1R b KQkq - 0 3")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::E5);
        assert_eq!(
            move_list,
            vec![Lan::try_from("e5e4")?, Lan::try_from("e5d4")?]
        );

        // Test capturing to the bottom right.
        let fen = Fen::try_from("rnbqkbnr/ppp1pppp/8/3p4/4P3/2N5/PPPP1PPP/R1BQKBNR b KQkq - 1 2")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::D5);
        assert_eq!(
            move_list,
            vec![Lan::try_from("d5d4")?, Lan::try_from("d5e4")?]
        );

        // Test ability to capture en passant.
        let fen = Fen::try_from("rnbqkbnr/ppppp1pp/8/4Pp2/8/8/PPPPKPPP/RNBQ1BNR w kq f6 0 4")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::E5);
        assert_eq!(
            move_list,
            vec![Lan::try_from("e5e6")?, Lan::try_from("e5f6")?]
        );

        let fen = Fen::try_from("rnbqkbnr/ppppp1pp/8/8/4Pp2/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 3")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::F4);
        assert_eq!(
            move_list,
            vec![Lan::try_from("f4f3")?, Lan::try_from("f4e3")?]
        );

        // Test promotion.
        let fen = Fen::try_from("rnbqk1nr/ppppppPp/8/6p1/8/8/PPPPPPP1/RNBQKBNR w KQkq - 1 5")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::G7);
        assert_eq!(
            move_list,
            vec![
                Lan::try_from("g7h8n")?,
                Lan::try_from("g7h8b")?,
                Lan::try_from("g7h8r")?,
                Lan::try_from("g7h8q")?
            ]
        );

        Ok(())
    }

    #[test]
    fn test_state_generate_pseudo_legal_knight_moves() -> Result<(), ChessError> {
        let state = State::default();
        let move_list = state.generate_pseudo_legal_knight_moves(Coordinate::E1);
        assert_eq!(move_list, vec![]);

        let state = State::default();
        let move_list = state.generate_pseudo_legal_knight_moves(Coordinate::G1);
        assert_eq!(
            move_list,
            vec![Lan::try_from("g1h3")?, Lan::try_from("g1f3")?]
        );

        let fen = Fen::try_from("rnbqkbnr/pppp1ppp/8/4p3/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 0 2")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_knight_moves(Coordinate::F3);
        assert_eq!(
            move_list,
            vec![
                Lan::try_from("f3g5")?,
                Lan::try_from("f3h4")?,
                Lan::try_from("f3g1")?,
                Lan::try_from("f3d4")?,
                Lan::try_from("f3e5")?,
            ]
        );

        Ok(())
    }

    #[test]
    fn test_state_generate_pseudo_legal_bishop_moves() -> Result<(), ChessError> {
        let state = State::default();
        let move_list = state.generate_pseudo_legal_bishop_moves(Coordinate::E1);
        assert_eq!(move_list, vec![]);

        let state = State::default();
        let move_list = state.generate_pseudo_legal_bishop_moves(Coordinate::F1);
        assert_eq!(move_list, vec![]);

        let fen = Fen::try_from("r1bqkbnr/pppppppp/8/1n6/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 5 4")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_bishop_moves(Coordinate::F1);
        assert_eq!(
            move_list,
            vec![
                Lan::try_from("f1e2")?,
                Lan::try_from("f1d3")?,
                Lan::try_from("f1c4")?,
                Lan::try_from("f1b5")?,
            ]
        );

        Ok(())
    }

    #[test]
    fn test_state_generate_pseudo_legal_rook_moves() -> Result<(), ChessError> {
        let state = State::default();
        let move_list = state.generate_pseudo_legal_rook_moves(Coordinate::E1);
        assert_eq!(move_list, vec![]);

        let state = State::default();
        let move_list = state.generate_pseudo_legal_rook_moves(Coordinate::H1);
        assert_eq!(move_list, vec![]);

        let fen = Fen::try_from("rnbqkb1r/pppppppp/8/8/7P/2n4R/PPPPPPP1/R1BQKBN1 w Qkq - 0 4")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_rook_moves(Coordinate::H3);
        assert_eq!(
            move_list,
            vec![
                Lan::try_from("h3h2")?,
                Lan::try_from("h3h1")?,
                Lan::try_from("h3g3")?,
                Lan::try_from("h3f3")?,
                Lan::try_from("h3e3")?,
                Lan::try_from("h3d3")?,
                Lan::try_from("h3c3")?,
            ]
        );

        Ok(())
    }

    #[test]
    fn test_state_generate_pseudo_legal_queen_moves() -> Result<(), ChessError> {
        let state = State::default();
        let move_list = state.generate_pseudo_legal_queen_moves(Coordinate::E1);
        assert_eq!(move_list, vec![]);

        let state = State::default();
        let move_list = state.generate_pseudo_legal_queen_moves(Coordinate::D1);
        assert_eq!(move_list, vec![]);

        let fen = Fen::try_from("r1bqkbnr/pppp1ppp/2n5/4p2Q/4P3/8/PPPP1PPP/RNB1KBNR w KQkq - 2 3")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_queen_moves(Coordinate::H5);
        assert_eq!(
            move_list,
            vec![
                Lan::try_from("h5h6")?,
                Lan::try_from("h5h7")?,
                Lan::try_from("h5h4")?,
                Lan::try_from("h5h3")?,
                Lan::try_from("h5g4")?,
                Lan::try_from("h5f3")?,
                Lan::try_from("h5e2")?,
                Lan::try_from("h5d1")?,
                Lan::try_from("h5g5")?,
                Lan::try_from("h5f5")?,
                Lan::try_from("h5e5")?,
                Lan::try_from("h5g6")?,
                Lan::try_from("h5f7")?,
            ]
        );

        Ok(())
    }

    #[test]
    fn test_state_generate_pseudo_legal_king_moves() -> Result<(), ChessError> {
        let state = State::default();
        let move_list = state.generate_pseudo_legal_king_moves(Coordinate::E2);
        assert_eq!(move_list, vec![]);

        let state = State::default();
        let move_list = state.generate_pseudo_legal_king_moves(Coordinate::E1);
        assert_eq!(move_list, vec![]);

        let fen = Fen::try_from("rnbqkb1r/pppp1ppp/8/4p3/4n3/4K3/PPPP1PPP/RNBQ1BNR w kq - 0 4")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_king_moves(Coordinate::E3);
        assert_eq!(
            move_list,
            vec![
                Lan::try_from("e3e4")?,
                Lan::try_from("e3f4")?,
                Lan::try_from("e3f3")?,
                Lan::try_from("e3e2")?,
                Lan::try_from("e3d3")?,
                Lan::try_from("e3d4")?,
            ]
        );

        Ok(())
    }

    #[test]
    fn test_state_generate_pseudo_legal_moves() -> Result<(), ChessError> {
        let fen = Fen::try_from("rnbq1bnr/ppppkppp/8/4p3/4P3/8/PPPPKPPP/RNBQ1BNR w - - 2 3")?;
        let state = State::from(fen);

        let moves = state.generate_pseudo_legal_moves(Color::White);
        let total_moves = moves
            .iter()
            .filter_map(|entry| entry.as_ref())
            .fold(0, |accumulator, entry| accumulator + entry.len());

        assert_eq!(total_moves, 23);

        let moves = state.generate_pseudo_legal_moves(Color::Black);
        let total_moves = moves
            .iter()
            .filter_map(|entry| entry.as_ref())
            .fold(0, |accumulator, entry| accumulator + entry.len());

        assert_eq!(total_moves, 23);

        let fen = Fen::try_from("rnbqkbnr/pp2pppp/3p4/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 3")?;
        let state = State::from(fen);

        let moves = state.generate_pseudo_legal_moves(Color::White);
        let total_moves = moves
            .iter()
            .filter_map(|entry| entry.as_ref())
            .fold(0, |accumulator, entry| accumulator + entry.len());

        assert_eq!(total_moves, 28);

        let moves = state.generate_pseudo_legal_moves(Color::Black);
        let total_moves = moves
            .iter()
            .filter_map(|entry| entry.as_ref())
            .fold(0, |accumulator, entry| accumulator + entry.len());

        assert_eq!(total_moves, 29);

        Ok(())
    }

    #[test]
    fn test_bitboard() -> Result<(), ChessError> {
        let mut bitboard = Bitboard::empty();

        bitboard.set(Coordinate::E4, true);
        bitboard.set(Coordinate::E4, true);
        bitboard.set(Coordinate::E4, true);
        assert_eq!(bitboard.get(Coordinate::E4), true);

        bitboard.set(Coordinate::E4, false);
        bitboard.set(Coordinate::E4, false);
        assert_eq!(bitboard.get(Coordinate::E4), false);

        Ok(())
    }

    #[test]
    fn test_bitboard_bit_operations() {
        let mut a = Bitboard::empty();
        let mut b = Bitboard::empty();

        a.set(Coordinate::A1, true);
        b.set(Coordinate::A2, true);

        let mut c = Bitboard::empty();
        c.set(Coordinate::A1, true);
        c.set(Coordinate::A2, true);

        let combined = a | b;

        assert_eq!(combined, c);

        let mut combined = a;
        combined |= b;

        assert_eq!(combined, c);

        let mut a = Bitboard::empty();
        a.set(Coordinate::E1, true);
        a.set(Coordinate::E2, true);
        a.set(Coordinate::E3, true);

        assert_eq!(a.population_count(), 3);
    }

    #[test]
    fn test_state_generate_pawn_danger_zone() -> Result<(), ChessError> {
        let state = State::default();

        let danger_zone = state.generate_pawn_danger_zone(Coordinate::E1);
        assert_eq!(danger_zone, None);

        let danger_zone = state.generate_pawn_danger_zone(Coordinate::E2);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::D3, true);
        expected.set(Coordinate::F3, true);

        assert_eq!(danger_zone, Some(expected));

        let fen = Fen::try_from("r1bqkbnr/pppp1ppp/2n5/4p3/4P3/3P4/PPP2PPP/RNBQKBNR w KQkq - 1 3")?;
        let state = State::from(fen);

        let danger_zone = state.generate_pawn_danger_zone(Coordinate::D3);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::C4, true);
        expected.set(Coordinate::E4, true);

        assert_eq!(danger_zone, Some(expected));

        Ok(())
    }

    #[test]
    fn test_state_generate_knight_danger_zone() -> Result<(), ChessError> {
        let state = State::default();

        let danger_zone = state.generate_knight_danger_zone(Coordinate::E1);
        assert_eq!(danger_zone, None);

        let danger_zone = state.generate_knight_danger_zone(Coordinate::G1);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::H3, true);
        expected.set(Coordinate::E2, true);
        expected.set(Coordinate::F3, true);

        assert_eq!(danger_zone, Some(expected));

        let fen = Fen::try_from("rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2")?;
        let state = State::from(fen);

        let danger_zone = state.generate_knight_danger_zone(Coordinate::F3);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::G5, true);
        expected.set(Coordinate::H4, true);
        expected.set(Coordinate::H2, true);
        expected.set(Coordinate::G1, true);
        expected.set(Coordinate::E1, true);
        expected.set(Coordinate::D2, true);
        expected.set(Coordinate::D4, true);
        expected.set(Coordinate::E5, true);

        assert_eq!(danger_zone, Some(expected));

        Ok(())
    }

    #[test]
    fn test_state_generate_bishop_danger_zone() -> Result<(), ChessError> {
        let state = State::default();

        let danger_zone = state.generate_bishop_danger_zone(Coordinate::E1);
        assert_eq!(danger_zone, None);

        let danger_zone = state.generate_bishop_danger_zone(Coordinate::F1);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::E2, true);
        expected.set(Coordinate::G2, true);

        assert_eq!(danger_zone, Some(expected));

        let fen = Fen::try_from("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")?;
        let state = State::from(fen);

        let danger_zone = state.generate_bishop_danger_zone(Coordinate::F1);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::G2, true);
        expected.set(Coordinate::E2, true);
        expected.set(Coordinate::D3, true);
        expected.set(Coordinate::C4, true);
        expected.set(Coordinate::B5, true);
        expected.set(Coordinate::A6, true);

        assert_eq!(danger_zone, Some(expected));

        Ok(())
    }

    #[test]
    fn test_state_generate_rook_danger_zone() -> Result<(), ChessError> {
        let state = State::default();

        let danger_zone = state.generate_rook_danger_zone(Coordinate::E1);
        assert_eq!(danger_zone, None);

        let danger_zone = state.generate_rook_danger_zone(Coordinate::H1);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::H2, true);
        expected.set(Coordinate::G1, true);

        assert_eq!(danger_zone, Some(expected));

        let fen = Fen::try_from("rnbqkbnr/pppppppp/8/8/7P/7R/PPPPPPP1/RNBQKBN1 w Qkq - 3 3")?;
        let state = State::from(fen);

        let danger_zone = state.generate_rook_danger_zone(Coordinate::H3);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::H4, true);
        expected.set(Coordinate::H2, true);
        expected.set(Coordinate::H1, true);
        expected.set(Coordinate::G3, true);
        expected.set(Coordinate::F3, true);
        expected.set(Coordinate::E3, true);
        expected.set(Coordinate::D3, true);
        expected.set(Coordinate::C3, true);
        expected.set(Coordinate::B3, true);
        expected.set(Coordinate::A3, true);

        assert_eq!(danger_zone, Some(expected));

        Ok(())
    }

    #[test]
    fn test_state_generate_queen_danger_zone() -> Result<(), ChessError> {
        let state = State::default();

        let danger_zone = state.generate_queen_danger_zone(Coordinate::E1);
        assert_eq!(danger_zone, None);

        let danger_zone = state.generate_queen_danger_zone(Coordinate::D1);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::D2, true);
        expected.set(Coordinate::E2, true);
        expected.set(Coordinate::E1, true);
        expected.set(Coordinate::C1, true);
        expected.set(Coordinate::C2, true);

        assert_eq!(danger_zone, Some(expected));

        let fen = Fen::try_from("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")?;
        let state = State::from(fen);

        let danger_zone = state.generate_queen_danger_zone(Coordinate::D1);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::D2, true);
        expected.set(Coordinate::E2, true);
        expected.set(Coordinate::F3, true);
        expected.set(Coordinate::G4, true);
        expected.set(Coordinate::H5, true);
        expected.set(Coordinate::E1, true);
        expected.set(Coordinate::C1, true);
        expected.set(Coordinate::C2, true);

        assert_eq!(danger_zone, Some(expected));

        Ok(())
    }

    #[test]
    fn test_state_generate_king_danger_zone() -> Result<(), ChessError> {
        let state = State::default();

        let danger_zone = state.generate_king_danger_zone(Coordinate::E2);
        assert_eq!(danger_zone, None);

        let danger_zone = state.generate_king_danger_zone(Coordinate::E1);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::E2, true);
        expected.set(Coordinate::F2, true);
        expected.set(Coordinate::F1, true);
        expected.set(Coordinate::D1, true);
        expected.set(Coordinate::D2, true);

        assert_eq!(danger_zone, Some(expected));

        Ok(())
    }

    #[test]
    fn test_state_generate_danger_zone() -> Result<(), ChessError> {
        let state = State::default();

        let danger_zone = state.generate_danger_zone(Color::White);
        assert_eq!(danger_zone.population_count(), 22);

        let danger_zone = state.generate_danger_zone(Color::Black);
        assert_eq!(danger_zone.population_count(), 22);

        let fen = Fen::try_from("rnbqkbnr/pp2pppp/3p4/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 3")?;
        let state = State::from(fen);

        let danger_zone = state.generate_danger_zone(Color::White);
        assert_eq!(danger_zone.population_count(), 31);

        let danger_zone = state.generate_danger_zone(Color::Black);
        assert_eq!(danger_zone.population_count(), 30);

        Ok(())
    }

    #[test]
    fn test_state_find_pins() -> Result<(), ChessError> {
        let fen = Fen::try_from("q3q3/1P4k1/4P1q1/5P2/1qP1KP1q/3P4/2q1P1P1/4q2q b - - 0 1")?;
        let state = State::from(fen);

        let pins = state.find_pins(Color::White);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::E6, true);
        expected.set(Coordinate::F5, true);
        expected.set(Coordinate::F4, true);
        expected.set(Coordinate::G2, true);
        expected.set(Coordinate::E2, true);
        expected.set(Coordinate::D3, true);
        expected.set(Coordinate::C4, true);
        expected.set(Coordinate::B7, true);

        assert_eq!(pins, Some(expected));

        let pins = state.find_pins(Color::Black);
        let expected = Bitboard::empty();

        assert_eq!(pins, Some(expected));

        let fen = Fen::try_from("8/1KPq2k1/8/1P6/1P2P3/8/1q6/7q w - - 0 1")?;
        let state = State::from(fen);

        let pins = state.find_pins(Color::White);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::C7, true);
        expected.set(Coordinate::E4, true);

        assert_eq!(pins, Some(expected));

        let pins = state.find_pins(Color::Black);
        let expected = Bitboard::empty();

        assert_eq!(pins, Some(expected));

        Ok(())
    }

    #[test]
    fn test_state_find_attackers() -> Result<(), ChessError> {
        let fen = Fen::try_from("8/1k6/2b5/8/4R3/8/q5K1/3R4 w - - 0 1")?;
        let state = State::from(fen);

        let attackers = state.find_attackers(Coordinate::G2);

        let expected_attackers_0 = Bitboard::from(vec![Coordinate::A2]);
        let expected_attackers_1 = Bitboard::from(vec![
            Coordinate::B2,
            Coordinate::C2,
            Coordinate::D2,
            Coordinate::E2,
            Coordinate::F2,
        ]);

        assert_eq!(
            attackers,
            Some((expected_attackers_0, expected_attackers_1))
        );

        let fen = Fen::try_from("rnb1kbnr/pp1p1ppp/2p5/q3P3/4P3/8/PPP2PPP/RNBQKBNR w KQkq - 1 4")?;
        let state = State::from(fen);

        let attackers = state.find_attackers(Coordinate::E1);

        let expected_attackers_0 = Bitboard::from(vec![Coordinate::A5]);
        let expected_attackers_1 =
            Bitboard::from(vec![Coordinate::B4, Coordinate::C3, Coordinate::D2]);

        assert_eq!(
            attackers,
            Some((expected_attackers_0, expected_attackers_1))
        );

        let fen = Fen::try_from("rnbk1b1r/pp3ppp/2p5/4q1B1/8/8/PPP2nPP/2KR1BNR b - - 1 11")?;
        let state = State::from(fen);

        let attackers = state.find_attackers(Coordinate::D8);

        let expected_attackers_0 = Bitboard::from(vec![Coordinate::G5, Coordinate::D1]);
        let expected_attackers_1 = Bitboard::from(vec![
            Coordinate::E7,
            Coordinate::F6,
            Coordinate::D7,
            Coordinate::D6,
            Coordinate::D5,
            Coordinate::D4,
            Coordinate::D3,
            Coordinate::D2,
        ]);

        assert_eq!(
            attackers,
            Some((expected_attackers_0, expected_attackers_1))
        );

        Ok(())
    }

    #[test]
    fn test_state_sanitize_pinned_pawn() -> Result<(), ChessError> {
        let fen = Fen::try_from("8/6k1/8/8/8/8/2KP2q1/8 w - - 0 1")?;
        let state = State::from(fen);

        let mut moves = state.generate_pseudo_legal_pawn_moves(Coordinate::D2);

        state.sanitize_pinned_pawn(&mut moves, Coordinate::C2, Coordinate::D2);

        assert_eq!(moves, vec![]);

        let fen = Fen::try_from("8/2k5/2q5/8/8/2P5/2K5/8 w - - 0 1")?;
        let state = State::from(fen);

        let mut moves = state.generate_pseudo_legal_pawn_moves(Coordinate::C3);

        state.sanitize_pinned_pawn(&mut moves, Coordinate::C2, Coordinate::C3);

        assert_eq!(moves, vec![Lan::try_from("c3c4")?]);

        let fen = Fen::try_from("8/1K6/8/3P4/8/8/6q1/7k w - - 0 1")?;
        let state = State::from(fen);

        let mut moves = state.generate_pseudo_legal_pawn_moves(Coordinate::D5);

        state.sanitize_pinned_pawn(&mut moves, Coordinate::B7, Coordinate::D5);

        assert_eq!(moves, vec![]);

        let fen = Fen::try_from("8/6k1/5q2/8/3P4/8/1K6/8 w - - 0 1")?;
        let state = State::from(fen);

        let mut moves = state.generate_pseudo_legal_pawn_moves(Coordinate::D4);

        state.sanitize_pinned_pawn(&mut moves, Coordinate::B2, Coordinate::D4);

        assert_eq!(moves, vec![]);

        let fen = Fen::try_from("8/6k1/8/4q3/3P4/8/1K6/8 w - - 0 1")?;
        let state = State::from(fen);

        let mut moves = state.generate_pseudo_legal_pawn_moves(Coordinate::D4);

        state.sanitize_pinned_pawn(&mut moves, Coordinate::B2, Coordinate::D4);

        assert_eq!(moves, vec![Lan::try_from("d4e5")?]);

        Ok(())
    }

    #[test]
    fn test_state_sanitize_pinned_bishop() -> Result<(), ChessError> {
        let fen = Fen::try_from("8/8/8/8/8/8/1K1B1qk1/8 w - - 0 1")?;
        let state = State::from(fen);

        let mut moves = state.generate_pseudo_legal_bishop_moves(Coordinate::D2);

        state.sanitize_pinned_bishop(&mut moves, Coordinate::B2, Coordinate::D2);

        assert_eq!(moves, vec![]);

        let fen = Fen::try_from("8/1k6/8/1q6/1B6/8/1K6/8 w - - 0 1")?;
        let state = State::from(fen);

        let mut moves = state.generate_pseudo_legal_bishop_moves(Coordinate::B4);

        state.sanitize_pinned_bishop(&mut moves, Coordinate::B2, Coordinate::B4);

        assert_eq!(moves, vec![]);

        let fen = Fen::try_from("8/6k1/8/4q3/8/2B5/1K6/8 w - - 0 1")?;
        let state = State::from(fen);

        let mut moves = state.generate_pseudo_legal_bishop_moves(Coordinate::C3);

        state.sanitize_pinned_bishop(&mut moves, Coordinate::B2, Coordinate::C3);

        assert_eq!(moves, vec![Lan::try_from("c3d4")?, Lan::try_from("c3e5")?]);

        Ok(())
    }

    #[test]
    fn test_state_sanitize_pinned_rook() -> Result<(), ChessError> {
        let fen = Fen::try_from("8/6k1/5q2/8/3R4/8/1K6/8 w - - 0 1")?;
        let state = State::from(fen);

        let mut moves = state.generate_pseudo_legal_rook_moves(Coordinate::D4);

        state.sanitize_pinned_rook(&mut moves, Coordinate::B2, Coordinate::D4);

        assert_eq!(moves, vec![]);

        let fen = Fen::try_from("8/1k6/1q6/8/1R6/8/1K6/8 w - - 0 1")?;
        let state = State::from(fen);

        let mut moves = state.generate_pseudo_legal_rook_moves(Coordinate::B4);

        state.sanitize_pinned_rook(&mut moves, Coordinate::B2, Coordinate::B4);

        assert_eq!(
            moves,
            vec![
                Lan::try_from("b4b5")?,
                Lan::try_from("b4b6")?,
                Lan::try_from("b4b3")?,
            ]
        );

        let fen = Fen::try_from("8/8/8/8/8/8/1K1R1qk1/8 w - - 0 1")?;
        let state = State::from(fen);

        let mut moves = state.generate_pseudo_legal_rook_moves(Coordinate::D2);

        state.sanitize_pinned_rook(&mut moves, Coordinate::B2, Coordinate::D2);

        assert_eq!(
            moves,
            vec![
                Lan::try_from("d2e2")?,
                Lan::try_from("d2f2")?,
                Lan::try_from("d2c2")?,
            ]
        );

        Ok(())
    }

    #[test]
    fn test_state_sanitize_pinned_queen() -> Result<(), ChessError> {
        let fen = Fen::try_from("8/8/8/8/8/8/1K1Q1qk1/8 w - - 0 1")?;
        let state = State::from(fen);

        let mut moves = state.generate_pseudo_legal_queen_moves(Coordinate::D2);

        state.sanitize_pinned_queen(&mut moves, Coordinate::B2, Coordinate::D2);

        assert_eq!(
            moves,
            vec![
                Lan::try_from("d2e2")?,
                Lan::try_from("d2f2")?,
                Lan::try_from("d2c2")?,
            ]
        );

        let fen = Fen::try_from("8/1k6/1q6/8/1Q6/8/1K6/8 w - - 0 1")?;
        let state = State::from(fen);

        let mut moves = state.generate_pseudo_legal_queen_moves(Coordinate::B4);

        state.sanitize_pinned_queen(&mut moves, Coordinate::B2, Coordinate::B4);

        assert_eq!(
            moves,
            vec![
                Lan::try_from("b4b5")?,
                Lan::try_from("b4b6")?,
                Lan::try_from("b4b3")?,
            ]
        );

        let fen = Fen::try_from("8/6k1/5q2/8/3Q4/8/1K6/8 w - - 0 1")?;
        let state = State::from(fen);

        let mut moves = state.generate_pseudo_legal_queen_moves(Coordinate::D4);

        state.sanitize_pinned_queen(&mut moves, Coordinate::B2, Coordinate::D4);

        assert_eq!(
            moves,
            vec![
                Lan::try_from("d4e5")?,
                Lan::try_from("d4f6")?,
                Lan::try_from("d4c3")?,
            ]
        );

        Ok(())
    }

    #[test]
    fn test_state_analyze() -> Result<(), ChessError> {
        let count_moves = |analysis: Analysis| {
            analysis
                .moves
                .iter()
                .filter_map(|entry| entry.as_ref())
                .fold(0, |accumulator, entry| accumulator + entry.len())
        };

        let fen = Fen::default();
        let state = State::from(fen);

        let analysis = state.analyze(Color::White).ok_or(ChessError(
            ChessErrorKind::Other,
            "Could not analyze the given state.",
        ))?;

        assert_eq!(analysis.king_safety, KingSafety::Safe);
        assert_eq!(count_moves(analysis), 20);

        let fen = Fen::try_from("r2qnrk1/3nbppp/3pb3/5PP1/p2NP3/4B3/PPpQ3P/1K1R1B1R w - - 0 19")?;
        let state = State::from(fen);

        let analysis = state.analyze(Color::White).ok_or(ChessError(
            ChessErrorKind::Other,
            "Could not analyze the given state.",
        ))?;

        assert_eq!(analysis.king_safety, KingSafety::Check);
        assert_eq!(count_moves(analysis), 5);

        let fen = Fen::try_from("2r4k/4bppp/3p4/4nPP1/1n1Bq2P/1p5R/1Q1RB3/2K5 w - - 2 35")?;
        let state = State::from(fen);

        let analysis = state.analyze(Color::White).ok_or(ChessError(
            ChessErrorKind::Other,
            "Could not analyze the given state.",
        ))?;

        assert_eq!(analysis.king_safety, KingSafety::Check);
        assert_eq!(count_moves(analysis), 8);

        let fen = Fen::try_from("8/8/8/3k3r/2Pp4/8/1K6/8 b - c3 0 1")?;
        let state = State::from(fen);

        let analysis = state.analyze(Color::Black).ok_or(ChessError(
            ChessErrorKind::Other,
            "Could not analyze the given state.",
        ))?;

        assert_eq!(analysis.king_safety, KingSafety::Check);
        assert_eq!(count_moves(analysis), 8);

        let fen = Fen::try_from("r1bqkbnr/pppp1Qpp/8/4p3/2BnP3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 4")?;
        let state = State::from(fen);

        let analysis = state.analyze(Color::Black).ok_or(ChessError(
            ChessErrorKind::Other,
            "Could not analyze the given state.",
        ))?;

        assert_eq!(analysis.king_safety, KingSafety::Checkmate);
        assert_eq!(count_moves(analysis), 0);

        let fen = Fen::try_from("k7/2Q5/1K6/8/8/8/8/8 b - - 0 1")?;
        let state = State::from(fen);

        let analysis = state.analyze(Color::Black).ok_or(ChessError(
            ChessErrorKind::Other,
            "Could not analyze the given state.",
        ))?;

        assert_eq!(analysis.king_safety, KingSafety::Stalemate);
        assert_eq!(count_moves(analysis), 0);

        let fen = Fen::try_from("rnbqk1nr/pppp1ppp/4p3/8/1b6/3P4/PPPKPPPP/RNBQ1BNR w kq - 2 3")?;
        let state = State::from(fen);

        let analysis = state.analyze(Color::White).ok_or(ChessError(
            ChessErrorKind::Other,
            "Could not analyze the given state.",
        ))?;

        assert_eq!(analysis.king_safety, KingSafety::Check);
        assert_eq!(count_moves(analysis), 3);

        Ok(())
    }
}
