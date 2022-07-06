// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use std::mem;

use crate::ws::token_vec::TokenVec;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    S = 0,
    T = 1,
    L = 2,
}

impl Token {
    #[inline]
    pub const unsafe fn from_unchecked(n: u8) -> Self {
        mem::transmute(n)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Mapping<T> {
    pub s: T,
    pub t: T,
    pub l: T,
}

impl<T: Copy + Eq> Mapping<T> {
    #[inline]
    pub const fn new(s: T, t: T, l: T) -> Self {
        Mapping { s, t, l }
    }

    #[inline]
    pub fn map(&self, v: T) -> Option<Token> {
        match v {
            _ if v == self.s => Some(Token::S),
            _ if v == self.t => Some(Token::T),
            _ if v == self.l => Some(Token::L),
            _ => None,
        }
    }

    #[inline]
    pub const fn map_token(&self, tok: Token) -> T {
        match tok {
            Token::S => self.s,
            Token::T => self.t,
            Token::L => self.l,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenSeq(pub u16);

impl TokenSeq {
    #[inline]
    pub const fn new() -> Self {
        TokenSeq(0)
    }

    #[inline]
    pub const fn push(&mut self, tok: Token) {
        self.0 = self.0 * 3 + tok as u16 + 1;
    }

    #[inline]
    pub const fn pop(&mut self) -> Token {
        let tok = unsafe { Token::from_unchecked(((self.0 - 1) % 3) as u8) };
        self.0 = (self.0 - 1) / 3;
        tok
    }

    #[inline]
    pub const fn len(&self) -> u16 {
        let mut seq = self.0;
        let mut len = 0;
        while seq != 0 {
            seq = (seq - 1) / 3;
            len += 1;
        }
        len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl const From<TokenVec> for TokenSeq {
    #[inline]
    fn from(toks: TokenVec) -> Self {
        let mut seq = TokenSeq::new();
        for tok in toks {
            seq.push(tok);
        }
        seq
    }
}

#[cfg(test)]
mod tests {
    use super::Token::*;
    use super::*;
    use crate::ws::token_vec::token_vec;

    #[test]
    fn convert_token_seq() {
        macro_rules! token_vecs(
            ($([$($seq:expr)*]),+$(,)?) => { vec![$(token_vec![$($seq)*]),+] }
        );
        let seqs: Vec<TokenVec> = token_vecs![
            [],
            [S], [T], [L],
            [S S], [S T], [S L],
            [T S], [T T], [T L],
            [L S], [L T], [L L],
            [S S S], [S S T], [S S L], [S T S], [S T T], [S T L], [S L S], [S L T], [S L L],
            [T S S], [T S T], [T S L], [T T S], [T T T], [T T L], [T L S], [T L T], [T L L],
            [L S S], [L S T], [L S L], [L T S], [L T T], [L T L], [L L S], [L L T], [L L L],
        ];
        for (i, &toks) in seqs.iter().enumerate() {
            let seq = TokenSeq(i as u16);
            let seq2 = TokenSeq::from(toks);
            assert_eq!(seq, seq2, "TokenSeq::from({:?})", toks);
            let toks2 = TokenVec::from(seq);
            assert_eq!(toks, toks2, "TokenVec::from({:?})", seq);
        }
    }
}
