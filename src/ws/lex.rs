// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::iter::FusedIterator;

use arrayvec::ArrayVec;
use bstr::decode_utf8;

use crate::ws::token::{Mapping, Token};

pub trait Lexer = Iterator<Item = Result<Token, LexError>>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LexError {
    InvalidUtf8(ArrayVec<u8, 3>),
}

#[derive(Clone, Debug)]
pub struct Utf8Lexer<'a> {
    src: &'a [u8],
    offset: usize,
    map: Mapping<char>,
    error_once: bool,
    valid_to: Option<usize>,
}

impl<'a> Utf8Lexer<'a> {
    #[inline]
    pub const fn new<B>(src: &'a B, map: Mapping<char>, error_once: bool) -> Self
    where
        B: ~const AsRef<[u8]> + ?Sized,
    {
        Utf8Lexer {
            src: src.as_ref(),
            offset: 0,
            map,
            error_once,
            valid_to: None,
        }
    }

    #[inline]
    pub const fn valid_to(&self) -> usize {
        match self.valid_to {
            Some(valid_to) => valid_to,
            None => self.offset,
        }
    }
}

impl<'a> Iterator for Utf8Lexer<'a> {
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
                None if self.valid_to == None || !self.error_once => {
                    self.valid_to = Some(offset);
                    // Size is guaranteed to be between 1 and 3, inclusive, for
                    // an unsuccessful decode.
                    let mut bad = ArrayVec::new();
                    bad.try_extend_from_slice(&self.src[offset..offset + size])
                        .unwrap();
                    return Some(Err(LexError::InvalidUtf8(bad)));
                }
                None => {}
            }
        }
        None
    }
}

impl<'a> const FusedIterator for Utf8Lexer<'a> {}

#[derive(Clone, Debug)]
pub struct ByteLexer<'a> {
    src: &'a [u8],
    offset: usize,
    map: Mapping<u8>,
}

impl<'a> ByteLexer<'a> {
    #[inline]
    pub const fn new<B: ~const AsRef<[u8]> + ?Sized>(src: &'a B, map: Mapping<u8>) -> Self {
        ByteLexer {
            src: src.as_ref(),
            offset: 0,
            map,
        }
    }
}

impl<'a> Iterator for ByteLexer<'a> {
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

impl<'a> const FusedIterator for ByteLexer<'a> {}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use static_assertions::const_assert;

    use super::*;

    const_assert!(size_of::<ArrayVec<u8, 3>>() < size_of::<Vec<u8>>());
}
