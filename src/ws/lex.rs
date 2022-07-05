// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use std::iter::FusedIterator;

use bstr::decode_utf8;

use crate::ws::token::{CharMapping, Token};

#[derive(Clone, Debug)]
pub struct Lexer {
    src: Vec<u8>,
    offset: usize,
    map: CharMapping,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LexError {
    Utf8Error,
}

impl Lexer {
    #[inline]
    pub const fn new(src: Vec<u8>, map: CharMapping) -> Self {
        Lexer { src, offset: 0, map }
    }
}

impl Iterator for Lexer {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.offset < self.src.len() {
            let (ch, size) = decode_utf8(&self.src[self.offset..]);
            self.offset += size;
            match ch {
                Some(ch) => {
                    if let Some(tok) = self.map.map_char(ch) {
                        return Some(Ok(tok));
                    }
                }
                None => return Some(Err(LexError::Utf8Error)),
            }
        }
        None
    }
}

impl const FusedIterator for Lexer {}
