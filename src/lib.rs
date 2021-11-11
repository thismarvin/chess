mod utils;

use bitflags::bitflags;
use std::borrow::Borrow;
use std::ops::{BitOr, BitOrAssign, Index, IndexMut};
use wasm_bindgen::prelude::*;

const BOARD_WIDTH: u8 = 8;
const BOARD_HEIGHT: u8 = 8;
const MOVE_LIST_CAPACITY: usize = 27;
const STARTING_PLACEMENT: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

#[derive(Debug, PartialEq, Eq)]
struct ChessError(ChessErrorKind, &'static str);

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

impl From<Piece> for &str {
    fn from(value: Piece) -> Self {
        match value {
            Piece(Color::White, PieceKind::Pawn) => "P",
            Piece(Color::White, PieceKind::Knight) => "N",
            Piece(Color::White, PieceKind::Bishop) => "B",
            Piece(Color::White, PieceKind::Rook) => "R",
            Piece(Color::White, PieceKind::Queen) => "Q",
            Piece(Color::White, PieceKind::King) => "K",
            Piece(Color::Black, PieceKind::Pawn) => "p",
            Piece(Color::Black, PieceKind::Knight) => "n",
            Piece(Color::Black, PieceKind::Bishop) => "b",
            Piece(Color::Black, PieceKind::Rook) => "r",
            Piece(Color::Black, PieceKind::Queen) => "q",
            Piece(Color::Black, PieceKind::King) => "k",
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

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct LAN {
    start: Coordinate,
    end: Coordinate,
    promotion: Option<PieceKind>,
}

impl TryFrom<&str> for LAN {
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
                Ok(promotion) => Ok(LAN {
                    start,
                    end,
                    promotion: Some(promotion),
                }),
                Err(error) => Err(error),
            },
            None => Ok(LAN {
                start,
                end,
                promotion: None,
            }),
        }
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
        let ranks: Vec<&str> = value.split("/").collect();

        if ranks.len() != BOARD_HEIGHT as usize {
            return Err(ChessError(
                ChessErrorKind::InvalidString,
                "A valid Placement must consist of eight sections separated by a forward slash.",
            ));
        }

        for rank in ranks {
            let characters = rank.chars();
            let mut reach = 0 as usize;

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
        let mut placement = "".to_string();

        let mut index = 0;
        let mut empty = 0;

        for piece in value.pieces {
            if let Some(piece) = piece {
                if empty != 0 {
                    placement.push_str(&empty.to_string()[..]);
                    empty = 0;
                }

                placement.push_str(piece.into());
            } else {
                empty += 1;
            }

            index += 1;

            if index == 8 {
                if empty > 0 {
                    placement.push_str(&empty.to_string()[..]);
                    empty = 0;
                }

                placement.push_str("/");

                index = 0;
            }
        }

        let placement = placement.strip_suffix("/").expect(
            "A forward slash should always be concatenated to the end of the string slice.",
        );

        Placement(String::from(placement))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct FEN {
    placement: Placement,
    side_to_move: Color,
    castling_ability: Option<CastlingAbility>,
    en_passant_target: Option<Coordinate>,
    half_moves: usize,
    full_moves: usize,
}

impl FEN {
    fn apply_move(&self, lan: LAN) -> Result<FEN, ChessError> {
        let mut board = Board::from(&self.placement);

        let piece = board[lan.start];
        let piece = piece.ok_or(ChessError(
            ChessErrorKind::TargetIsNone,
            "Cannot move a piece that does not exist.",
        ))?;

        let target = board[lan.end];

        let capture = matches!(target, Some(_));

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
        match piece {
            Piece(color, PieceKind::King) => {
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
                                    ^ (CastlingAbility::WHITE_KINGSIDE
                                        | CastlingAbility::WHITE_QUEENSIDE),
                            );
                        }
                    }
                    Color::Black => {
                        if let Some(ability) = castling_ability {
                            castling_ability = Some(
                                ability
                                    ^ (CastlingAbility::BLACK_KINGSIDE
                                        | CastlingAbility::BLACK_QUEENSIDE),
                            );
                        }
                    }
                }
            }
            _ => (),
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
                        castling_ability = Some(ability ^ king_side);
                    }
                } else if lan.start as u8 == queen_side_index {
                    if let Some(ability) = castling_ability {
                        castling_ability = Some(ability ^ queen_side);
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

            match lan.end.try_move(-1, 0) {
                Ok(coordinate) => match board[coordinate] {
                    Some(Piece(color, PieceKind::Pawn)) if color == side_to_move => {
                        en_passant_target = Some(potential_en_passant_target);
                        pawns += 1;
                    }
                    _ => (),
                },
                _ => (),
            }
            match lan.end.try_move(1, 0) {
                Ok(coordinate) => match board[coordinate] {
                    Some(Piece(color, PieceKind::Pawn)) if color == side_to_move => {
                        en_passant_target = Some(potential_en_passant_target);
                        pawns += 1;
                    }
                    _ => (),
                },
                _ => (),
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
                        Some(Piece(_, PieceKind::King)) => {
                            kings_coordinate = Some(Coordinate::try_from(index as u8)?);
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
                            Some(Piece(color, piece_type)) if color == self.side_to_move => {
                                if let PieceKind::Rook | PieceKind::Queen = piece_type {
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

        Ok(FEN {
            placement,
            side_to_move,
            castling_ability,
            en_passant_target,
            half_moves,
            full_moves,
        })
    }
}

impl Default for FEN {
    fn default() -> Self {
        FEN {
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

impl TryFrom<&str> for FEN {
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

            CastlingAbility::try_from(castling_ability).map(|result| Some(result))
        })()?;

        let en_passant_target = sections[3];
        let en_passant_target = (|| {
            if en_passant_target == "-" {
                return Ok(None);
            }

            Coordinate::try_from(en_passant_target).map(|result| Some(result))
        })()?;

        let half_moves = sections[4];
        let half_moves: usize = half_moves
            .parse()
            .map_err(|_| ChessError(ChessErrorKind::InvalidString, "Expected a number."))?;

        let full_moves = sections[5];
        let full_moves: usize = full_moves
            .parse()
            .map_err(|_| ChessError(ChessErrorKind::InvalidString, "Expected a number."))?;

        Ok(FEN {
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
    fn apply_move(&self, lan: LAN) -> Result<Board, ChessError> {
        let mut pieces = self.pieces.clone();

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
        let ranks: Vec<&str> = value.0.split("/").collect();

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
        Bitboard(0)
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

#[derive(Default)]
struct State {
    fen: FEN,
    board: Board,
}

impl State {
    fn walk(&self, moves: &mut Vec<LAN>, start: Coordinate, opponent: Color, dx: i8, dy: i8) {
        let size = BOARD_WIDTH.max(BOARD_HEIGHT) as i8;

        let mut push_move = |end: Coordinate| {
            moves.push(LAN {
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

    fn generate_pseudo_legal_pawn_moves(&self, start: Coordinate) -> Vec<LAN> {
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
                    moves.push(LAN {
                        start,
                        end,
                        promotion: Some(kind),
                    });
                }
            } else {
                moves.push(LAN {
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
                    if let None = self.board[end] {
                        register_move(end);
                    }
                }

                // Handle advancing two squares (if the pawn has not moved before).
                if start.y() == BOARD_HEIGHT - 2 {
                    if let (Ok(prerequisite), Ok(end)) =
                        (start.try_move(0, 1), start.try_move(0, 2))
                    {
                        if let (None, None) = (self.board[prerequisite], self.board[end]) {
                            register_move(end)
                        }
                    }
                }

                // Handle capturing to the top left.
                if let Ok(end) = start.try_move(-1, 1) {
                    if let Some(Piece(Color::Black, _)) = self.board[end] {
                        register_move(end)
                    }
                }

                // Handle capturing to the top right.
                if let Ok(end) = start.try_move(1, 1) {
                    if let Some(Piece(Color::Black, _)) = self.board[end] {
                        register_move(end)
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
                    if let None = self.board[end] {
                        register_move(end)
                    }
                }

                // Handle advancing two squares (if the pawn has not moved before).
                if start.y() == 1 {
                    if let (Ok(prerequisite), Ok(end)) =
                        (start.try_move(0, -1), start.try_move(0, -2))
                    {
                        if let (None, None) = (self.board[prerequisite], self.board[end]) {
                            register_move(end)
                        }
                    }
                }

                // Handle capturing to the bottom left.
                if let Ok(end) = start.try_move(-1, -1) {
                    if let Some(Piece(Color::White, _)) = self.board[end] {
                        register_move(end)
                    }
                }

                // Handle capturing to the bottom right.
                if let Ok(end) = start.try_move(1, -1) {
                    if let Some(Piece(Color::White, _)) = self.board[end] {
                        register_move(end)
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

    fn generate_pseudo_legal_knight_moves(&self, start: Coordinate) -> Vec<LAN> {
        let mut moves = Vec::with_capacity(MOVE_LIST_CAPACITY);

        if let Some(Piece(color, PieceKind::Knight)) = self.board[start] {
            let opponent = color.opponent();

            let mut try_register_move = |dx: i8, dy: i8| {
                let mut push_move = |end: Coordinate| {
                    moves.push(LAN {
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

    fn generate_pseudo_legal_bishop_moves(&self, start: Coordinate) -> Vec<LAN> {
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

    fn generate_pseudo_legal_rook_moves(&self, start: Coordinate) -> Vec<LAN> {
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

    fn generate_pseudo_legal_queen_moves(&self, start: Coordinate) -> Vec<LAN> {
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

    fn generate_pseudo_legal_king_moves(&self, start: Coordinate) -> Vec<LAN> {
        let mut moves = Vec::with_capacity(MOVE_LIST_CAPACITY);

        let mut push_move = |end: Coordinate| {
            moves.push(LAN {
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
                            push_move(end)
                        }
                    }
                }

                if (castling_ability & queen_side) != CastlingAbility::empty() {
                    if let (Ok(prerequisite_a), Ok(prerequisite_b), Ok(end)) = (
                        start.try_move(-1, 0),
                        start.try_move(-2, 0),
                        start.try_move(-3, 0),
                    ) {
                        if let (None, None, None) = (
                            self.board[prerequisite_a],
                            self.board[prerequisite_b],
                            self.board[end],
                        ) {
                            push_move(end)
                        }
                    }
                }
            }
        }

        moves
    }

    fn generate_pseudo_legal_moves(&self, color: Color) -> Vec<Option<Vec<LAN>>> {
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

impl From<FEN> for State {
    fn from(value: FEN) -> Self {
        let board = Board::from(&value.placement);

        State { fen: value, board }
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
        let lan = LAN::try_from("a1a9");
        assert!(lan.is_err());

        let lan = LAN::try_from("e2e1m");
        assert!(lan.is_err());

        let lan = LAN::try_from("a1a2");
        assert_eq!(
            lan,
            Ok(LAN {
                start: Coordinate::try_from("a1")?,
                end: Coordinate::try_from("a2")?,
                promotion: None,
            })
        );

        let lan = LAN::try_from("e7e8q");
        assert_eq!(
            lan,
            Ok(LAN {
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
        let fen = FEN::try_from("what is a fen string for?");
        assert!(fen.is_err());

        let fen = FEN::try_from("rnbqkbnr /pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert!(fen.is_err());

        let fen = FEN::try_from("rnbqkbnrr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert!(fen.is_err());

        let fen = FEN::try_from("rnbqkbnr/pppppppp/9/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert!(fen.is_err());

        let fen = FEN::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR m KQkq - 0 1");
        assert!(fen.is_err());

        let fen = FEN::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w king - 0 1");
        assert!(fen.is_err());

        let fen = FEN::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq m1 0 1");
        assert!(fen.is_err());

        let fen = FEN::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - a 1");
        assert!(fen.is_err());

        let fen = FEN::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 a");
        assert!(fen.is_err());

        let fen = FEN::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(fen, Ok(FEN::default()));

        let fen = FEN::try_from("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
        assert_eq!(
            fen,
            Ok(FEN {
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
            FEN::try_from("r2qkbnr/pp1n1ppp/2p1p3/3pPb2/3P4/5N2/PPP1BPPP/RNBQ1RK1 b kq - 3 6 ");
        assert_eq!(
            fen,
            Ok(FEN {
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
            FEN::try_from("r4rk1/2qn1pb1/1p2p1np/3pPb2/8/1N1N2B1/PPP1B1PP/R2Q1RK1 w - - 3 17");
        assert_eq!(
            fen,
            Ok(FEN {
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
        let lan = LAN::try_from("e3e4")?;
        let result = board.apply_move(lan);
        assert!(result.is_err());

        let board = Board::from(Placement("1k6/6R1/1K6/8/8/8/8/8".into()));
        let lan = LAN::try_from("g7g8q")?;
        let result = board.apply_move(lan);
        assert!(result.is_err());

        let board = Board::default();
        let lan = LAN::try_from("e2e4")?;
        let result = board.apply_move(lan);
        assert!(result.is_ok());
        let result = result?;
        assert_eq!(result[Coordinate::E2], None);
        assert_eq!(
            result[Coordinate::E4],
            Some(Piece(Color::White, PieceKind::Pawn))
        );

        let board = Board::from(Placement("8/2k1PK2/8/8/8/8/8/8".into()));
        let lan = LAN::try_from("e7e8q")?;
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

        let board = initial.apply_move(LAN::try_from("e2e4")?)?;
        let placement = Placement::from(board);
        assert_eq!(
            placement,
            Placement("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR".into())
        );

        let board = initial.apply_move(LAN::try_from("e2e4")?)?;
        let board = board.apply_move(LAN::try_from("c7c5")?)?;
        let board = board.apply_move(LAN::try_from("g1f3")?)?;
        let board = board.apply_move(LAN::try_from("d7d6")?)?;
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
        let fen = FEN::default();
        let result = fen.apply_move(LAN::try_from("e2e4")?);
        assert_eq!(
            result,
            Ok(FEN::try_from(
                "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
            )?)
        );

        // Advance a pawn two squares; the enemy is in a position to take en passant.
        let fen = FEN::try_from("rnbqkbnr/ppp1pppp/8/8/3p4/8/PPPPPPPP/RNBQKBNR w KQkq - 0 3")?;
        let result = fen.apply_move(LAN::try_from("e2e4")?);
        assert_eq!(
            result,
            Ok(FEN::try_from(
                "rnbqkbnr/ppp1pppp/8/8/3pP3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 3"
            )?)
        );

        // Taking en passant results in check.
        let fen = FEN::try_from("8/8/8/8/1k3p1R/8/4P3/4K3 w - - 0 1")?;
        let result = fen.apply_move(LAN::try_from("e2e4")?);
        assert_eq!(
            result,
            Ok(FEN::try_from("8/8/8/8/1k2Pp1R/8/8/4K3 b - - 0 1")?)
        );

        // Castle kingside.
        let fen =
            FEN::try_from("r1bqkbnr/pp1npppp/3p4/1Bp5/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 2 4")?;
        let result = fen.apply_move(LAN::try_from("e1g1")?);
        assert_eq!(
            result,
            Ok(FEN::try_from(
                "r1bqkbnr/pp1npppp/3p4/1Bp5/4P3/5N2/PPPP1PPP/RNBQ1RK1 b kq - 3 4"
            )?)
        );

        // The kingside rook moves; the king can no longer castle king side.
        let fen =
            FEN::try_from("r1bqkbnr/pp1npppp/3p4/1Bp5/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 2 4")?;
        let result = fen.apply_move(LAN::try_from("h1f1")?);
        assert_eq!(
            result,
            Ok(FEN::try_from(
                "r1bqkbnr/pp1npppp/3p4/1Bp5/4P3/5N2/PPPP1PPP/RNBQKR2 b Qkq - 3 4"
            )?)
        );

        // The kingside rook is captured; the king can no longer castle king side.
        let fen = FEN::try_from("rnbqkb1r/pppppppp/8/8/8/6n1/PPPPPPPP/RNBQKBNR b KQkq - 7 4")?;
        let result = fen.apply_move(LAN::try_from("g3h1")?);
        assert_eq!(
            result,
            Ok(FEN::try_from(
                "rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNn w Qkq - 0 5"
            )?)
        );

        // Promote a pawn to a queen.
        let fen = FEN::try_from("rnbqkbnr/ppppppPp/8/8/8/8/PPPPPPP1/RNBQKBNR w KQkq - 1 5")?;
        let result = fen.apply_move(LAN::try_from("g7h8q")?);
        assert_eq!(
            result,
            Ok(FEN::try_from(
                "rnbqkbnQ/pppppp1p/8/8/8/8/PPPPPPP1/RNBQKBNR b KQq - 0 5"
            )?)
        );

        Ok(())
    }

    #[test]
    fn test_board_generate_pseudo_legal_pawn_moves() -> Result<(), ChessError> {
        // Moving None should return an empty move list.
        let state = State::default();
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::E1);
        assert_eq!(move_list, vec![]);

        // A pawn that hasn't moved should be able to advance one or two squares.
        let state = State::default();
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::E2);
        assert_eq!(
            move_list,
            vec![LAN::try_from("e2e3")?, LAN::try_from("e2e4")?]
        );

        let fen = FEN::try_from("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::E7);
        assert_eq!(
            move_list,
            vec![LAN::try_from("e7e6")?, LAN::try_from("e7e5")?]
        );

        // A pawn that has already moved should only be able to advance one square.
        let fen = FEN::try_from("rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 1 2")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::E4);
        assert_eq!(move_list, vec![LAN::try_from("e4e5")?]);

        let fen = FEN::try_from("rnbqkbnr/pppp1ppp/8/4p3/8/8/PPPPPPPP/RNBQKBNR b KQkq - 1 2")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::E5);
        assert_eq!(move_list, vec![LAN::try_from("e5e4")?]);

        // Test capturing to the top left.
        let fen = FEN::try_from("r1bqkb1r/pppppppp/2n2n2/3P4/8/8/PPP1PPPP/RNBQKBNR w KQkq - 1 3")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::D5);
        assert_eq!(
            move_list,
            vec![LAN::try_from("d5d6")?, LAN::try_from("d5c6")?]
        );

        // Test capturing to the top right.
        let fen = FEN::try_from("r1bqkb1r/pppppppp/2n2n2/4P3/8/8/PPPP1PPP/RNBQKBNR w KQkq - 1 3")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::E5);
        assert_eq!(
            move_list,
            vec![LAN::try_from("e5e6")?, LAN::try_from("e5f6")?]
        );

        // Test capturing to the bottom left.
        let fen =
            FEN::try_from("rnbqkb1r/pppp1ppp/5n2/4p3/2PP4/5N2/PP2PPPP/RNBQKB1R b KQkq - 0 3")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::E5);
        assert_eq!(
            move_list,
            vec![LAN::try_from("e5e4")?, LAN::try_from("e5d4")?]
        );

        // Test capturing to the bottom right.
        let fen = FEN::try_from("rnbqkbnr/ppp1pppp/8/3p4/4P3/2N5/PPPP1PPP/R1BQKBNR b KQkq - 1 2")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::D5);
        assert_eq!(
            move_list,
            vec![LAN::try_from("d5d4")?, LAN::try_from("d5e4")?]
        );

        // Test ability to capture en passant.
        let fen = FEN::try_from("rnbqkbnr/ppppp1pp/8/4Pp2/8/8/PPPPKPPP/RNBQ1BNR w kq f6 0 4")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::E5);
        assert_eq!(
            move_list,
            vec![LAN::try_from("e5e6")?, LAN::try_from("e5f6")?]
        );

        let fen = FEN::try_from("rnbqkbnr/ppppp1pp/8/8/4Pp2/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 3")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::F4);
        assert_eq!(
            move_list,
            vec![LAN::try_from("f4f3")?, LAN::try_from("f4e3")?]
        );

        // Test promotion.
        let fen = FEN::try_from("rnbqk1nr/ppppppPp/8/6p1/8/8/PPPPPPP1/RNBQKBNR w KQkq - 1 5")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_pawn_moves(Coordinate::G7);
        assert_eq!(
            move_list,
            vec![
                LAN::try_from("g7h8n")?,
                LAN::try_from("g7h8b")?,
                LAN::try_from("g7h8r")?,
                LAN::try_from("g7h8q")?
            ]
        );

        Ok(())
    }

    #[test]
    fn test_board_generate_pseudo_legal_knight_moves() -> Result<(), ChessError> {
        let state = State::default();
        let move_list = state.generate_pseudo_legal_knight_moves(Coordinate::E1);
        assert_eq!(move_list, vec![]);

        let state = State::default();
        let move_list = state.generate_pseudo_legal_knight_moves(Coordinate::G1);
        assert_eq!(
            move_list,
            vec![LAN::try_from("g1h3")?, LAN::try_from("g1f3")?]
        );

        let fen = FEN::try_from("rnbqkbnr/pppp1ppp/8/4p3/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 0 2")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_knight_moves(Coordinate::F3);
        assert_eq!(
            move_list,
            vec![
                LAN::try_from("f3g5")?,
                LAN::try_from("f3h4")?,
                LAN::try_from("f3g1")?,
                LAN::try_from("f3d4")?,
                LAN::try_from("f3e5")?,
            ]
        );

        Ok(())
    }

    #[test]
    fn test_board_generate_pseudo_legal_bishop_moves() -> Result<(), ChessError> {
        let state = State::default();
        let move_list = state.generate_pseudo_legal_bishop_moves(Coordinate::E1);
        assert_eq!(move_list, vec![]);

        let state = State::default();
        let move_list = state.generate_pseudo_legal_bishop_moves(Coordinate::F1);
        assert_eq!(move_list, vec![]);

        let fen = FEN::try_from("r1bqkbnr/pppppppp/8/1n6/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 5 4")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_bishop_moves(Coordinate::F1);
        assert_eq!(
            move_list,
            vec![
                LAN::try_from("f1e2")?,
                LAN::try_from("f1d3")?,
                LAN::try_from("f1c4")?,
                LAN::try_from("f1b5")?,
            ]
        );

        Ok(())
    }

    #[test]
    fn test_board_generate_pseudo_legal_rook_moves() -> Result<(), ChessError> {
        let state = State::default();
        let move_list = state.generate_pseudo_legal_rook_moves(Coordinate::E1);
        assert_eq!(move_list, vec![]);

        let state = State::default();
        let move_list = state.generate_pseudo_legal_rook_moves(Coordinate::H1);
        assert_eq!(move_list, vec![]);

        let fen = FEN::try_from("rnbqkb1r/pppppppp/8/8/7P/2n4R/PPPPPPP1/R1BQKBN1 w Qkq - 0 4")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_rook_moves(Coordinate::H3);
        assert_eq!(
            move_list,
            vec![
                LAN::try_from("h3h2")?,
                LAN::try_from("h3h1")?,
                LAN::try_from("h3g3")?,
                LAN::try_from("h3f3")?,
                LAN::try_from("h3e3")?,
                LAN::try_from("h3d3")?,
                LAN::try_from("h3c3")?,
            ]
        );

        Ok(())
    }

    #[test]
    fn test_board_generate_pseudo_legal_queen_moves() -> Result<(), ChessError> {
        let state = State::default();
        let move_list = state.generate_pseudo_legal_queen_moves(Coordinate::E1);
        assert_eq!(move_list, vec![]);

        let state = State::default();
        let move_list = state.generate_pseudo_legal_queen_moves(Coordinate::D1);
        assert_eq!(move_list, vec![]);

        let fen = FEN::try_from("r1bqkbnr/pppp1ppp/2n5/4p2Q/4P3/8/PPPP1PPP/RNB1KBNR w KQkq - 2 3")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_queen_moves(Coordinate::H5);
        assert_eq!(
            move_list,
            vec![
                LAN::try_from("h5h6")?,
                LAN::try_from("h5h7")?,
                LAN::try_from("h5h4")?,
                LAN::try_from("h5h3")?,
                LAN::try_from("h5g4")?,
                LAN::try_from("h5f3")?,
                LAN::try_from("h5e2")?,
                LAN::try_from("h5d1")?,
                LAN::try_from("h5g5")?,
                LAN::try_from("h5f5")?,
                LAN::try_from("h5e5")?,
                LAN::try_from("h5g6")?,
                LAN::try_from("h5f7")?,
            ]
        );

        Ok(())
    }

    #[test]
    fn test_board_generate_pseudo_legal_king_moves() -> Result<(), ChessError> {
        let state = State::default();
        let move_list = state.generate_pseudo_legal_king_moves(Coordinate::E2);
        assert_eq!(move_list, vec![]);

        let state = State::default();
        let move_list = state.generate_pseudo_legal_king_moves(Coordinate::E1);
        assert_eq!(move_list, vec![]);

        let fen = FEN::try_from("rnbqkb1r/pppp1ppp/8/4p3/4n3/4K3/PPPP1PPP/RNBQ1BNR w kq - 0 4")?;
        let state = State::from(fen);
        let move_list = state.generate_pseudo_legal_king_moves(Coordinate::E3);
        assert_eq!(
            move_list,
            vec![
                LAN::try_from("e3e4")?,
                LAN::try_from("e3f4")?,
                LAN::try_from("e3f3")?,
                LAN::try_from("e3e2")?,
                LAN::try_from("e3d3")?,
                LAN::try_from("e3d4")?,
            ]
        );

        Ok(())
    }

    #[test]
    fn test_board_generate_pseudo_legal_moves() -> Result<(), ChessError> {
        let fen = FEN::try_from("rnbq1bnr/ppppkppp/8/4p3/4P3/8/PPPPKPPP/RNBQ1BNR w - - 2 3")?;
        let state = State::from(fen);

        let moves = state.generate_pseudo_legal_moves(Color::White);
        let total_moves = moves
            .iter()
            .filter_map(|entry| entry.as_ref().or(None))
            .fold(0, |accumulator, entry| accumulator + entry.len());

        assert_eq!(total_moves, 23);

        let moves = state.generate_pseudo_legal_moves(Color::Black);
        let total_moves = moves
            .iter()
            .filter_map(|entry| entry.as_ref().or(None))
            .fold(0, |accumulator, entry| accumulator + entry.len());

        assert_eq!(total_moves, 23);

        let fen = FEN::try_from("rnbqkbnr/pp2pppp/3p4/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 3")?;
        let state = State::from(fen);

        let moves = state.generate_pseudo_legal_moves(Color::White);
        let total_moves = moves
            .iter()
            .filter_map(|entry| entry.as_ref().or(None))
            .fold(0, |accumulator, entry| accumulator + entry.len());

        assert_eq!(total_moves, 28);

        let moves = state.generate_pseudo_legal_moves(Color::Black);
        let total_moves = moves
            .iter()
            .filter_map(|entry| entry.as_ref().or(None))
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
    }
}
