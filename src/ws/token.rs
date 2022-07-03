// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use self::Token::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    S = 0,
    T = 1,
    L = 2,
}

impl Token {
    #[inline]
    const fn from_u16(n: u16) -> Self {
        match n {
            0 => S,
            1 => T,
            2 => L,
            _ => unreachable!(),
        }
    }
}

#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TokenMapping {
    pub S: char,
    pub T: char,
    pub L: char,
}

impl TokenMapping {
    pub const STL: TokenMapping = TokenMapping::new('S', 'T', 'L');

    #[inline]
    pub const fn new(s: char, t: char, l: char) -> Self {
        TokenMapping { S: s, T: t, L: l }
    }

    #[inline]
    pub const fn map(&self, c: char) -> Option<Token> {
        match c {
            _ if c == self.S => Some(S),
            _ if c == self.T => Some(T),
            _ if c == self.L => Some(L),
            _ => None,
        }
    }
}

impl const Default for TokenMapping {
    #[inline]
    fn default() -> Self {
        TokenMapping::new(' ', '\t', '\n')
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
    pub const fn push(&self, tok: Token) -> TokenSeq {
        TokenSeq(self.0 * 3 + tok as u16 + 1)
    }

    #[inline]
    pub const fn pop(&self) -> (TokenSeq, Token) {
        (
            TokenSeq((self.0 - 1) / 3),
            Token::from_u16((self.0 - 1) % 3),
        )
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
    pub const fn from_tokens(toks: &[Token]) -> TokenSeq {
        let mut seq = TokenSeq::new();
        let mut i = 0;
        while i < toks.len() {
            seq = seq.push(toks[i]);
            i += 1;
        }
        seq
    }

    // TODO: make an inline, fixed-capacity container type with const methods
    #[inline]
    pub fn to_tokens(&self) -> Vec<Token> {
        let mut seq = *self;
        let len = seq.len() as usize;
        let mut toks = vec![S; len];
        for i in (0..len).rev() {
            (seq, toks[i]) = seq.pop();
        }
        toks
    }
}

#[cfg(test)]
mod test {
    use super::{Token, Token::*, TokenSeq};

    macro_rules! seq_vec(
        ($([$($seq:expr)*]),+$(,)?) => { vec![$(&[$($seq),*]),+] }
    );

    #[test]
    fn test_token_seq_convert() {
        let seqs: Vec<&[Token]> = seq_vec![
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
            let seq2 = TokenSeq::from_tokens(toks);
            assert_eq!(seq, seq2, "TokenSeq::from_tokens({:?})", toks);
            let toks2 = seq.to_tokens();
            assert_eq!(toks, toks2, "{:?}.to_tokens()", seq);
        }
    }
}
