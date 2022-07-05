// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use std::mem::transmute;

use self::Token::*;
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
        transmute(n)
    }
}

#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CharMapping {
    pub S: char,
    pub T: char,
    pub L: char,
}

impl CharMapping {
    pub const STL: CharMapping = CharMapping::new('S', 'T', 'L');

    #[inline]
    pub const fn new(s: char, t: char, l: char) -> Self {
        CharMapping { S: s, T: t, L: l }
    }

    #[inline]
    pub const fn map_char(&self, ch: char) -> Option<Token> {
        match ch {
            _ if ch == self.S => Some(S),
            _ if ch == self.T => Some(T),
            _ if ch == self.L => Some(L),
            _ => None,
        }
    }

    #[inline]
    pub const fn map_token(&self, tok: Token) -> char {
        match tok {
            S => self.S,
            T => self.T,
            L => self.L,
        }
    }
}

impl const Default for CharMapping {
    #[inline]
    fn default() -> Self {
        CharMapping::new(' ', '\t', '\n')
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
    pub const fn empty(&self) -> bool {
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
mod test {
    use super::*;
    use crate::ws::token_vec::token_vec;

    #[test]
    fn test_token_seq_convert() {
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
