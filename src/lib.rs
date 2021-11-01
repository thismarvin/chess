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

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    utils::set_panic_hook();

    alert("Hello, chess!");
}
