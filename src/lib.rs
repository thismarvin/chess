mod utils;

use bitflags::bitflags;
use wasm_bindgen::prelude::*;

const BOARD_WIDTH: usize = 8;
const BOARD_HEIGHT: usize = 8;

enum Color {
    White,
    Black,
}

enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

enum Piece {
    Color(PieceType),
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

struct Coordinate {
    index: usize,
}

impl TryFrom<usize> for Coordinate {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value >= BOARD_WIDTH * BOARD_HEIGHT {
            return Err(());
        }

        Ok(Coordinate { index: value })
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

                Ok(Coordinate { index })
            }
            _ => Err(()),
        }
    }
}

struct LAN {
    start: Coordinate,
    end: Coordinate,
    promotion: Option<PieceType>,
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
        assert_eq!(coordinate.unwrap().index, 32);

        let coordinate = Coordinate::try_from(128);
        assert!(coordinate.is_err());
    }

    #[test]
    fn test_coordinate_from_str() {
        let coordinate = Coordinate::try_from("a8");
        assert_eq!(coordinate.unwrap().index, 0);

        let coordinate = Coordinate::try_from("e4");
        assert_eq!(coordinate.unwrap().index, 36);

        let coordinate = Coordinate::try_from("h1");
        assert_eq!(coordinate.unwrap().index, 63);

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
}
