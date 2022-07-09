// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::iter::FusedIterator;

use crate::text::{ByteIterator, EncodingError, Utf8Iterator};
use crate::ws::token::Token;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Mapping<T> {
    pub s: T,
    pub t: T,
    pub l: T,
}

impl<T: Eq> Mapping<T> {
    #[inline]
    pub const fn new(s: T, t: T, l: T) -> Self {
        Mapping { s, t, l }
    }

    #[inline]
    pub fn map(&self, v: &T) -> Option<Token> {
        match v {
            _ if v == &self.s => Some(Token::S),
            _ if v == &self.t => Some(Token::T),
            _ if v == &self.l => Some(Token::L),
            _ => None,
        }
    }

    #[inline]
    pub const fn map_token(&self, tok: Token) -> &T {
        match tok {
            Token::S => &self.s,
            Token::T => &self.t,
            Token::L => &self.l,
        }
    }
}

impl Mapping<char> {
    pub const STL: Self = Mapping::new('S', 'T', 'L');
}

impl Mapping<u8> {
    pub const STL: Self = Mapping::new(b'S', b'T', b'L');
}

impl const Default for Mapping<char> {
    #[inline]
    fn default() -> Self {
        Mapping::new(' ', '\t', '\n')
    }
}

impl const Default for Mapping<u8> {
    #[inline]
    fn default() -> Self {
        Mapping::new(b' ', b'\t', b'\n')
    }
}

#[derive(Clone, Debug)]
pub struct MappingLexer<I, T> {
    iter: I,
    map: Mapping<T>,
}

impl<I, T> MappingLexer<I, T> {
    #[inline]
    pub const fn new(iter: I, map: Mapping<T>) -> Self {
        MappingLexer { iter, map }
    }
}

impl<'a> MappingLexer<Utf8Iterator<'a>, char> {
    #[inline]
    pub const fn new_utf8<B>(src: &'a B, map: Mapping<char>, error_once: bool) -> Self
    where
        B: ~const AsRef<[u8]> + ?Sized,
    {
        Self::new(Utf8Iterator::new(src, error_once), map)
    }
}

impl<'a> MappingLexer<ByteIterator<'a>, u8> {
    #[inline]
    pub const fn new_bytes<B>(src: &'a B, map: Mapping<u8>) -> Self
    where
        B: ~const AsRef<[u8]> + ?Sized,
    {
        Self::new(ByteIterator::new(src), map)
    }
}

impl<I, T> Iterator for MappingLexer<I, T>
where
    I: Iterator<Item = Result<T, EncodingError>>,
    T: Eq,
{
    type Item = Result<Token, EncodingError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(Ok(v)) => {
                    if let Some(tok) = self.map.map(&v) {
                        return Some(Ok(tok));
                    }
                }
                Some(Err(err)) => return Some(Err(err)),
                None => return None,
            }
        }
    }
}

impl<I, T> const FusedIterator for MappingLexer<I, T>
where
    I: Iterator<Item = Result<T, EncodingError>> + FusedIterator,
    T: Eq,
{
}
