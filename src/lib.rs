mod utils;

use bitflags::bitflags;
use std::ops::{Index, IndexMut};
use wasm_bindgen::prelude::*;

const BOARD_WIDTH: u8 = 8;
const BOARD_HEIGHT: u8 = 8;

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

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
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
        let mut board = Board::from(self.placement.clone());

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
            let direction: isize = if dy > 0 { 1 } else { -1 };
            let target = Coordinate::try_from(
                (lan.start.y() as isize + direction) as u8 * BOARD_WIDTH + lan.start.x() as u8,
            )?;

            // Only enable en_passant_target if an enemy pawn is in position to capture en passant.
            let mut pawns = 0;

            if target.x() > 0 {
                match board.pieces.get((lan.end as u8 - 1) as usize) {
                    Some(Some(Piece(color, PieceKind::Pawn))) if *color == side_to_move => {
                        en_passant_target = Some(target);
                        pawns += 1;
                    }
                    _ => (),
                }
            }
            if target.x() < BOARD_WIDTH - 1 {
                match board.pieces.get((lan.end as u8 + 1) as usize) {
                    Some(Some(Piece(color, PieceKind::Pawn))) if *color == side_to_move => {
                        en_passant_target = Some(target);
                        pawns += 1;
                    }
                    _ => (),
                }
            }

            // Taking en passant could lead to a discovered check; we need to make sure that cannot happen.
            if pawns == 1 {
                let mut king_coords: Option<Coordinate> = None;
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
                            king_coords = Some(Coordinate::try_from(index as u8)?);
                        }
                        _ => (),
                    }

                    rank[x as usize] = target;
                }

                if let Some(king_coords) = king_coords {
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
                    let mut king_x = king_coords.x() as isize;
                    let dir_x: isize = if x > king_x as u8 { 1 } else { -1 };

                    king_x += dir_x;

                    // Walk King and check if a Rook or Queen is in its line of sight.
                    let mut danger = false;

                    while king_x > -1 && king_x < BOARD_WIDTH as isize {
                        match rank[king_x as usize] {
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

                        king_x += dir_x;
                    }

                    // Taking en passant would have resulted in a discovered check; en_passant_target should be disabled.
                    if danger {
                        en_passant_target = None;
                    }
                }
            }
        }

        // Deal with an en passant (Holy hell).
        match piece {
            Piece(_, PieceKind::Pawn) => match self.en_passant_target {
                Some(target) => {
                    if lan.end == target {
                        let direction: isize = if dy > 0 { -1 } else { 1 };
                        let index =
                            (target.y() as isize + direction) as u8 * BOARD_WIDTH + target.x();
                        let coordinate = Coordinate::try_from(index)?;

                        board[coordinate] = None;
                    }
                }
                _ => (),
            },
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

impl From<Placement> for Board {
    fn from(value: Placement) -> Self {
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
        assert_eq!(
            fen,
            Ok(FEN {
                placement: Placement("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".into()),
                side_to_move: Color::White,
                castling_ability: Some(
                    CastlingAbility::WHITE_KINGSIDE
                        | CastlingAbility::WHITE_QUEENSIDE
                        | CastlingAbility::BLACK_KINGSIDE
                        | CastlingAbility::BLACK_QUEENSIDE
                ),
                en_passant_target: None,
                half_moves: 0,
                full_moves: 1,
            })
        );

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
        let board = Board::from(Placement(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".into(),
        ));
        let lan = LAN::try_from("e3e4")?;
        let result = board.apply_move(lan);
        assert!(result.is_err());

        let board = Board::from(Placement("1k6/6R1/1K6/8/8/8/8/8".into()));
        let lan = LAN::try_from("g7g8q")?;
        let result = board.apply_move(lan);
        assert!(result.is_err());

        let board = Board::from(Placement(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".into(),
        ));
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
        let initial = Board::from(Placement(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".into(),
        ));

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
        let fen = FEN::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")?;
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
}
