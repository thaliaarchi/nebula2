// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::iter::FusedIterator;

use crate::text::{ByteIterator, EncodingError, Utf8Iterator};
use crate::ws::token::{Mapping, Token};

pub trait Lexer = Iterator<Item = Result<Token, EncodingError>>;

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
    T: Copy + Eq,
{
    type Item = Result<Token, EncodingError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(Ok(v)) => {
                    if let Some(tok) = self.map.map(v) {
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
    T: Copy + Eq,
{
}
