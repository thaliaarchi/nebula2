// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use std::iter::FusedIterator;

use crate::ws::lex::{LexError, Lexer};
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

    fn next_bit(&mut self) -> Option<bool> {
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

impl Lexer for BitLexer {}

impl Iterator for BitLexer {
    type Item = Result<Token, LexError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.next_bit() {
            Some(true) => match self.next_bit() {
                Some(true) => Some(Ok(L)),
                Some(false) => Some(Ok(T)),
                None => None, // Marker bit
            },
            Some(false) => Some(Ok(S)),
            None => None,
        }
    }
}

impl const FusedIterator for BitLexer {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws::tests::{TUTORIAL_BITS, TUTORIAL_TOKENS};

    #[test]
    fn bit_lex_tutorial() -> Result<(), LexError> {
        let lex = BitLexer::new(TUTORIAL_BITS.to_owned());
        let toks = lex.collect::<Result<Vec<_>, LexError>>()?;
        assert_eq!(TUTORIAL_TOKENS, toks);
        Ok(())
    }
}
