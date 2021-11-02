mod utils;

use bitflags::bitflags;
use wasm_bindgen::prelude::*;

const BOARD_WIDTH: usize = 8;
const BOARD_HEIGHT: usize = 8;

enum Color {
    White,
    Black,
}

#[derive(Debug, PartialEq, Eq)]
enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl TryFrom<char> for PieceType {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        let value = value.to_ascii_lowercase();

        match value {
            'p' => Ok(PieceType::Pawn),
            'n' => Ok(PieceType::Knight),
            'b' => Ok(PieceType::Bishop),
            'r' => Ok(PieceType::Rook),
            'q' => Ok(PieceType::Queen),
            'k' => Ok(PieceType::King),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for PieceType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.chars().count() == 1 {
            if let Some(character) = value.chars().next() {
                return PieceType::try_from(character);
            }
        }

        Err(())
    }
}

impl TryFrom<String> for PieceType {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        PieceType::try_from(&value[..])
    }
}

struct Piece(Color, PieceType);

impl TryFrom<char> for Piece {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'p' => Ok(Piece(Color::White, PieceType::Pawn)),
            'n' => Ok(Piece(Color::White, PieceType::Knight)),
            'b' => Ok(Piece(Color::White, PieceType::Bishop)),
            'r' => Ok(Piece(Color::White, PieceType::Rook)),
            'q' => Ok(Piece(Color::White, PieceType::Queen)),
            'k' => Ok(Piece(Color::White, PieceType::King)),
            'P' => Ok(Piece(Color::Black, PieceType::Pawn)),
            'N' => Ok(Piece(Color::Black, PieceType::Knight)),
            'B' => Ok(Piece(Color::Black, PieceType::Bishop)),
            'R' => Ok(Piece(Color::Black, PieceType::Rook)),
            'Q' => Ok(Piece(Color::Black, PieceType::Queen)),
            'K' => Ok(Piece(Color::Black, PieceType::King)),
            _ => Err(()),
        }
    }
}

// TODO(thismarvin): Is there a more idiomatic approach to this?
bitflags! {
    struct CastlingAbility : u32 {
        const WHITE_KING_SIDE = 1 << 0;
        const WHITE_QUEEN_SIDE = 1 << 1;
        const BLACK_KING_SIDE = 1 << 2;
        const BLACK_QUEEN_SIDE = 1 << 3;
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Coordinate(usize);

impl TryFrom<usize> for Coordinate {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value >= BOARD_WIDTH * BOARD_HEIGHT {
            return Err(());
        }

        Ok(Coordinate(value))
    }
}

impl TryFrom<&str> for Coordinate {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.to_lowercase();
        let mut characters = value.chars();

        match (characters.next(), characters.next()) {
            (Some(file), Some(rank)) => {
                if !file.is_ascii_alphabetic() || !rank.is_ascii_digit() {
                    return Err(());
                }

                let x = file as u32 - 'a' as u32;

                if x >= BOARD_WIDTH as u32 {
                    return Err(());
                }

                let y = rank.to_digit(10).ok_or(())?;

                if y == 0 || y > BOARD_HEIGHT as u32 {
                    return Err(());
                }

                let y = BOARD_HEIGHT as u32 - y;

                let index = (y * BOARD_WIDTH as u32 + x) as usize;

                Ok(Coordinate(index))
            }
            _ => Err(()),
        }
    }
}

impl TryFrom<String> for Coordinate {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Coordinate::try_from(&value[..])
    }
}

#[derive(Debug, PartialEq, Eq)]
struct LAN {
    start: Coordinate,
    end: Coordinate,
    promotion: Option<PieceType>,
}

impl TryFrom<&str> for LAN {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() < 4 || value.len() > 5 {
            return Err(());
        }

        let value = value.to_lowercase();

        let mut start = value.chars();
        let start = Coordinate::try_from(format!(
            "{}{}",
            start.next().unwrap_or('_'),
            start.next().unwrap_or('_')
        ))?;

        let mut end = value.chars().skip(2);
        let end = Coordinate::try_from(format!(
            "{}{}",
            end.next().unwrap_or('_'),
            end.next().unwrap_or('_')
        ))?;

        let character = value.chars().skip(4).next();

        match character {
            Some(character) => match PieceType::try_from(character) {
                Ok(promotion) => Ok(LAN {
                    start,
                    end,
                    promotion: Some(promotion),
                }),
                Err(_) => Err(()),
            },
            None => Ok(LAN {
                start,
                end,
                promotion: None,
            }),
        }
    }
}

impl TryFrom<String> for LAN {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        LAN::try_from(&value[..])
    }
}

struct FEN<'a> {
    placement: &'a str,
    side_to_move: Color,
    castling_ability: Option<CastlingAbility>,
    en_passant_target: Option<Coordinate>,
    half_moves: usize,
    full_moves: usize,
}

struct Board {
    pieces: [Piece; BOARD_WIDTH * BOARD_HEIGHT],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_from_usize() {
        let coordinate = Coordinate::try_from(32);
        assert_eq!(coordinate, Ok(Coordinate(32)));

        let coordinate = Coordinate::try_from(128);
        assert!(coordinate.is_err());
    }

    #[test]
    fn test_coordinate_from_str() {
        let coordinate = Coordinate::try_from("a8");
        assert_eq!(coordinate, Ok(Coordinate(0)));

        let coordinate = Coordinate::try_from("e4");
        assert_eq!(coordinate, Ok(Coordinate(36)));

        let coordinate = Coordinate::try_from("h1");
        assert_eq!(coordinate, Ok(Coordinate(63)));

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
    }

    #[test]
    fn test_lan_from_str() {
        let lan = LAN::try_from("a1a9");
        assert!(lan.is_err());

        let lan = LAN::try_from("e2e1m");
        assert!(lan.is_err());

        let lan = LAN::try_from("a1a2");
        assert_eq!(
            lan,
            Ok(LAN {
                start: Coordinate::try_from("a1").unwrap(),
                end: Coordinate::try_from("a2").unwrap(),
                promotion: None
            })
        );

        let lan = LAN::try_from("e7e8q");
        assert_eq!(
            lan,
            Ok(LAN {
                start: Coordinate::try_from("e7").unwrap(),
                end: Coordinate::try_from("e8").unwrap(),
                promotion: Some(PieceType::Queen)
            })
        );
    }
}
