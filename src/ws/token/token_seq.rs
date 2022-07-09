// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use crate::ws::token::{Token, TokenVec};

// Maximum TokenSeq value for each integer width:
// - u8  [T T L L L]
// - u16 [T L T T T S T L S L]
// - u32 [L S T L S L T T S L S T T S S S S S L L]
// - u64 [L S T S S T L L S L L S L S S L S L L L S S L L T S S T T S T T L T S T T S S S S]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenSeq(pub u32);

impl TokenSeq {
    #[inline]
    pub const fn new() -> Self {
        TokenSeq(0)
    }

    #[inline]
    pub const fn push(&mut self, tok: Token) {
        self.0 = self.0 * 3 + tok as u32 + 1;
    }

    #[inline]
    pub const fn pop(&mut self) -> Token {
        let tok = unsafe { Token::from_unchecked(((self.0 - 1) % 3) as u8) };
        self.0 = (self.0 - 1) / 3;
        tok
    }

    #[allow(dead_code)]
    #[inline]
    pub const fn len(&self) -> u32 {
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

impl const From<usize> for TokenSeq {
    #[inline]
    fn from(seq: usize) -> Self {
        TokenSeq(seq as u32)
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
    use super::*;
    use crate::ws::token::token_vec;

    #[test]
    fn convert_token_seq() {
        macro_rules! token_vecs(
            ($([$($seq:tt)*]),+$(,)?) => { vec![$(token_vec![$($seq)*]),+] }
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
            let seq = TokenSeq::from(i);
            let seq2 = TokenSeq::from(toks);
            assert_eq!(seq, seq2, "TokenSeq::from({:?})", toks);
            let toks2 = TokenVec::from(seq);
            assert_eq!(toks, toks2, "TokenVec::from({:?})", seq);
        }
    }
}
