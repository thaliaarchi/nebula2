// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use crate::ws::token::Token::{self, *};

#[derive(Clone, Debug)]
pub struct BitLexer {
    src: Vec<u8>,
    byte_offset: usize,
    bit_offset: u8,
}

impl BitLexer {
    #[inline]
    pub const fn new(src: Vec<u8>) -> Self {
        BitLexer {
            src,
            byte_offset: 0,
            bit_offset: 7,
        }
    }

    pub fn next_bit(&mut self) -> Option<bool> {
        if self.byte_offset >= self.src.len() {
            return None;
        }
        let byte = self.src[self.byte_offset];
        // Ignore trailing zeros on the last byte
        if self.byte_offset + 1 == self.src.len() && byte << (7 - self.bit_offset) == 0 {
            return None;
        }
        let bit = byte & (1 << self.bit_offset) != 0;
        if self.bit_offset == 0 {
            self.bit_offset = 7;
            self.byte_offset += 1;
        } else {
            self.bit_offset -= 1;
        }
        Some(bit)
    }
}

impl Iterator for BitLexer {
    type Item = Token;

    #[inline]
    fn next(&mut self) -> Option<Token> {
        match self.next_bit() {
            Some(true) => match self.next_bit() {
                Some(true) => Some(L),
                Some(false) => Some(T),
                None => None, // Marker bit
            },
            Some(false) => Some(S),
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bit_lex_tutorial() {
        let src = vec![
            0b00010111, 0b10001000, 0b00101011, 0b01101011, 0b01000010, 0b01001110, 0b11000001,
            0b01110000, 0b01100001, 0b00101011, 0b10001011, 0b10001000, 0b01001011, 0b11011010,
            0b00001010, 0b11110001, 0b00001001, 0b01101111, 0b11111100,
        ];
        let toks = vec![
            S, S, S, T, L, L, S, S, S, T, S, S, S, S, T, T, L, S, L, S, T, L, S, T, S, S, S, T, S,
            T, S, L, T, L, S, S, S, S, S, T, L, T, S, S, S, S, L, S, S, S, S, T, S, T, T, L, T, S,
            S, T, L, T, S, S, T, S, S, S, T, S, T, L, L, S, L, S, T, S, S, S, S, T, T, L, L, S, S,
            S, T, S, S, S, T, S, T, L, S, L, L, L, L, L,
        ];

        let toks2 = BitLexer::new(src).collect::<Vec<_>>();
        assert_eq!(toks, toks2);
    }
}
