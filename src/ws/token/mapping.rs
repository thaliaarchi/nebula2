// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::iter::FusedIterator;

use crate::text::{ByteIterator, EncodingError, Utf8Iterator};
use crate::ws::token::{Lexer, Token};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Mapping<T> {
    s: T,
    t: T,
    l: T,
}

impl<T: Eq> Mapping<T> {
    #[inline]
    pub fn new(s: T, t: T, l: T) -> Option<Self> {
        if s == t || s == l || t == l {
            return None;
        }
        Some(Mapping { s, t, l })
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
    pub const STL: Self = Mapping { s: 'S', t: 'T', l: 'L' };
}

impl Mapping<u8> {
    pub const STL: Self = Mapping { s: b'S', t: b'T', l: b'L' };
}

impl const Default for Mapping<char> {
    #[inline]
    fn default() -> Self {
        Mapping { s: ' ', t: '\t', l: '\n' }
    }
}

impl const Default for Mapping<u8> {
    #[inline]
    fn default() -> Self {
        Mapping { s: b' ', t: b'\t', l: b'\n' }
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BytesMapping {
    s: Vec<u8>,
    t: Vec<u8>,
    l: Vec<u8>,
}

impl BytesMapping {
    pub fn new(s: Vec<u8>, t: Vec<u8>, l: Vec<u8>) -> Option<Self> {
        if Self::is_prefix(&s, &t) || Self::is_prefix(&s, &l) || Self::is_prefix(&t, &l) {
            return None;
        }
        Some(BytesMapping { s, t, l })
    }

    #[inline]
    fn is_prefix(s1: &[u8], s2: &[u8]) -> bool {
        s1.iter().zip(s2.iter()).all(|(b1, b2)| b1 == b2)
    }

    #[inline]
    pub fn map(&self, v: &[u8]) -> Option<(Token, usize)> {
        match v {
            _ if v.starts_with(&self.s) => Some((Token::S, self.s.len())),
            _ if v.starts_with(&self.t) => Some((Token::T, self.t.len())),
            _ if v.starts_with(&self.l) => Some((Token::L, self.l.len())),
            _ => None,
        }
    }

    #[inline]
    pub fn map_token(&self, tok: Token) -> &[u8] {
        match tok {
            Token::S => &self.s,
            Token::T => &self.t,
            Token::L => &self.l,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BytesMappingLexer<'a> {
    src: &'a [u8],
    offset: usize,
    map: BytesMapping,
}

impl<'a> BytesMappingLexer<'a> {
    #[inline]
    pub const fn new<B>(src: &'a B, map: BytesMapping) -> Self
    where
        B: ~const AsRef<[u8]> + ?Sized,
    {
        BytesMappingLexer {
            src: src.as_ref(),
            offset: 0,
            map,
        }
    }
}

impl<'a> Iterator for BytesMappingLexer<'a> {
    type Item = Result<Token, EncodingError>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.offset < self.src.len() {
            if let Some((tok, size)) = self.map.map(&self.src[self.offset..]) {
                self.offset += size;
                return Some(Ok(tok));
            }
            self.offset += 1;
        }
        None
    }
}

impl<'a> const FusedIterator for BytesMappingLexer<'a> {}

pub fn lex_mapping<'a>(
    src: &'a [u8],
    s: Vec<u8>,
    t: Vec<u8>,
    l: Vec<u8>,
    is_utf8: bool,
    error_once: bool,
) -> Option<Box<dyn Lexer + 'a>> {
    #[inline]
    fn decode_one_char(b: &[u8]) -> Option<char> {
        match bstr::decode_utf8(b) {
            (Some(ch), size) if size == b.len() => Some(ch),
            _ => None,
        }
    }

    if is_utf8 {
        if let Some(s_ch) = decode_one_char(&s)
            && let Some(t_ch) = decode_one_char(&t)
            && let Some(l_ch) = decode_one_char(&l)
        {
            let map = Mapping::new(s_ch, t_ch, l_ch)?;
            let iter = Utf8Iterator::new(src, error_once);
            return Some(box MappingLexer::new(iter, map));
        }
        // TODO: Handle invalid UTF-8 in BytesMappingLexer case
    } else if s.len() == 1 && t.len() == 1 && l.len() == 1 {
        let map = Mapping::new(s[0], t[0], l[0])?;
        let iter = ByteIterator::new(src);
        return Some(box MappingLexer::new(iter, map));
    }
    let map = BytesMapping::new(s, t, l)?;
    Some(box BytesMappingLexer::new(src, map))
}
