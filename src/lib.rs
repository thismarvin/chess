mod utils;

use bitflags::bitflags;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::Display;
use std::ops::{BitOr, BitOrAssign, Index, IndexMut};
use wasm_bindgen::prelude::*;

const BOARD_WIDTH: u8 = 8;
const BOARD_HEIGHT: u8 = 8;
const STARTING_PLACEMENT: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
const CHECKMATE_EVALUATION: i16 = i16::MAX - 42;

#[derive(Debug, PartialEq, Eq)]
enum ChessErrorKind {
    InvalidCharacter,
    InvalidString,
    IndexOutOfRange,
    InvalidPromotion,
    TargetIsNone,
    Other,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ChessError(ChessErrorKind, &'static str);

impl std::error::Error for ChessError {}

impl Display for ChessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.1).fmt(f)
    }
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

impl From<Color> for &str {
    fn from(value: Color) -> Self {
        match value {
            Color::White => "w",
            Color::Black => "b",
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", <&str>::from(*self))
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

impl PieceKind {
    fn value(&self) -> i16 {
        match self {
            PieceKind::Pawn => 100,
            PieceKind::Bishop => 300,
            PieceKind::Knight => 300,
            PieceKind::Rook => 500,
            PieceKind::Queen => 900,
            PieceKind::King => 0,
        }
    }
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

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", char::from(*self))
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

impl From<CastlingAbility> for String {
    fn from(value: CastlingAbility) -> Self {
        let mut result: Option<String> = None;

        if value.contains(CastlingAbility::WHITE_KINGSIDE) {
            if let Some(contents) = result.as_mut() {
                contents.push('K');
            } else {
                result = Some(String::from("K"));
            }
        }

        if value.contains(CastlingAbility::WHITE_QUEENSIDE) {
            if let Some(contents) = result.as_mut() {
                contents.push('Q');
            } else {
                result = Some(String::from("Q"));
            }
        }

        if value.contains(CastlingAbility::BLACK_KINGSIDE) {
            if let Some(contents) = result.as_mut() {
                contents.push('k');
            } else {
                result = Some(String::from("k"));
            }
        }

        if value.contains(CastlingAbility::BLACK_QUEENSIDE) {
            if let Some(contents) = result.as_mut() {
                contents.push('q');
            } else {
                result = Some(String::from("q"));
            }
        }

        result.unwrap_or_else(|| String::from("-"))
    }
}

impl Display for CastlingAbility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(*self))
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

impl From<Lan> for String {
    fn from(value: Lan) -> Self {
        let promotion: &str = match value.promotion {
            Some(promotion) => promotion.into(),
            None => "",
        };

        format!("{}{}{}", value.start, value.end, promotion)
    }
}

impl Display for Lan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(*self))
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

impl Display for Placement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
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
        let castling_ability = if castling_ability == "-" {
            Ok(None)
        } else {
            CastlingAbility::try_from(castling_ability).map(Some)
        }?;

        let en_passant_target = sections[3];
        let en_passant_target = if en_passant_target == "-" {
            Ok(None)
        } else {
            Coordinate::try_from(en_passant_target).map(Some)
        }?;

        let half_moves = sections[4];
        let half_moves: usize = half_moves
            .parse()
            .map_err(|_| ChessError(ChessErrorKind::InvalidString, "Expected a number."))?;

        let full_moves = sections[5];
        let full_moves: usize = full_moves
            .parse()
            .map_err(|_| ChessError(ChessErrorKind::InvalidString, "Expected a number."))?;

        // At a surface level the string appears to be a valid Fen; however, there are still a
        // couple of edge cases that may invalidate the fen string.

        // Make sure there is exactly one white and black king.
        let mut contains_white_king = false;
        let mut contains_black_king = false;

        for char in sections[0].chars() {
            match char {
                'K' => {
                    if !contains_white_king {
                        contains_white_king = true;
                    } else {
                        return Err(ChessError(
                            ChessErrorKind::Other,
                            "A valid Fen should only have one white king.",
                        ));
                    }
                }
                'k' => {
                    if !contains_black_king {
                        contains_black_king = true;
                    } else {
                        return Err(ChessError(
                            ChessErrorKind::Other,
                            "A valid Fen should only have one black king.",
                        ));
                    }
                }
                _ => (),
            }
        }

        if !contains_white_king || !contains_black_king {
            return Err(ChessError(
                ChessErrorKind::Other,
                "Expected exactly one white and black king.",
            ));
        }

        let board = Board::from(placement.clone());

        // Make sure the castling ability adds up.
        if let Some(castling_ability) = castling_ability {
            if !(castling_ability
                & (CastlingAbility::WHITE_KINGSIDE | CastlingAbility::WHITE_QUEENSIDE))
                .is_empty()
            {
                match board[Coordinate::E1] {
                    Some(Piece(Color::White, PieceKind::King)) => (),
                    _ => {
                        return Err(ChessError(
                            ChessErrorKind::Other,
                            "The king must be in its starting square if it can castle.",
                        ))
                    }
                }

                if !(castling_ability & CastlingAbility::WHITE_KINGSIDE).is_empty() {
                    match board[Coordinate::H1] {
                        Some(Piece(Color::White, PieceKind::Rook)) => (),
                        _ => {
                            return Err(ChessError(
                                ChessErrorKind::Other,
                                "The rook is not in the correct position to castle kingside.",
                            ))
                        }
                    }
                }

                if !(castling_ability & CastlingAbility::WHITE_QUEENSIDE).is_empty() {
                    match board[Coordinate::A1] {
                        Some(Piece(Color::White, PieceKind::Rook)) => (),
                        _ => {
                            return Err(ChessError(
                                ChessErrorKind::Other,
                                "The rook is not in the correct position to castle queenside.",
                            ))
                        }
                    }
                }
            }

            if !(castling_ability
                & (CastlingAbility::BLACK_KINGSIDE | CastlingAbility::BLACK_QUEENSIDE))
                .is_empty()
            {
                match board[Coordinate::E8] {
                    Some(Piece(Color::Black, PieceKind::King)) => (),
                    _ => {
                        return Err(ChessError(
                            ChessErrorKind::Other,
                            "The king must be in its starting square if it can castle.",
                        ))
                    }
                }

                if !(castling_ability & CastlingAbility::BLACK_KINGSIDE).is_empty() {
                    match board[Coordinate::H8] {
                        Some(Piece(Color::Black, PieceKind::Rook)) => (),
                        _ => {
                            return Err(ChessError(
                                ChessErrorKind::Other,
                                "The rook is not in the correct position to castle kingside.",
                            ))
                        }
                    }
                }

                if !(castling_ability & CastlingAbility::BLACK_QUEENSIDE).is_empty() {
                    match board[Coordinate::A8] {
                        Some(Piece(Color::Black, PieceKind::Rook)) => (),
                        _ => {
                            return Err(ChessError(
                                ChessErrorKind::Other,
                                "The rook is not in the correct position to castle queenside.",
                            ))
                        }
                    }
                }
            }
        }

        if let Some(en_passant_target) = en_passant_target {
            // Make sure the en passant target is in the correct rank.
            match en_passant_target.y() {
                2 | 5 => (),
                _ => {
                    return Err(ChessError(
                        ChessErrorKind::Other,
                        "An en passant target must either be in rank three or six.",
                    ))
                }
            }

            let dy = match side_to_move {
                Color::White => -1,
                Color::Black => 1,
            };

            // Make sure a pawn is in position to capture the en passant target.
            let left = en_passant_target.try_move(-1, dy).expect("The en passant target should always be in position where the Coordinate below it is valid.");
            let right = en_passant_target.try_move(1, dy).expect("The en passant target should always be in position where the Coordinate above it is valid.");

            let mut valid_attacker = false;

            match board[left] {
                Some(Piece(color, PieceKind::Pawn)) if color == side_to_move => {
                    valid_attacker = true;
                }
                _ => (),
            }
            match board[right] {
                Some(Piece(color, PieceKind::Pawn)) if color == side_to_move => {
                    valid_attacker = true;
                }
                _ => (),
            }

            if !valid_attacker {
                return Err(ChessError(
                    ChessErrorKind::Other,
                    "A pawn must be in position to capture the en passant target.",
                ));
            }
        }

        // Make sure the other king cannot immediately be captured.
        let danger_zone = board.generate_danger_zone(side_to_move);
        let kings_coordinate = board
            .find_king(side_to_move.opponent())
            .expect("A valid Fen should always have one white and black king.");

        if danger_zone.get(kings_coordinate) {
            return Err(ChessError(
                ChessErrorKind::Other,
                "The opponent's king should not be under attack.",
            ));
        }

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

impl From<&Fen> for String {
    fn from(value: &Fen) -> Self {
        let castling_ability = value
            .castling_ability
            .map(String::from)
            .unwrap_or_else(|| String::from("-"));

        let en_passant_target = value.en_passant_target.map(<&str>::from).unwrap_or("-");

        format!(
            "{} {} {} {} {} {}",
            value.placement,
            value.side_to_move,
            castling_ability,
            en_passant_target,
            value.half_moves,
            value.full_moves
        )
    }
}

impl Display for Fen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

#[derive(Debug, PartialEq, Eq)]
enum MoveModifier {
    Castle,
    EnPassant,
    Promotion,
}

#[derive(Debug, PartialEq, Eq)]
struct MoveUndoer {
    lan: Lan,
    /// The Piece that previously occupied the square.
    previous: Option<Piece>,
    modifer: Option<MoveModifier>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Board {
    pieces: [Option<Piece>; (BOARD_WIDTH * BOARD_HEIGHT) as usize],
}

impl Board {
    fn make_move(&mut self, lan: Lan) -> Result<MoveUndoer, ChessError> {
        let start = self.pieces[lan.start as usize];
        let previous = self.pieces[lan.end as usize];

        let dx = lan.end.x() as i8 - lan.start.x() as i8;
        let dy = lan.end.y() as i8 - lan.start.y() as i8;

        match start {
            Some(piece) => {
                match piece {
                    Piece(color, PieceKind::Pawn) => {
                        if let Some(promotion) = lan.promotion {
                            self.pieces[lan.start as usize] = None;
                            self.pieces[lan.end as usize] = Some(Piece(color, promotion));

                            return Ok(MoveUndoer {
                                lan,
                                previous,
                                modifer: Some(MoveModifier::Promotion),
                            });
                        }

                        // Deal with an en passant (Holy hell).
                        if dx != 0 && previous.is_none() {
                            let direction: i8 = dy.signum();
                            let coordinate = lan.end.try_move(0, direction).expect(
                                    "If a pawn captured en passant then the coordinate above and below the target should always be valid.",
                                );

                            self.pieces[coordinate as usize] = None;

                            self.pieces[lan.start as usize] = None;
                            self.pieces[lan.end as usize] = start;

                            return Ok(MoveUndoer {
                                lan,
                                previous,
                                modifer: Some(MoveModifier::EnPassant),
                            });
                        }

                        self.pieces[lan.start as usize] = None;
                        self.pieces[lan.end as usize] = start;

                        Ok(MoveUndoer {
                            lan,
                            previous,
                            modifer: None,
                        })
                    }
                    Piece(color, PieceKind::King) => {
                        // If the king castled then make sure to also move the rook.
                        if dx.abs() == 2 {
                            let y = match color {
                                Color::White => BOARD_HEIGHT - 1,
                                Color::Black => 0,
                            };

                            let (rook_start, rook_end) = match dx.cmp(&0) {
                                // Castling king side.
                                Ordering::Greater => {
                                    let x = BOARD_WIDTH - 1;
                                    let index = y * BOARD_WIDTH + x;

                                    (index, index - 2)
                                }
                                // Castling queen side.
                                Ordering::Less => {
                                    let x = 0;
                                    let index = y * BOARD_WIDTH + x;

                                    (index, index + 3)
                                }
                                _ => unreachable!(),
                            };

                            self.pieces[rook_start as usize] = None;
                            self.pieces[rook_end as usize] = Some(Piece(color, PieceKind::Rook));

                            self.pieces[lan.start as usize] = None;
                            self.pieces[lan.end as usize] = start;

                            return Ok(MoveUndoer {
                                lan,
                                previous,
                                modifer: Some(MoveModifier::Castle),
                            });
                        }

                        self.pieces[lan.start as usize] = None;
                        self.pieces[lan.end as usize] = start;

                        Ok(MoveUndoer {
                            lan,
                            previous,
                            modifer: None,
                        })
                    }
                    _ => {
                        if lan.promotion.is_some() {
                            return Err(ChessError(
                                ChessErrorKind::InvalidPromotion,
                                "Only pawns can be promoted.",
                            ));
                        }

                        self.pieces[lan.start as usize] = None;
                        self.pieces[lan.end as usize] = start;

                        Ok(MoveUndoer {
                            lan,
                            previous,
                            modifer: None,
                        })
                    }
                }
            }
            _ => Err(ChessError(
                ChessErrorKind::TargetIsNone,
                "Cannot move a piece that does not exist.",
            )),
        }
    }

    fn unmake_move(&mut self, undoer: MoveUndoer) {
        let piece = self.pieces[undoer.lan.end as usize];

        self.pieces[undoer.lan.start as usize] = piece;
        self.pieces[undoer.lan.end as usize] = undoer.previous;

        if let Some(modifier) = undoer.modifer {
            let piece =
                piece.expect("When unmaking a move a Lan's end should always index a Some Piece.");

            match modifier {
                MoveModifier::Castle => {
                    let dx = undoer.lan.end.x() as i8 - undoer.lan.start.x() as i8;

                    let y = match piece.0 {
                        Color::White => BOARD_HEIGHT - 1,
                        Color::Black => 0,
                    };

                    let (rook_start, rook_end) = match dx.cmp(&0) {
                        // Castling king side.
                        Ordering::Greater => {
                            let x = BOARD_WIDTH - 1;
                            let index = y * BOARD_WIDTH + x;

                            (index, index - 2)
                        }
                        // Castling queen side.
                        Ordering::Less => {
                            let x = 0;
                            let index = y * BOARD_WIDTH + x;

                            (index, index + 3)
                        }
                        _ => unreachable!(),
                    };

                    self.pieces[rook_start as usize] = Some(Piece(piece.0, PieceKind::Rook));
                    self.pieces[rook_end as usize] = None;
                }
                MoveModifier::EnPassant => {
                    let dy = undoer.lan.end.y() as i8 - undoer.lan.start.y() as i8;
                    let direction = dy.signum();

                    let coordinate = undoer.lan.end.try_move(0, direction).expect("The coordinates above and below an en passant target should always be valid.");

                    self.pieces[coordinate as usize] =
                        Some(Piece(piece.0.opponent(), PieceKind::Pawn));
                }
                MoveModifier::Promotion => {
                    self.pieces[undoer.lan.start as usize] = Some(Piece(piece.0, PieceKind::Pawn));
                }
            }
        }
    }

    fn walk_dangerously(&self, danger_zone: &mut Bitboard, start: Coordinate, dx: i8, dy: i8) {
        let size = BOARD_WIDTH.max(BOARD_HEIGHT) as i8;
        let opponent = self.pieces[start as usize]
            .expect("The starting Coordinate should always index a Some piece")
            .0
            .opponent();

        for i in 1..size {
            if let Ok(end) = start.try_move(i * dx, i * dy) {
                match self.pieces[end as usize] {
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

    fn generate_pawn_danger_zone(&self, coordinate: Coordinate) -> Option<Bitboard> {
        match self.pieces[coordinate as usize] {
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
        match self.pieces[coordinate as usize] {
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
        match self.pieces[coordinate as usize] {
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
        match self.pieces[coordinate as usize] {
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
        match self.pieces[coordinate as usize] {
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
        match self.pieces[coordinate as usize] {
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

                if let Some(piece) = self.pieces[coordinate as usize] {
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

                match self.pieces[coordinate as usize] {
                    Some(Piece(temp, PieceKind::King)) if temp == color => {
                        return Some(coordinate);
                    }
                    _ => (),
                }
            }
        }

        None
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
        let ranks = value.0.split('/');

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct State {
    board: Board,
    side_to_move: Color,
    castling_ability: Option<CastlingAbility>,
    en_passant_target: Option<Coordinate>,
    half_moves: usize,
    full_moves: usize,
}

struct StateUndoer {
    move_undoer: MoveUndoer,
    castling_ability: Option<CastlingAbility>,
    en_passant_target: Option<Coordinate>,
    half_moves: usize,
}

impl Default for State {
    fn default() -> Self {
        State {
            board: Default::default(),
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

impl State {
    fn make_move(&mut self, lan: Lan) -> Result<StateUndoer, ChessError> {
        let current_side = self.side_to_move;
        let opponent = self.side_to_move.opponent();

        // Make a copy of irreversible properties of State.
        let castling_ability = self.castling_ability;
        let en_passant_target = self.en_passant_target;
        let half_moves = self.half_moves;

        let piece = self.board[lan.start].ok_or(ChessError(
            ChessErrorKind::TargetIsNone,
            "Cannot move a piece that does not exist.",
        ))?;
        let target = self.board[lan.end];

        let capture = target.is_some();
        let dy = lan.end.y() as i8 - lan.start.y() as i8;

        // Toggle the current side.
        self.side_to_move = opponent;

        // If the king moves then remove their ability to castle.
        if let Piece(color, PieceKind::King) = piece {
            match color {
                Color::White => {
                    if let Some(ability) = self.castling_ability {
                        self.castling_ability = Some(
                            ability
                                & (!(CastlingAbility::WHITE_KINGSIDE
                                    | CastlingAbility::WHITE_QUEENSIDE)),
                        );
                    }
                }
                Color::Black => {
                    if let Some(ability) = self.castling_ability {
                        self.castling_ability = Some(
                            ability
                                & (!(CastlingAbility::BLACK_KINGSIDE
                                    | CastlingAbility::BLACK_QUEENSIDE)),
                        );
                    }
                }
            }
        }

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

            Coordinate::try_from(y * BOARD_WIDTH + x)
                .expect("The given index should always be a valid Coordinate.")
        };

        let king_side = match current_side {
            Color::White => CastlingAbility::WHITE_KINGSIDE,
            Color::Black => CastlingAbility::BLACK_KINGSIDE,
        };
        let queen_side = match current_side {
            Color::White => CastlingAbility::WHITE_QUEENSIDE,
            Color::Black => CastlingAbility::BLACK_QUEENSIDE,
        };

        let king_side_index = significant_rook_index(king_side);
        let queen_side_index = significant_rook_index(queen_side);

        // Make sure that moving a rook affects the king's ability to castle.
        if piece.1 == PieceKind::Rook {
            if lan.start == king_side_index {
                if let Some(ability) = self.castling_ability {
                    if !(ability & king_side).is_empty() {
                        self.castling_ability = Some(ability ^ king_side);
                    }
                }
            } else if lan.start == queen_side_index {
                if let Some(ability) = self.castling_ability {
                    if !(ability & queen_side).is_empty() {
                        self.castling_ability = Some(ability ^ queen_side);
                    }
                }
            }
        }

        let king_side = match opponent {
            Color::White => CastlingAbility::WHITE_KINGSIDE,
            Color::Black => CastlingAbility::BLACK_KINGSIDE,
        };
        let queen_side = match opponent {
            Color::White => CastlingAbility::WHITE_QUEENSIDE,
            Color::Black => CastlingAbility::BLACK_QUEENSIDE,
        };

        let king_side_index = significant_rook_index(king_side);
        let queen_side_index = significant_rook_index(queen_side);

        // Capturing a rook on either corner should disable castling on that side.
        if let Some(Piece(_, PieceKind::Rook)) = target {
            if lan.end == king_side_index {
                if let Some(ability) = self.castling_ability {
                    if (ability & king_side) != CastlingAbility::empty() {
                        self.castling_ability = Some(ability ^ king_side);
                    }
                }
            } else if lan.end == queen_side_index {
                if let Some(ability) = self.castling_ability {
                    if (ability & queen_side) != CastlingAbility::empty() {
                        self.castling_ability = Some(ability ^ queen_side);
                    }
                }
            }
        }

        self.en_passant_target = None;

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
                match self.board[coordinate] {
                    Some(Piece(color, PieceKind::Pawn)) if color == opponent => {
                        self.en_passant_target = Some(potential_en_passant_target);
                        pawns += 1;
                    }
                    _ => (),
                }
            }
            if let Ok(coordinate) = lan.end.try_move(1, 0) {
                match self.board[coordinate] {
                    Some(Piece(color, PieceKind::Pawn)) if color == opponent => {
                        self.en_passant_target = Some(potential_en_passant_target);
                        pawns += 1;
                    }
                    _ => (),
                }
            }

            // Taking en passant could lead to a discovered check; we need to make sure that cannot happen.
            if pawns == 1 {
                let mut kings_coordinate: Option<Coordinate> = None;
                let mut rank: [Option<Piece>; BOARD_WIDTH as usize] = [None; BOARD_WIDTH as usize];

                let y = match current_side {
                    Color::White => 4,
                    Color::Black => 3,
                };

                for x in 0..BOARD_WIDTH {
                    let index = y * BOARD_WIDTH + x;
                    let coordinate = Coordinate::try_from(index)
                        .expect("The given index should always be within the board's length.");
                    let target = self.board[coordinate];

                    match target {
                        Some(Piece(color, PieceKind::King)) if color == opponent => {
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
                            Some(Piece(color, PieceKind::Pawn)) if color == opponent => {
                                rank[index] = None;
                            }
                            _ => (),
                        }
                    }
                    if x > 0 {
                        let index = x as usize - 1;

                        match rank[index] {
                            Some(Piece(color, PieceKind::Pawn)) if color == opponent => {
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
                            Some(Piece(color, kind)) if color == current_side => {
                                if let PieceKind::Rook | PieceKind::Queen = kind {
                                    danger = true;
                                }

                                break;
                            }
                            Some(Piece(color, _)) if color == opponent => {
                                break;
                            }
                            _ => (),
                        }

                        kings_x += dir_x;
                    }

                    // Taking en passant would have resulted in a discovered check; en_passant_target should be disabled.
                    if danger {
                        self.en_passant_target = None;
                    }
                }
            }
        }

        self.half_moves += 1;

        if capture || piece.1 == PieceKind::Pawn {
            self.half_moves = 0;
        }

        if current_side == Color::Black {
            self.full_moves += 1;
        }

        // Move the piece.
        let move_undoer = self.board.make_move(lan)?;

        Ok(StateUndoer {
            move_undoer,
            castling_ability,
            en_passant_target,
            half_moves,
        })
    }

    fn unmake_move(&mut self, undoer: StateUndoer) {
        self.board.unmake_move(undoer.move_undoer);

        self.side_to_move = self.side_to_move.opponent();
        self.castling_ability = undoer.castling_ability;
        self.en_passant_target = undoer.en_passant_target;
        self.half_moves = undoer.half_moves;

        if self.side_to_move == Color::Black {
            self.full_moves -= 1;
        }
    }

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

    fn generate_pseudo_legal_pawn_moves(&self, start: Coordinate) -> Vec<Lan> {
        let mut moves = Vec::with_capacity(4);

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
                    if let Some(en_passant_target) = self.en_passant_target {
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
                    if let Some(en_passant_target) = self.en_passant_target {
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
        let mut moves = Vec::with_capacity(8);

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
        let mut moves = Vec::with_capacity(13);

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
        let mut moves = Vec::with_capacity(14);

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
        let mut moves = Vec::with_capacity(27);

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
        let mut moves = Vec::with_capacity(8);

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

            if let Some(castling_ability) = self.castling_ability {
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

    fn find_pins(&self, coordinate: Coordinate) -> Option<Bitboard> {
        let target = coordinate;
        let color = self.board[target]?.0;
        let opponent = color.opponent();

        let mut result = Bitboard::empty();

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
                            if coordinate.x() == target.x() || coordinate.y() == target.y() {
                                return None;
                            }

                            let difference_x = target.x() as i8 - coordinate.x() as i8;
                            let difference_y = target.y() as i8 - coordinate.y() as i8;

                            if difference_x.abs() != difference_y.abs() {
                                return None;
                            }

                            let x = -(coordinate.x() as i8 - target.x() as i8).signum();
                            let y = (coordinate.y() as i8 - target.y() as i8).signum();

                            Some((x, y))
                        }
                        PieceKind::Rook => {
                            if coordinate.x() != target.x() && coordinate.y() != target.y() {
                                return None;
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
                        PieceKind::Queen => {
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

                            if coordinate.x() != target.x() && coordinate.y() != target.y() {
                                let difference_x = target.x() as i8 - coordinate.x() as i8;
                                let difference_y = target.y() as i8 - coordinate.y() as i8;

                                if difference_x.abs() != difference_y.abs() {
                                    return None;
                                }

                                let x = -(coordinate.x() as i8 - target.x() as i8).signum();
                                let y = (coordinate.y() as i8 - target.y() as i8).signum();

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
                                Some(Piece(temp, _)) if temp == color => {
                                    if target == coordinate {
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
                                    if (dx > 0 && coordinate.x() > target.x())
                                        || (dx < 0 && coordinate.x() < target.x())
                                        || (dy > 0 && coordinate.y() < target.y())
                                        || (dy < 0 && coordinate.y() > target.y())
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

    fn analyze(&self, color: Color) -> Analysis {
        let kings_coordinate = self
            .board
            .find_king(color)
            .expect("A valid State must always have one white and black king.");

        let opponent = color.opponent();

        let mut moves = self.generate_pseudo_legal_moves(color);
        let danger_zone = self.board.generate_danger_zone(opponent);
        let pins = self
            .find_pins(kings_coordinate)
            .expect("The given coordinates should always index a Some Piece.");
        let attackers = self
            .find_attackers(kings_coordinate)
            .expect("The given coordinates should always index a Some Piece.");

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

                                            if let Some(castling_ability) = self.castling_ability {
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
                                    if let Some(en_passant_target) = self.en_passant_target {
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

        Analysis {
            moves,
            danger_zone,
            king_location: kings_coordinate,
            king_safety,
        }
    }
}

impl From<Fen> for State {
    fn from(value: Fen) -> Self {
        let board = Board::from(&value.placement);

        State {
            board,
            side_to_move: value.side_to_move,
            castling_ability: value.castling_ability,
            en_passant_target: value.en_passant_target,
            half_moves: value.half_moves,
            full_moves: value.full_moves,
        }
    }
}

impl From<State> for Fen {
    fn from(value: State) -> Self {
        let placement = Placement::from(value.board);

        Fen {
            placement,
            side_to_move: value.side_to_move,
            castling_ability: value.castling_ability,
            en_passant_target: value.en_passant_target,
            half_moves: value.half_moves,
            full_moves: value.full_moves,
        }
    }
}

impl From<State> for String {
    fn from(value: State) -> Self {
        String::from(&Fen::from(value))
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(*self))
    }
}

#[derive(Clone, Copy)]
enum Strategy {
    Maximizing,
    Minimizing,
}

impl Strategy {
    fn opposite(&self) -> Strategy {
        match &self {
            Strategy::Maximizing => Strategy::Minimizing,
            Strategy::Minimizing => Strategy::Maximizing,
        }
    }
}

impl From<Color> for Strategy {
    fn from(value: Color) -> Self {
        match value {
            Color::White => Strategy::Maximizing,
            Color::Black => Strategy::Minimizing,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Evaluation {
    Winner(Color),
    Draw,
    Static(i16),
}

impl Evaluation {
    fn min(&self, value: Evaluation) -> Evaluation {
        let left = i16::from(*self);
        let right = i16::from(value);

        match left.cmp(&right) {
            Ordering::Less | Ordering::Equal => *self,
            Ordering::Greater => value,
        }
    }

    fn max(&self, value: Evaluation) -> Evaluation {
        let left = i16::from(*self);
        let right = i16::from(value);

        match left.cmp(&right) {
            Ordering::Greater | Ordering::Equal => *self,
            Ordering::Less => value,
        }
    }
}

impl From<Evaluation> for i16 {
    fn from(value: Evaluation) -> Self {
        match value {
            Evaluation::Winner(side) => match side {
                Color::White => CHECKMATE_EVALUATION,
                Color::Black => -CHECKMATE_EVALUATION,
            },
            Evaluation::Draw => 0,
            Evaluation::Static(value) => value,
        }
    }
}

struct MinimaxParams<'a> {
    state: &'a mut State,
    depth: u8,
    searched: &'a mut u128,
    line: &'a Option<Vec<Lan>>,
    alpha: i16,
    beta: i16,
    strategy: Strategy,
}

struct SearchNode {
    evaluation: Evaluation,
    /// The move that resulted in this state.
    transformation: Option<Lan>,
    child: Option<Box<SearchNode>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Score {
    Cp(i16),
    Mate(i8),
    // Lowerbound,
    // Upperbound,
}

impl From<Score> for String {
    fn from(value: Score) -> Self {
        match value {
            Score::Cp(cp) => {
                format!("cp {}", cp)
            }
            Score::Mate(mate) => {
                format!("mate {}", mate)
            }
        }
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(*self))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct InfoStatistics {
    depth: Option<u8>,
    // seldepth: Option<u8>,
    time: Option<u64>,
    nodes: Option<u128>,
    pv: Option<Vec<Lan>>,
    // multipv: Option<u8>,
    score: Option<Score>,
    currmove: Option<Lan>,
    currmovenumber: Option<u64>,
    // hashfull: Option<()>,
    nps: Option<u64>,
    // tbhits: Option<u64>,
    // sbhits: Option<u64>,
    // cpuload: Option<()>,
    // refutation: Option<Vec<Lan>>
    // currline: Option<()>
}

impl From<&InfoStatistics> for String {
    fn from(value: &InfoStatistics) -> Self {
        let mut result = String::from("info");

        if let Some(depth) = value.depth {
            result.push_str(" depth ");
            result.push_str(depth.to_string().as_str());
        }

        if let Some(score) = value.score {
            result.push(' ');
            result.push_str(score.to_string().as_str());
        }

        if let Some(time) = value.time {
            result.push_str(" time ");
            result.push_str(time.to_string().as_str());
        }

        if let Some(nodes) = value.nodes {
            result.push_str(" nodes ");
            result.push_str(nodes.to_string().as_str());
        }

        if let Some(currmove) = value.currmove {
            result.push_str(" currmove ");
            result.push_str(currmove.to_string().as_str());
        }

        if let Some(currmovenumber) = value.currmovenumber {
            result.push_str(" currmovenumber ");
            result.push_str(currmovenumber.to_string().as_str());
        }

        if let Some(nps) = value.nps {
            result.push_str(" nps ");
            result.push_str(nps.to_string().as_str());
        }

        if let Some(pv) = &value.pv {
            result.push_str(" pv");

            for lan in pv {
                result.push(' ');
                result.push_str(lan.to_string().as_str());
            }
        }

        result
    }
}

#[derive(Debug, Clone, Copy)]
struct Suggestion {
    lan: Lan,
    ponder: Option<Lan>,
}

impl From<Suggestion> for String {
    fn from(value: Suggestion) -> Self {
        let mut result = format!("bestmove {}", value.lan);

        if let Some(ponder) = value.ponder {
            result.push_str(" ponder ");
            result.push_str(ponder.to_string().as_str());
        }

        result
    }
}

impl Display for Suggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(*self))
    }
}

// TODO(thismarvin): This definitely needs a better name... right?
#[derive(Debug, Clone, Copy)]
enum GoParams {
    Depth(u8),
    Perft(u8),
}

#[derive(Debug, Clone, Copy)]
enum Command {
    Uci,
    Isready,
    Position(State),
    Go(GoParams),
    Quit,
    // The following are non-standard commands.
    D,
    Flip,
}

impl TryFrom<&str> for Command {
    type Error = ChessError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value == "uci" {
            return Ok(Command::Uci);
        }

        if value == "isready" {
            return Ok(Command::Isready);
        }

        if value.starts_with("position") {
            let mut sections = value.split_whitespace().skip(1);

            let next = sections.next().ok_or(ChessError(
                ChessErrorKind::InvalidString,
                "Expected <startpos | fen> subcommand.",
            ))?;

            return match next {
                "startpos" => {
                    let mut state = State::default();

                    if let Some(subcommand) = sections.next() {
                        match subcommand {
                            "moves" => {
                                let mut sequence = Vec::new();

                                for lan in sections {
                                    let lan = Lan::try_from(lan).map_err(|_| {
                                        ChessError(
                                            ChessErrorKind::InvalidString,
                                            "A string in the given move sequence is not a valid Lan string.",
                                        )
                                    })?;

                                    sequence.push(lan);
                                }

                                Engine::make_sequence(&mut state, &sequence)?;
                            }
                            _ => {
                                return Err(ChessError(
                                    ChessErrorKind::InvalidString,
                                    "The given subcommand is not valid; expected [moves <move>...]",
                                ));
                            }
                        }
                    }

                    Ok(Command::Position(state))
                }
                "fen" => {
                    let placement = sections.next().ok_or(ChessError(
                        ChessErrorKind::InvalidString,
                        "Expected a valid Fen string to follow \"position fen\".",
                    ))?;

                    let side_to_move = sections.next().ok_or(ChessError(
                        ChessErrorKind::InvalidString,
                        "Expected a valid Fen string to follow \"position fen\".",
                    ))?;

                    let castling_ability = sections.next().ok_or(ChessError(
                        ChessErrorKind::InvalidString,
                        "Expected a valid Fen string to follow \"position fen\".",
                    ))?;

                    let en_passant_target = sections.next().ok_or(ChessError(
                        ChessErrorKind::InvalidString,
                        "Expected a valid Fen string to follow \"position fen\".",
                    ))?;

                    let half_moves = sections.next().ok_or(ChessError(
                        ChessErrorKind::InvalidString,
                        "Expected a valid Fen string to follow \"position fen\".",
                    ))?;

                    let full_moves = sections.next().ok_or(ChessError(
                        ChessErrorKind::InvalidString,
                        "Expected a valid Fen string to follow \"position fen\".",
                    ))?;

                    let fen = format!(
                        "{} {} {} {} {} {}",
                        placement,
                        side_to_move,
                        castling_ability,
                        en_passant_target,
                        half_moves,
                        full_moves
                    );

                    let fen = Fen::try_from(fen.as_str()).map_err(|_| {
                        ChessError(
                            ChessErrorKind::InvalidString,
                            "The given string is not a valid Fen string.",
                        )
                    })?;

                    let mut state = State::from(fen);

                    if let Some(subcommand) = sections.next() {
                        match subcommand {
                            "moves" => {
                                let mut sequence = Vec::new();

                                for lan in sections {
                                    let lan = Lan::try_from(lan).map_err(|_| {
                                        ChessError(
                                            ChessErrorKind::InvalidString,
                                            "A string in the given move sequence is not a valid Lan string.",
                                        )
                                    })?;

                                    sequence.push(lan);
                                }

                                Engine::make_sequence(&mut state, &sequence)?;
                            }
                            _ => {
                                return Err(ChessError(
                                    ChessErrorKind::InvalidString,
                                    "The given subcommand is not valid; expected [moves <move>...]",
                                ));
                            }
                        }
                    }

                    Ok(Command::Position(state))
                }
                _ => Err(ChessError(
                    ChessErrorKind::InvalidString,
                    "The given subcommand is not valid; expected <startpos | fen>",
                )),
            };
        }

        if value.starts_with("go") {
            let mut sections = value.split_whitespace().skip(1);

            let next = sections.next().ok_or(ChessError(
                ChessErrorKind::InvalidString,
                "Expected <depth | perft> subcommand.",
            ))?;

            return match next {
                "depth" => {
                    let depth = sections.next().ok_or(ChessError(
                        ChessErrorKind::InvalidString,
                        "Expected a valid u8 string to follow \"go depth\".",
                    ))?;

                    let depth = depth.parse::<u8>().map_err(|_| {
                        ChessError(
                            ChessErrorKind::InvalidString,
                            "The given string is not a valid u8 string.",
                        )
                    })?;

                    Ok(Command::Go(GoParams::Depth(depth)))
                }
                "perft" => {
                    let depth = sections.next().ok_or(ChessError(
                        ChessErrorKind::InvalidString,
                        "Expected a valid u8 string to follow \"go perft\".",
                    ))?;

                    let depth = depth.parse::<u8>().map_err(|_| {
                        ChessError(
                            ChessErrorKind::InvalidString,
                            "The given string is not a valid u8 string.",
                        )
                    })?;

                    Ok(Command::Go(GoParams::Perft(depth)))
                }
                _ => Err(ChessError(
                    ChessErrorKind::InvalidString,
                    "The given subcommand is not valid; expected <depth | perft>",
                )),
            };
        }

        if value == "quit" {
            return Ok(Command::Quit);
        }

        if value == "d" {
            return Ok(Command::D);
        }

        if value == "flip" {
            return Ok(Command::Flip);
        }

        Err(ChessError(
            ChessErrorKind::InvalidString,
            "Unknown command.",
        ))
    }
}

pub struct Engine;

impl Engine {
    fn make_sequence(state: &mut State, sequence: &[Lan]) -> Result<(), ChessError> {
        for lan in sequence {
            let analysis = state.analyze(state.side_to_move);

            if let Some(list) = &analysis.moves[lan.start as usize] {
                if list.contains(lan) {
                    state
                        .make_move(*lan)
                        .expect("The given move should always be valid.");

                    continue;
                }
            }

            return Err(ChessError(
                ChessErrorKind::Other,
                "A move in the given sequence is not legal.",
            ));
        }

        Ok(())
    }

    pub fn perft(state: &mut State, depth: u8) -> u128 {
        if depth == 0 {
            return 1;
        }

        let analysis = state.analyze(state.side_to_move);

        // At a depth of one, the total amount of legal moves is the perft value.
        if depth == 1 {
            return analysis
                .moves
                .iter()
                .filter_map(|entry| entry.as_ref())
                .fold(0, |accumulator, entry| accumulator + entry.len() as u128);
        }

        let mut total = 0;

        for move_list in analysis.moves.into_iter().flatten() {
            for lan in move_list {
                let undoer = state
                    .make_move(lan)
                    .expect("The given move should always be valid");

                total += Engine::perft(state, depth - 1);

                state.unmake_move(undoer);
            }
        }

        total
    }

    fn evaluate(state: State) -> Evaluation {
        let white_analysis = state.analyze(Color::White);
        let black_analysis = state.analyze(Color::Black);

        if white_analysis.king_safety == KingSafety::Checkmate {
            return Evaluation::Winner(Color::Black);
        }

        if black_analysis.king_safety == KingSafety::Checkmate {
            return Evaluation::Winner(Color::White);
        }

        // Draws
        // TODO(thismarvin): How will this function handle other types of draws?

        // Draw by stalemate.
        if white_analysis.king_safety == KingSafety::Stalemate
            || black_analysis.king_safety == KingSafety::Stalemate
        {
            return Evaluation::Draw;
        }

        // Draw by the seventy-five-move rule.
        if state.half_moves >= 75 {
            return Evaluation::Draw;
        }

        let mut white_score: f32 = 0.0;
        let mut black_score: f32 = 0.0;

        // Accumulate the material value for each piece.
        for piece in state.board.pieces.iter().flatten() {
            match piece.0 {
                Color::White => white_score += piece.1.value() as f32,
                Color::Black => black_score += piece.1.value() as f32,
            }
        }

        // The following rewards/penalties are opinionated and their values are completely arbitrary.

        // Reward each side for checking the opponent's king.
        if white_analysis.king_safety == KingSafety::Check {
            white_score += 75.0;
        }

        if black_analysis.king_safety == KingSafety::Check {
            black_score += 75.0;
        }

        // Reward each side for the total amount for moves they can make.
        let white_total_moves = white_analysis
            .moves
            .iter()
            .filter_map(|entry| entry.as_ref())
            .fold(0, |accumulator, entry| accumulator + entry.len())
            as isize;

        let black_total_moves = black_analysis
            .moves
            .iter()
            .filter_map(|entry| entry.as_ref())
            .fold(0, |accumulator, entry| accumulator + entry.len())
            as isize;

        white_score += (white_total_moves * 2 - black_total_moves) as f32;
        black_score += (black_total_moves * 2 - white_total_moves) as f32;

        // Reward each side for the total amount of squares they control.
        let white_total_control = black_analysis.danger_zone.population_count() as isize;
        let black_total_control = white_analysis.danger_zone.population_count() as isize;

        white_score += (white_total_control * 2 - black_total_control) as f32;
        black_score += (black_total_control * 2 - white_total_control) as f32;

        // Penalize each side for the amount of pieces still in their initial ranks.
        // TODO(thismarvin): This should really be a mask that applies to each piece.
        let mut white_total_initial_ranks: isize = 0;
        let mut black_total_initial_ranks: isize = 0;

        let white_starting_coordinate_a = Coordinate::A1;
        let white_starting_coordinate_b = Coordinate::A2;
        let black_starting_coordinate_a = Coordinate::A8;
        let black_starting_coordinate_b = Coordinate::A7;

        for dx in 0..BOARD_WIDTH {
            if let Some(Piece(Color::White, _)) = state.board[white_starting_coordinate_a
                .try_move(dx as i8, 0)
                .expect("This should always be within the board.")]
            {
                white_total_initial_ranks += 1;
            }
            if let Some(Piece(Color::White, _)) = state.board[white_starting_coordinate_b
                .try_move(dx as i8, 0)
                .expect("This should always be within the board.")]
            {
                white_total_initial_ranks += 1;
            }

            if let Some(Piece(Color::Black, _)) = state.board[black_starting_coordinate_a
                .try_move(dx as i8, 0)
                .expect("This should always be within the board.")]
            {
                black_total_initial_ranks += 1;
            }
            if let Some(Piece(Color::Black, _)) = state.board[black_starting_coordinate_b
                .try_move(dx as i8, 0)
                .expect("This should always be within the board.")]
            {
                black_total_initial_ranks += 1;
            }
        }

        white_score -= (white_total_initial_ranks * 7) as f32;
        black_score -= (black_total_initial_ranks * 7) as f32;

        // Penalize each side if the amount of half moves is approaching fifty moves.
        // TODO(thismarvin): Test out whether or not these weights make any sense.
        let half_moves_penalty_multiplier = match state.half_moves {
            0..=10 => 0,
            11..=25 => 1,
            26..=40 => 4,
            41..=45 => 8,
            _ => 16,
        };

        let half_moves_penalty = state.half_moves * half_moves_penalty_multiplier;

        white_score -= half_moves_penalty as f32;
        black_score -= half_moves_penalty as f32;

        // The final evaluation is simply the difference in white and black score.
        Evaluation::Static((white_score - black_score).round() as i16)
    }

    // TODO(thismarvin): Is it possible to combine this with `minimax`?
    fn quiescence_minimax(params: &mut MinimaxParams, analysis: Analysis) -> Evaluation {
        let opponent = params.state.side_to_move.opponent();

        let mut needs_sorting = false;
        let mut moves = analysis
            .moves
            .iter()
            .flatten()
            .flatten()
            .map(|lan| {
                let score: u16 = match params.state.board[lan.end] {
                    // Score captures higher.
                    Some(Piece(color, kind)) if color == opponent => {
                        needs_sorting = true;

                        let start = params.state.board[lan.start]
                            .expect("This should always be a Some Piece.");

                        match kind {
                            // Evaluate capturing with a king last.
                            PieceKind::King => 1,
                            // Prefer capturing with pieces with the least value.
                            _ => (900 + kind.value() - start.1.value()) as u16,
                        }
                    }
                    _ => 0,
                };

                (score, lan)
            })
            .collect::<Vec<(u16, &Lan)>>();

        if needs_sorting {
            moves.sort_by(|a, b| b.0.cmp(&a.0));
        }

        let moves = moves;

        let mut alpha = params.alpha;
        let mut beta = params.beta;
        let mut evaluation = match params.state.side_to_move {
            Color::White => Evaluation::Static(i16::MIN),
            Color::Black => Evaluation::Static(i16::MAX),
        };

        for (_, &lan) in moves {
            (*params.searched) += 1;

            let undoer = params
                .state
                .make_move(lan)
                .expect("The given move should always be valid.");

            let mut next = MinimaxParams {
                state: params.state,
                depth: params.depth,
                searched: params.searched,
                line: params.line,
                alpha,
                beta,
                strategy: params.strategy.opposite(),
            };

            let eval = Engine::quiescence(&mut next);
            let score = i16::from(eval);

            params.state.unmake_move(undoer);

            match params.strategy {
                Strategy::Maximizing => {
                    evaluation = evaluation.max(eval);
                    alpha = alpha.max(score);
                }
                Strategy::Minimizing => {
                    evaluation = evaluation.min(eval);
                    beta = beta.min(score);
                }
            }

            if beta <= alpha {
                break;
            }
        }

        evaluation
    }

    fn quiescence(params: &mut MinimaxParams) -> Evaluation {
        let analysis = params.state.analyze(params.state.side_to_move);

        match analysis.king_safety {
            KingSafety::Checkmate => {
                return Evaluation::Winner(params.state.side_to_move.opponent())
            }
            KingSafety::Stalemate => {
                return Evaluation::Draw;
            }
            KingSafety::Check => {
                return Engine::quiescence_minimax(params, analysis);
            }
            _ => (),
        }

        let mut alpha = params.alpha;
        let mut beta = params.beta;
        let mut evaluation = match params.state.side_to_move {
            Color::White => Evaluation::Static(i16::MIN),
            Color::Black => Evaluation::Static(i16::MAX),
        };

        let standing_pat = Engine::evaluate(*params.state);
        let score = i16::from(standing_pat);

        match params.strategy {
            Strategy::Maximizing => {
                evaluation = evaluation.max(standing_pat);
                alpha = alpha.max(score);
            }
            Strategy::Minimizing => {
                evaluation = evaluation.min(standing_pat);
                beta = beta.min(score);
            }
        }

        if beta <= alpha {
            return evaluation;
        }

        let mut moves = analysis
            .moves
            .iter()
            .flatten()
            .flatten()
            .filter(|lan| params.state.board[lan.end].is_some())
            .map(|lan| {
                let score: u16 = match params.state.board[lan.end] {
                    // Score captures higher.
                    Some(Piece(_, kind)) => {
                        let start = params.state.board[lan.start]
                            .expect("This should always be a Some Piece.");

                        match kind {
                            // Evaluate capturing with a king last.
                            PieceKind::King => 1,
                            // Prefer capturing with pieces with the least value.
                            _ => (900 + kind.value() - start.1.value()) as u16,
                        }
                    }
                    _ => unreachable!(),
                };

                (score, lan)
            })
            .collect::<Vec<(u16, &Lan)>>();

        if moves.is_empty() {
            return evaluation;
        }

        moves.sort_by(|a, b| b.0.cmp(&a.0));

        let moves = moves;

        for (_, &lan) in moves {
            (*params.searched) += 1;

            let undoer = params
                .state
                .make_move(lan)
                .expect("The given move should always be valid.");

            let mut next = MinimaxParams {
                state: params.state,
                depth: params.depth,
                searched: params.searched,
                line: params.line,
                alpha,
                beta,
                strategy: params.strategy.opposite(),
            };

            let eval = Engine::quiescence(&mut next);
            let score = i16::from(eval);

            params.state.unmake_move(undoer);

            match params.strategy {
                Strategy::Maximizing => {
                    evaluation = evaluation.max(eval);
                    alpha = alpha.max(score);
                }
                Strategy::Minimizing => {
                    evaluation = evaluation.min(eval);
                    beta = beta.min(score);
                }
            }

            if beta <= alpha {
                break;
            }
        }

        evaluation
    }

    fn minimax(params: &mut MinimaxParams) -> SearchNode {
        if params.depth == 0 {
            let evaluation = Engine::quiescence(params);

            return SearchNode {
                evaluation,
                transformation: None,
                child: None,
            };
        }

        let opponent = params.state.side_to_move.opponent();
        let analysis = params.state.analyze(params.state.side_to_move);

        match analysis.king_safety {
            KingSafety::Checkmate => {
                let evaluation = Evaluation::Winner(params.state.side_to_move.opponent());

                return SearchNode {
                    evaluation,
                    transformation: None,
                    child: None,
                };
            }
            KingSafety::Stalemate => {
                return SearchNode {
                    evaluation: Evaluation::Draw,
                    transformation: None,
                    child: None,
                };
            }
            _ => (),
        }

        // TODO(thismarvin): There has to be a better way to incorporate the previous search...
        let target = if let Some(line) = params.line {
            line.get(line.len() + 1 - params.depth as usize)
        } else {
            None
        };
        let mut pivot = None;

        // `minimax` should be faster when the best moves are searched first.
        let mut needs_sorting = false;
        let mut moves = analysis
            .moves
            .iter()
            .flatten()
            .flatten()
            .enumerate()
            .map(|(i, lan)| {
                if let Some(target) = target {
                    if *lan == *target {
                        pivot = Some(i);

                        return (u16::MAX, lan);
                    }
                }

                let score: u16 = match params.state.board[lan.end] {
                    // Score captures higher.
                    Some(Piece(color, kind)) if color == opponent => {
                        needs_sorting = true;

                        let start = params.state.board[lan.start]
                            .expect("This should always be a Some Piece.");

                        match kind {
                            // Evaluate capturing with a king last.
                            PieceKind::King => 1,
                            // Prefer capturing with pieces with the least value.
                            _ => (900 + kind.value() - start.1.value()) as u16,
                        }
                    }
                    _ => 0,
                };

                (score, lan)
            })
            .collect::<Vec<(u16, &Lan)>>();

        // Evaluate the previous best move at this depth first.
        if let Some(pivot) = pivot {
            moves.swap(0, pivot);
        }

        if needs_sorting {
            moves.sort_by(|a, b| b.0.cmp(&a.0));
        }

        let moves = moves;

        let mut alpha = params.alpha;
        let mut beta = params.beta;
        let mut evaluation = match params.state.side_to_move {
            Color::White => Evaluation::Static(i16::MIN),
            Color::Black => Evaluation::Static(i16::MAX),
        };
        let mut best_lan: Option<Lan> = None;
        let mut best_child: Option<SearchNode> = None;

        for (_, &lan) in moves {
            (*params.searched) += 1;

            let undoer = params
                .state
                .make_move(lan)
                .expect("The given move should always be valid.");

            let mut next = MinimaxParams {
                state: params.state,
                depth: params.depth - 1,
                searched: params.searched,
                line: params.line,
                alpha,
                beta,
                strategy: params.strategy.opposite(),
            };

            let node = Engine::minimax(&mut next);

            params.state.unmake_move(undoer);

            let score = i16::from(node.evaluation);

            match params.strategy {
                Strategy::Maximizing => {
                    evaluation = evaluation.max(node.evaluation);

                    if score > alpha {
                        alpha = score;
                        best_lan = Some(lan);
                        best_child = Some(node);
                    }
                }
                Strategy::Minimizing => {
                    evaluation = evaluation.min(node.evaluation);

                    if score < beta {
                        beta = score;
                        best_lan = Some(lan);
                        best_child = Some(node);
                    }
                }
            }

            if beta <= alpha {
                break;
            }
        }

        let transformation = best_lan;
        let child = best_child.map(Box::new);

        SearchNode {
            evaluation,
            transformation,
            child,
        }
    }

    fn analyze(state: &mut State, depth: u8, line: Option<Vec<Lan>>) -> InfoStatistics {
        if depth == 0 {
            panic!("Depth should never be zero.");
        }

        let mut searched = 0;
        let strategy = Strategy::from(state.side_to_move);

        let mut params = MinimaxParams {
            state,
            depth,
            searched: &mut searched,
            line: &line,
            alpha: i16::MIN,
            beta: i16::MAX,
            strategy,
        };

        let result = Engine::minimax(&mut params);

        let evaluation = result.evaluation;
        let lan = result
            .transformation
            .expect("There should always be a move suggestion.");
        let mut line: Vec<Lan> = Vec::with_capacity(depth as usize);

        line.push(lan);

        let mut head = result.child;

        while let Some(contents) = head {
            if let Some(lan) = contents.transformation {
                line.push(lan);
            }

            head = contents.child;
        }

        let score = match evaluation {
            Evaluation::Winner(side) => {
                // "If the engine is getting mated use negative values for y."
                let sign = if state.side_to_move != side { -1 } else { 1 };

                // Convert plies to moves.
                let moves = (line.len() as f32 / 2.0).ceil() as i8 * sign;

                Score::Mate(moves)
            }
            _ => Score::Cp(i16::from(evaluation)),
        };

        InfoStatistics {
            depth: Some(depth),
            nodes: Some(searched),
            pv: Some(line),
            score: Some(score),
            ..Default::default()
        }
    }
}

pub struct Pescado {
    state: State,
    cb: Box<dyn Fn(String)>,
}

impl Pescado {
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(String) + 'static,
    {
        utils::set_panic_hook();

        Pescado {
            state: State::default(),
            cb: Box::new(callback),
        }
    }

    fn go_depth(&mut self, depth: u8) {
        if depth == 0 {
            // TODO(thismarvin): Should zero just make the engine search forever?
            return;
        }

        let mut line = None;

        // Iterative Deepening.
        for i in 1..=depth {
            let info = Engine::analyze(&mut self.state, i, line);

            (self.cb)(String::from(&info));

            line = info.pv;
        }

        let line = line.expect("Analysis should always return the best line.");

        let suggestion = Suggestion {
            lan: line[0],
            ponder: line.get(1).copied(),
        };

        (self.cb)(format!("{}", suggestion));
    }

    fn go_perft(&mut self, depth: u8) {
        if depth == 0 {
            return;
        }

        let mut string = String::new();

        let analysis = self.state.analyze(self.state.side_to_move);
        let moves = analysis.moves.iter().flatten().flatten();

        let mut total = 0;

        for &lan in moves {
            let undoer = self
                .state
                .make_move(lan)
                .expect("The given move should always be valid.");

            let perft = Engine::perft(&mut self.state, depth - 1);

            total += perft;

            self.state.unmake_move(undoer);

            string.push_str(&format!("{}: {}\n", lan, perft));
        }

        string.push('\n');
        string.push_str(&format!("Nodes searched: {}", total));

        (self.cb)(string);
    }

    fn d(&self) {
        // TODO(thismarvin): Checkers field? (e.g. Checkers: e4)
        // TODO(thismarvin): Key field? (e.g. Key: 8F8F01D4562F59FB)

        let mut string = String::new();

        string.push_str("┌───┬───┬───┬───┬───┬───┬───┬───┐\n");

        for y in 0..BOARD_HEIGHT {
            let mut row = String::new();

            row.push('│');

            for x in 0..BOARD_WIDTH {
                row.push_str(
                    format!(
                        " {} │",
                        self.state.board.pieces[(y * BOARD_WIDTH + x) as usize]
                            .map(<char>::from)
                            .unwrap_or(' ')
                    )
                    .as_str(),
                );
            }

            row.push_str(format!(" {}\n", BOARD_HEIGHT - y).as_str());

            string.push_str(&row);

            if y != BOARD_HEIGHT - 1 {
                string.push_str("├───┼───┼───┼───┼───┼───┼───┼───┤\n");
            } else {
                string.push_str("└───┴───┴───┴───┴───┴───┴───┴───┘\n");
            }
        }

        let mut row = String::from(" ");

        for x in 0..BOARD_WIDTH {
            row.push_str(format!(" {}  ", (b'a' + x as u8) as char).as_str());
        }

        string.push_str(&row);
        string.push_str("\n\n");
        string.push_str(&format!("Fen: {}", Fen::from(self.state)));

        (self.cb)(string);
    }

    fn flip(&mut self) {
        self.state.side_to_move = self.state.side_to_move.opponent();
    }

    pub fn send(&mut self, command: &str) {
        let command = Command::try_from(command);

        match command {
            Ok(command) => match command {
                Command::Uci => {
                    (self.cb)("id name Pescado".to_string());
                    (self.cb)("id author the Pescado developers".to_string());
                    (self.cb)("uciok".to_string());
                }
                Command::Isready => {
                    (self.cb)("readyok".to_string());
                }
                Command::Position(state) => {
                    self.state = state;
                }
                Command::Go(params) => match params {
                    GoParams::Depth(depth) => {
                        self.go_depth(depth);
                    }
                    GoParams::Perft(depth) => {
                        self.go_perft(depth);
                    }
                },
                Command::Quit => {}
                Command::D => {
                    self.d();
                }
                Command::Flip => {
                    self.flip();
                }
            },
            Err(error) => {
                let message = String::from(error.1);

                (self.cb)(format!("Error: {}", message));
            }
        }
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
                start: Coordinate::A1,
                end: Coordinate::A2,
                promotion: None,
            })
        );

        let lan = Lan::try_from("e7e8q");
        assert_eq!(
            lan,
            Ok(Lan {
                start: Coordinate::E7,
                end: Coordinate::E8,
                promotion: Some(PieceKind::Queen),
            })
        );

        Ok(())
    }

    #[test]
    fn test_placement_from_str() {
        let placement = Placement::try_from("what is this really called?");
        assert!(placement.is_err());

        let placement = Placement::try_from("pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
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

        let fen = Fen::try_from("pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
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

        // Too few kings.
        let fen = Fen::try_from("rnbq1bnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 1");
        assert!(fen.is_err());

        // Too many kings.
        let fen = Fen::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPKPPP/RNBQKBNR w kq - 0 1");
        assert!(fen.is_err());

        // En passant target in wrong rank.
        let fen = Fen::try_from("rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq e4 0 1");
        assert!(fen.is_err());

        // The en passant target is in a valid rank, but an enemy pawn is not in position to capture it.
        let fen = Fen::try_from("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
        assert!(fen.is_err());

        // The king is not in position to castle.
        let fen = Fen::try_from("rnbq1bnr/ppppkppp/8/4p3/4P3/8/PPPPKPPP/RNBQ1BNR w KQ - 0 1");
        assert!(fen.is_err());

        // A rook is not in position to castle queenside.
        let fen = Fen::try_from("4k3/8/8/8/8/8/8/4K2R w KQ - 0 1");
        assert!(fen.is_err());

        // The opponent's king is under attack.
        let fen = Fen::try_from("rnbqkbnr/pppp1ppp/8/4Q3/4P3/8/PPPP1PPP/RNB1KBNR w KQkq - 0 1");
        assert!(fen.is_err());

        let fen = Fen::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(fen, Ok(Fen::default()));

        // An example of a valid en passant target.
        let fen = Fen::try_from("rnbqkbnr/ppp1pppp/8/8/3pP3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 3");
        assert_eq!(
            fen,
            Ok(Fen {
                placement: Placement("rnbqkbnr/ppp1pppp/8/8/3pP3/8/PPPP1PPP/RNBQKBNR".into()),
                side_to_move: Color::Black,
                castling_ability: Some(
                    CastlingAbility::WHITE_KINGSIDE
                        | CastlingAbility::WHITE_QUEENSIDE
                        | CastlingAbility::BLACK_KINGSIDE
                        | CastlingAbility::BLACK_QUEENSIDE
                ),
                en_passant_target: Some(Coordinate::E3),
                half_moves: 0,
                full_moves: 3,
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
    fn test_board_make_move() -> Result<(), ChessError> {
        // Test moving nothing.
        let mut board = Board::default();
        let lan = Lan::try_from("e3e4")?;
        let result = board.make_move(lan);
        assert!(result.is_err());

        // Test promoting something other than a pawn.
        let mut board = Board::from(Placement("1k6/6R1/1K6/8/8/8/8/8".into()));
        let lan = Lan::try_from("g7g8q")?;
        let result = board.make_move(lan);
        assert!(result.is_err());

        // Test moving a piece.
        let mut board = Board::default();
        let lan = Lan::try_from("e2e4")?;

        board.make_move(lan)?;

        assert_eq!(board[Coordinate::E2], None);
        assert_eq!(
            board[Coordinate::E4],
            Some(Piece(Color::White, PieceKind::Pawn))
        );

        // Test promoting a pawn to a queen.
        let mut board = Board::from(Placement("8/2k1PK2/8/8/8/8/8/8".into()));
        let lan = Lan::try_from("e7e8q")?;

        board.make_move(lan)?;

        assert_eq!(board[Coordinate::E7], None);
        assert_eq!(
            board[Coordinate::E8],
            Some(Piece(Color::White, PieceKind::Queen))
        );

        // Test capturing en passant.
        let mut board = Board::from(Placement("4k3/8/8/8/4Pp2/8/8/4K3".into()));
        let lan = Lan::try_from("f4e3")?;

        board.make_move(lan)?;

        assert_eq!(board[Coordinate::F4], None);
        assert_eq!(
            board[Coordinate::E3],
            Some(Piece(Color::Black, PieceKind::Pawn))
        );
        assert_eq!(board[Coordinate::E4], None);

        // Test castling king side.
        let mut board = Board::from(Placement("4k3/8/8/8/8/8/8/4K2R".into()));
        let lan = Lan::try_from("e1g1")?;

        board.make_move(lan)?;

        assert_eq!(board[Coordinate::E1], None);
        assert_eq!(
            board[Coordinate::G1],
            Some(Piece(Color::White, PieceKind::King))
        );
        assert_eq!(board[Coordinate::H1], None);
        assert_eq!(
            board[Coordinate::F1],
            Some(Piece(Color::White, PieceKind::Rook))
        );

        // Test castling king side.
        let mut board = Board::from(Placement("r3k3/8/8/8/8/8/8/4K3".into()));
        let lan = Lan::try_from("e8c8")?;

        board.make_move(lan)?;

        assert_eq!(board[Coordinate::E8], None);
        assert_eq!(
            board[Coordinate::C8],
            Some(Piece(Color::Black, PieceKind::King))
        );
        assert_eq!(board[Coordinate::A8], None);
        assert_eq!(
            board[Coordinate::D8],
            Some(Piece(Color::Black, PieceKind::Rook))
        );

        Ok(())
    }

    #[test]
    fn test_board_unmake_move() -> Result<(), ChessError> {
        // Test moving a piece.
        let mut board = Board::default();
        let lan = Lan::try_from("e2e4")?;

        let initial = board.clone();
        let undoer = board.make_move(lan)?;

        assert_eq!(
            undoer,
            MoveUndoer {
                lan,
                previous: None,
                modifer: None
            }
        );

        board.unmake_move(undoer);

        assert_eq!(board, initial);

        // Test promoting a pawn to a queen.
        let mut board = Board::from(Placement("8/2k1PK2/8/8/8/8/8/8".into()));
        let lan = Lan::try_from("e7e8q")?;

        let initial = board.clone();
        let undoer = board.make_move(lan)?;

        assert_eq!(
            undoer,
            MoveUndoer {
                lan,
                previous: None,
                modifer: Some(MoveModifier::Promotion)
            }
        );

        board.unmake_move(undoer);

        assert_eq!(board, initial);

        // Test capturing en passant.
        let mut board = Board::from(Placement("4k3/8/8/8/4Pp2/8/8/4K3".into()));
        let lan = Lan::try_from("f4e3")?;

        let initial = board.clone();
        let undoer = board.make_move(lan)?;

        assert_eq!(
            undoer,
            MoveUndoer {
                lan,
                previous: None,
                modifer: Some(MoveModifier::EnPassant)
            }
        );

        board.unmake_move(undoer);

        assert_eq!(board, initial);

        // Test castling king side.
        let mut board = Board::from(Placement("4k3/8/8/8/8/8/8/4K2R".into()));
        let lan = Lan::try_from("e1g1")?;

        let initial = board.clone();
        let undoer = board.make_move(lan)?;

        assert_eq!(
            undoer,
            MoveUndoer {
                lan,
                previous: None,
                modifer: Some(MoveModifier::Castle)
            }
        );

        board.unmake_move(undoer);

        assert_eq!(board, initial);

        // Test castling king side.
        let mut board = Board::from(Placement("r3k3/8/8/8/8/8/8/4K3".into()));
        let lan = Lan::try_from("e8c8")?;

        let initial = board.clone();
        let undoer = board.make_move(lan)?;

        assert_eq!(
            undoer,
            MoveUndoer {
                lan,
                previous: None,
                modifer: Some(MoveModifier::Castle)
            }
        );

        board.unmake_move(undoer);

        assert_eq!(board, initial);

        Ok(())
    }

    #[test]
    fn test_placement_from_board() -> Result<(), ChessError> {
        let mut board = Board::default();

        board.make_move(Lan::try_from("e2e4")?)?;

        let placement = Placement::from(board);
        assert_eq!(
            placement,
            Placement("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR".into())
        );

        let mut board = Board::default();

        board.make_move(Lan::try_from("e2e4")?)?;
        board.make_move(Lan::try_from("c7c5")?)?;
        board.make_move(Lan::try_from("g1f3")?)?;
        board.make_move(Lan::try_from("d7d6")?)?;

        let placement = Placement::from(board);
        assert_eq!(
            placement,
            Placement("rnbqkbnr/pp2pppp/3p4/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R".into())
        );

        Ok(())
    }

    #[test]
    fn test_state_make_move() -> Result<(), ChessError> {
        let assert_make_move = |starting_fen: &str, lan: &str, expected_fen: &str| {
            let mut state = State::from(Fen::try_from(starting_fen)?);

            state.make_move(Lan::try_from(lan)?)?;

            let expected = State::from(Fen::try_from(expected_fen)?);

            assert_eq!(state, expected);

            Ok(()) as Result<(), ChessError>
        };

        // Advance a pawn two squares; the enemy is not in a position to take en passant.
        assert_make_move(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "e2e4",
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
        )?;

        // Advance a pawn two squares; the enemy is in a position to take en passant.
        assert_make_move(
            "rnbqkbnr/ppp1pppp/8/8/3p4/8/PPPPPPPP/RNBQKBNR w KQkq - 0 3",
            "e2e4",
            "rnbqkbnr/ppp1pppp/8/8/3pP3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 3",
        )?;

        // Taking en passant results in check.
        assert_make_move(
            "8/8/8/8/1k3p1R/8/4P3/4K3 w - - 0 1",
            "e2e4",
            "8/8/8/8/1k2Pp1R/8/8/4K3 b - - 0 1",
        )?;

        // Castle kingside.
        assert_make_move(
            "r1bqkbnr/pp1npppp/3p4/1Bp5/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 2 4",
            "e1g1",
            "r1bqkbnr/pp1npppp/3p4/1Bp5/4P3/5N2/PPPP1PPP/RNBQ1RK1 b kq - 3 4",
        )?;

        // The kingside rook moves; the king can no longer castle king side.
        assert_make_move(
            "r1bqkbnr/pp1npppp/3p4/1Bp5/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 2 4",
            "h1f1",
            "r1bqkbnr/pp1npppp/3p4/1Bp5/4P3/5N2/PPPP1PPP/RNBQKR2 b Qkq - 3 4",
        )?;

        // The kingside rook is captured; the king can no longer castle king side.
        assert_make_move(
            "rnbqkb1r/pppppppp/8/8/8/6n1/PPPPPPPP/RNBQKBNR b KQkq - 7 4",
            "g3h1",
            "rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNn w Qkq - 0 5",
        )?;

        // Promote a pawn to a queen.
        assert_make_move(
            "rnbqkbnr/ppppppPp/8/8/8/8/PPPPPPP1/RNBQKBNR w KQkq - 1 5",
            "g7h8q",
            "rnbqkbnQ/pppppp1p/8/8/8/8/PPPPPPP1/RNBQKBNR b KQq - 0 5",
        )?;

        Ok(())
    }

    #[test]
    fn test_state_unmake_move() -> Result<(), ChessError> {
        let assert_make_unmake_move = |fen: &str, lan: &str| {
            let mut state = State::from(Fen::try_from(fen)?);
            let initial = state.clone();

            let undoer = state.make_move(Lan::try_from(lan)?)?;
            state.unmake_move(undoer);

            assert_eq!(state, initial);

            Ok(()) as Result<(), ChessError>
        };

        // Advance a pawn two squares; the enemy is not in a position to take en passant.
        assert_make_unmake_move(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "e2e4",
        )?;

        // Advance a pawn two squares; the enemy is in a position to take en passant.
        assert_make_unmake_move(
            "rnbqkbnr/ppp1pppp/8/8/3p4/8/PPPPPPPP/RNBQKBNR w KQkq - 0 3",
            "e2e4",
        )?;

        // Taking en passant results in check.
        assert_make_unmake_move("8/8/8/8/1k3p1R/8/4P3/4K3 w - - 0 1", "e2e4")?;

        // Castle kingside.
        assert_make_unmake_move(
            "r1bqkbnr/pp1npppp/3p4/1Bp5/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 2 4",
            "e1g1",
        )?;

        // The kingside rook moves; the king can no longer castle king side.
        assert_make_unmake_move(
            "r1bqkbnr/pp1npppp/3p4/1Bp5/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 2 4",
            "h1f1",
        )?;

        // The kingside rook is captured; the king can no longer castle king side.
        assert_make_unmake_move(
            "rnbqkb1r/pppppppp/8/8/8/6n1/PPPPPPPP/RNBQKBNR b KQkq - 7 4",
            "g3h1",
        )?;

        // Promote a pawn to a queen.
        assert_make_unmake_move(
            "rnbqkbnr/ppppppPp/8/8/8/8/PPPPPPP1/RNBQKBNR w KQkq - 1 5",
            "g7h8q",
        )?;

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
    fn test_bitboard_get_set() -> Result<(), ChessError> {
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
    fn test_board_generate_pawn_danger_zone() -> Result<(), ChessError> {
        let board = Board::default();

        let danger_zone = board.generate_pawn_danger_zone(Coordinate::E1);
        assert_eq!(danger_zone, None);

        let danger_zone = board.generate_pawn_danger_zone(Coordinate::E2);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::D3, true);
        expected.set(Coordinate::F3, true);

        assert_eq!(danger_zone, Some(expected));

        let placement = Placement::try_from("r1bqkbnr/pppp1ppp/2n5/4p3/4P3/3P4/PPP2PPP/RNBQKBNR")?;
        let board = Board::from(placement);

        let danger_zone = board.generate_pawn_danger_zone(Coordinate::D3);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::C4, true);
        expected.set(Coordinate::E4, true);

        assert_eq!(danger_zone, Some(expected));

        Ok(())
    }

    #[test]
    fn test_board_generate_knight_danger_zone() -> Result<(), ChessError> {
        let board = Board::default();

        let danger_zone = board.generate_knight_danger_zone(Coordinate::E1);
        assert_eq!(danger_zone, None);

        let danger_zone = board.generate_knight_danger_zone(Coordinate::G1);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::H3, true);
        expected.set(Coordinate::E2, true);
        expected.set(Coordinate::F3, true);

        assert_eq!(danger_zone, Some(expected));

        let placement = Placement::try_from("rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R")?;
        let board = Board::from(placement);

        let danger_zone = board.generate_knight_danger_zone(Coordinate::F3);
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
    fn test_board_generate_bishop_danger_zone() -> Result<(), ChessError> {
        let board = Board::default();

        let danger_zone = board.generate_bishop_danger_zone(Coordinate::E1);
        assert_eq!(danger_zone, None);

        let danger_zone = board.generate_bishop_danger_zone(Coordinate::F1);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::E2, true);
        expected.set(Coordinate::G2, true);

        assert_eq!(danger_zone, Some(expected));

        let placement = Placement::try_from("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR")?;
        let board = Board::from(placement);

        let danger_zone = board.generate_bishop_danger_zone(Coordinate::F1);
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
        let board = Board::default();

        let danger_zone = board.generate_rook_danger_zone(Coordinate::E1);
        assert_eq!(danger_zone, None);

        let danger_zone = board.generate_rook_danger_zone(Coordinate::H1);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::H2, true);
        expected.set(Coordinate::G1, true);

        assert_eq!(danger_zone, Some(expected));

        let placement = Placement::try_from("rnbqkbnr/pppppppp/8/8/7P/7R/PPPPPPP1/RNBQKBN1")?;
        let board = Board::from(placement);

        let danger_zone = board.generate_rook_danger_zone(Coordinate::H3);
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
    fn test_board_generate_queen_danger_zone() -> Result<(), ChessError> {
        let board = Board::default();

        let danger_zone = board.generate_queen_danger_zone(Coordinate::E1);
        assert_eq!(danger_zone, None);

        let danger_zone = board.generate_queen_danger_zone(Coordinate::D1);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::D2, true);
        expected.set(Coordinate::E2, true);
        expected.set(Coordinate::E1, true);
        expected.set(Coordinate::C1, true);
        expected.set(Coordinate::C2, true);

        assert_eq!(danger_zone, Some(expected));

        let placement = Placement::try_from("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR")?;
        let board = Board::from(placement);

        let danger_zone = board.generate_queen_danger_zone(Coordinate::D1);
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
    fn test_board_generate_king_danger_zone() -> Result<(), ChessError> {
        let board = Board::default();

        let danger_zone = board.generate_king_danger_zone(Coordinate::E2);
        assert_eq!(danger_zone, None);

        let danger_zone = board.generate_king_danger_zone(Coordinate::E1);
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
    fn test_board_generate_danger_zone() -> Result<(), ChessError> {
        let board = Board::default();

        let danger_zone = board.generate_danger_zone(Color::White);
        assert_eq!(danger_zone.population_count(), 22);

        let danger_zone = board.generate_danger_zone(Color::Black);
        assert_eq!(danger_zone.population_count(), 22);

        let placement = Placement::try_from("rnbqkbnr/pp2pppp/3p4/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R")?;
        let board = Board::from(placement);

        let danger_zone = board.generate_danger_zone(Color::White);
        assert_eq!(danger_zone.population_count(), 31);

        let danger_zone = board.generate_danger_zone(Color::Black);
        assert_eq!(danger_zone.population_count(), 30);

        Ok(())
    }

    #[test]
    fn test_state_find_pins() -> Result<(), ChessError> {
        let fen = Fen::try_from("q3q3/1P4k1/4P1q1/5P2/1qP1KP1q/3P4/2q1P1P1/4q2q b - - 0 1")?;
        let state = State::from(fen);

        let pins = state.find_pins(Coordinate::E4);
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

        let pins = state.find_pins(Coordinate::G7);
        let expected = Bitboard::empty();

        assert_eq!(pins, Some(expected));

        let fen = Fen::try_from("8/1KPq2k1/8/1P6/1P2P3/8/1q6/7q w - - 0 1")?;
        let state = State::from(fen);

        let pins = state.find_pins(Coordinate::B7);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::C7, true);
        expected.set(Coordinate::E4, true);

        assert_eq!(pins, Some(expected));

        let pins = state.find_pins(Coordinate::G7);
        let expected = Bitboard::empty();

        assert_eq!(pins, Some(expected));

        // Check pins for something other than a king.
        let fen = Fen::try_from("4k3/8/8/8/q1P1P3/5P2/6b1/4K3 w - - 0 1")?;
        let state = State::from(fen);

        let pins = state.find_pins(Coordinate::E4);
        let mut expected = Bitboard::empty();
        expected.set(Coordinate::F3, true);
        expected.set(Coordinate::C4, true);

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

        let analysis = state.analyze(Color::White);

        assert_eq!(analysis.king_safety, KingSafety::Safe);
        assert_eq!(count_moves(analysis), 20);

        let fen = Fen::try_from("r2qnrk1/3nbppp/3pb3/5PP1/p2NP3/4B3/PPpQ3P/1K1R1B1R w - - 0 19")?;
        let state = State::from(fen);

        let analysis = state.analyze(Color::White);

        assert_eq!(analysis.king_safety, KingSafety::Check);
        assert_eq!(count_moves(analysis), 5);

        let fen = Fen::try_from("2r4k/4bppp/3p4/4nPP1/1n1Bq2P/1p5R/1Q1RB3/2K5 w - - 2 35")?;
        let state = State::from(fen);

        let analysis = state.analyze(Color::White);

        assert_eq!(analysis.king_safety, KingSafety::Check);
        assert_eq!(count_moves(analysis), 8);

        let fen = Fen::try_from("8/8/8/3k3r/2Pp4/8/1K6/8 b - c3 0 1")?;
        let state = State::from(fen);

        let analysis = state.analyze(Color::Black);

        assert_eq!(analysis.king_safety, KingSafety::Check);
        assert_eq!(count_moves(analysis), 8);

        let fen = Fen::try_from("r1bqkbnr/pppp1Qpp/8/4p3/2BnP3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 4")?;
        let state = State::from(fen);

        let analysis = state.analyze(Color::Black);

        assert_eq!(analysis.king_safety, KingSafety::Checkmate);
        assert_eq!(count_moves(analysis), 0);

        let fen = Fen::try_from("k7/2Q5/1K6/8/8/8/8/8 b - - 0 1")?;
        let state = State::from(fen);

        let analysis = state.analyze(Color::Black);

        assert_eq!(analysis.king_safety, KingSafety::Stalemate);
        assert_eq!(count_moves(analysis), 0);

        let fen = Fen::try_from("rnbqk1nr/pppp1ppp/4p3/8/1b6/3P4/PPPKPPPP/RNBQ1BNR w kq - 2 3")?;
        let state = State::from(fen);

        let analysis = state.analyze(Color::White);

        assert_eq!(analysis.king_safety, KingSafety::Check);
        assert_eq!(count_moves(analysis), 3);

        Ok(())
    }

    #[test]
    fn test_engine_make_sequence() -> Result<(), ChessError> {
        let mut state = State::default();

        let result = Engine::make_sequence(&mut state, &[Lan::try_from("e2e5")?]);

        assert!(result.is_err());

        let mut state = State::from(Fen::try_from(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )?);

        let result = Engine::make_sequence(
            &mut state,
            &[Lan::try_from("e2a6")?, Lan::try_from("e2e4")?],
        );

        assert!(result.is_err());

        let mut state = State::default();

        let result = Engine::make_sequence(&mut state, &[Lan::try_from("e2e4")?]);

        assert!(result.is_ok());
        assert_eq!(
            state,
            State::from(Fen::try_from(
                "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
            )?)
        );

        let mut state = State::from(Fen::try_from(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )?);

        let result = Engine::make_sequence(
            &mut state,
            &[
                Lan::try_from("e2a6")?,
                Lan::try_from("h3g2")?,
                Lan::try_from("f3g2")?,
            ],
        );

        assert!(result.is_ok());
        assert_eq!(
            state,
            State::from(Fen::try_from(
                "r3k2r/p1ppqpb1/Bn2pnp1/3PN3/1p2P3/2N5/PPPB1PQP/R3K2R b KQkq - 0 2"
            )?)
        );

        Ok(())
    }

    #[test]
    fn test_engine_analyze() -> Result<(), ChessError> {
        // We cannot reliably test most of InfoStatistics' properties, but we can test whether or
        // not the engine can find mate in x amount of moves.

        let mut state = State::from(Fen::try_from(
            "6k1/pp3r2/6rp/3QN3/5p2/2P1p2R/PPq3PP/4R1K1 b - - 0 1",
        )?);

        let info = Engine::analyze(&mut state, 3, None);

        assert_eq!(info.score, Some(Score::Mate(2)));

        let mut state = State::from(Fen::try_from(
            "6k1/pp3r2/6rp/3QN3/5p2/2P1p2R/PP3qPP/4R1K1 w - - 1 2",
        )?);

        let info = Engine::analyze(&mut state, 3, None);

        assert_eq!(info.score, Some(Score::Mate(-1)));

        Ok(())
    }
}
