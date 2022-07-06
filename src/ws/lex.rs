// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use std::iter::FusedIterator;

use arrayvec::ArrayVec;
use bstr::decode_utf8;

use crate::ws::token::{Mapping, Token};

pub trait Lexer: Iterator<Item = Result<Token, LexError>> {}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LexError {
    InvalidUtf8(ArrayVec<u8, 3>),
}

#[derive(Clone, Debug)]
pub struct Utf8Lexer {
    src: Vec<u8>,
    offset: usize,
    map: Mapping<char>,
}

impl Utf8Lexer {
    #[inline]
    pub const fn new(src: Vec<u8>, map: Mapping<char>) -> Self {
        Utf8Lexer { src, offset: 0, map }
    }
}

impl Lexer for Utf8Lexer {}

impl Iterator for Utf8Lexer {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.offset < self.src.len() {
            let offset = self.offset;
            let (ch, size) = decode_utf8(&self.src[offset..]);
            self.offset += size;
            match ch {
                Some(ch) => {
                    if let Some(tok) = self.map.map(ch) {
                        return Some(Ok(tok));
                    }
                }
                None => {
                    // Size is guaranteed to be between 1 and 3, inclusive, for
                    // an unsuccessful decode.
                    let mut bad = ArrayVec::new();
                    bad.try_extend_from_slice(&self.src[offset..offset + size])
                        .unwrap();
                    return Some(Err(LexError::InvalidUtf8(bad)));
                }
            }
        }
        None
    }
}

impl const FusedIterator for Utf8Lexer {}

#[derive(Clone, Debug)]
pub struct ByteLexer {
    src: Vec<u8>,
    offset: usize,
    map: Mapping<u8>,
}

impl ByteLexer {
    #[inline]
    pub const fn new(src: Vec<u8>, map: Mapping<u8>) -> Self {
        ByteLexer { src, offset: 0, map }
    }
}

impl Lexer for ByteLexer {}

impl Iterator for ByteLexer {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.offset < self.src.len() {
            let b = self.src[self.offset];
            self.offset += 1;
            if let Some(tok) = self.map.map(b) {
                return Some(Ok(tok));
            }
        }
        None
    }
}

impl const FusedIterator for ByteLexer {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws::tests::{TUTORIAL_STL, TUTORIAL_TOKENS};

    #[test]
    fn lex_tutorial() -> Result<(), LexError> {
        let lex = Utf8Lexer::new(TUTORIAL_STL.as_bytes().to_owned(), Mapping::<char>::STL);
        let toks = lex.collect::<Result<Vec<_>, LexError>>()?;
        assert_eq!(TUTORIAL_TOKENS, toks);
        Ok(())
    }
}
