mod utils;

use bitflags::bitflags;
use wasm_bindgen::prelude::*;

const BOARD_WIDTH: usize = 8;
const BOARD_HEIGHT: usize = 8;

#[derive(Debug, PartialEq, Eq)]
enum Color {
    White,
    Black,
}

impl TryFrom<&str> for Color {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "w" => Ok(Color::White),
            "b" => Ok(Color::Black),
            _ => Err(()),
        }
    }
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

impl TryFrom<char> for CastlingAbility {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'K' => Ok(CastlingAbility::WHITE_KING_SIDE),
            'Q' => Ok(CastlingAbility::WHITE_QUEEN_SIDE),
            'k' => Ok(CastlingAbility::BLACK_KING_SIDE),
            'q' => Ok(CastlingAbility::BLACK_QUEEN_SIDE),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for CastlingAbility {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut ability: Option<CastlingAbility> = None;

        for character in value.chars() {
            if let Ok(value) = CastlingAbility::try_from(character) {
                ability = if let Some(ability) = ability {
                    Some(ability | value)
                } else {
                    Some(value)
                };
            } else {
                return Err(());
            }
        }

        match ability {
            Some(ability) => Ok(ability),
            None => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Coordinate(usize);

impl Coordinate {

    fn x(&self) -> usize {
        self.0 % BOARD_WIDTH
    }

    fn y(&self) -> usize {
        self.0 / BOARD_WIDTH
    }
}

impl TryFrom<usize> for Coordinate {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value >= BOARD_WIDTH * BOARD_HEIGHT {
            return Err(());
        }

        Ok(Coordinate(value))
    }
}

impl TryFrom<(usize, usize)> for Coordinate {
    type Error = ();

    fn try_from(value: (usize, usize)) -> Result<Self, Self::Error> {
        if value.0 >= BOARD_WIDTH || value.1 >= BOARD_HEIGHT {
            return Err(());
        }

        Coordinate::try_from(value.1 * BOARD_WIDTH + value.0)
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

#[derive(Debug, PartialEq, Eq)]
struct Placement<'a>(&'a str);

impl TryFrom<&'static str> for Placement<'static> {
    type Error = ();

    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        let ranks: Vec<&str> = value.split("/").collect();

        if ranks.len() != BOARD_HEIGHT {
            return Err(());
        }

        for rank in ranks {
            let characters = rank.chars();
            let mut reach = 0 as usize;

            for character in characters {
                if let Some(digit) = character.to_digit(10) {
                    reach += digit as usize;
                    continue;
                }

                if let Ok(_) = Piece::try_from(character) {
                    reach += 1;
                    continue;
                }

                return Err(());
            }

            if reach != BOARD_WIDTH {
                return Err(());
            }
        }

        Ok(Placement(value))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct FEN<'a> {
    placement: Placement<'a>,
    side_to_move: Color,
    castling_ability: Option<CastlingAbility>,
    en_passant_target: Option<Coordinate>,
    half_moves: usize,
    full_moves: usize,
}

// TODO(thismarvin): Am I using the right lifetime?
impl TryFrom<&'static str> for FEN<'static> {
    type Error = ();

    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        let mut sections: Vec<&str> = value.split_whitespace().collect();

        if sections.len() != 6 {
            return Err(());
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

        let mut half_moves = sections[4];
        let half_moves: usize = half_moves.parse().map_err(|_| ())?;

        let mut full_moves = sections[5];
        let full_moves: usize = full_moves.parse().map_err(|_| ())?;

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
    pieces: [Option<Piece>; BOARD_WIDTH * BOARD_HEIGHT],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_from_xy() {
        let coordinate = Coordinate::try_from((4, 4));
        assert_eq!(coordinate, Ok(Coordinate(36)));

        let coordinate = Coordinate::try_from((8, 1));
        assert!(coordinate.is_err());

        let coordinate = Coordinate::try_from((1, 8));
        assert!(coordinate.is_err());

        let coordinate = Coordinate::try_from((7, 3)).unwrap();
        assert_eq!(coordinate.x(), 7);
        assert_eq!(coordinate.y(), 3);
    }

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

    #[test]
    fn test_placement_from_str() {
        let placement = Placement::try_from("what is this really called?");
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
    fn test_fen_from_str() {
        let fen = FEN::try_from("what is a fen string for?");
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
                placement: Placement("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR"),
                side_to_move: Color::White,
                castling_ability: Some(
                    CastlingAbility::WHITE_KING_SIDE
                        | CastlingAbility::WHITE_QUEEN_SIDE
                        | CastlingAbility::BLACK_KING_SIDE
                        | CastlingAbility::BLACK_QUEEN_SIDE
                ),
                en_passant_target: None,
                half_moves: 0,
                full_moves: 1
            })
        );

        let fen = FEN::try_from("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
        assert_eq!(
            fen,
            Ok(FEN {
                placement: Placement("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR"),
                side_to_move: Color::Black,
                castling_ability: Some(
                    CastlingAbility::WHITE_KING_SIDE
                        | CastlingAbility::WHITE_QUEEN_SIDE
                        | CastlingAbility::BLACK_KING_SIDE
                        | CastlingAbility::BLACK_QUEEN_SIDE
                ),
                en_passant_target: Some(Coordinate::try_from("e3").unwrap()),
                half_moves: 0,
                full_moves: 1
            })
        );

        let fen =
            FEN::try_from("r2qkbnr/pp1n1ppp/2p1p3/3pPb2/3P4/5N2/PPP1BPPP/RNBQ1RK1 b kq - 3 6 ");
        assert_eq!(
            fen,
            Ok(FEN {
                placement: Placement("r2qkbnr/pp1n1ppp/2p1p3/3pPb2/3P4/5N2/PPP1BPPP/RNBQ1RK1"),
                side_to_move: Color::Black,
                castling_ability: Some(
                    CastlingAbility::BLACK_KING_SIDE | CastlingAbility::BLACK_QUEEN_SIDE
                ),
                en_passant_target: None,
                half_moves: 3,
                full_moves: 6
            })
        );

        let fen =
            FEN::try_from("r4rk1/2qn1pb1/1p2p1np/3pPb2/8/1N1N2B1/PPP1B1PP/R2Q1RK1 w - - 3 17");
        assert_eq!(
            fen,
            Ok(FEN {
                placement: Placement("r4rk1/2qn1pb1/1p2p1np/3pPb2/8/1N1N2B1/PPP1B1PP/R2Q1RK1"),
                side_to_move: Color::White,
                castling_ability: None,
                en_passant_target: None,
                half_moves: 3,
                full_moves: 17
            })
        );
    }
}
