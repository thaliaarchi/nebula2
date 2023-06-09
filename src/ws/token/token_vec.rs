// Copyright (C) 2022 Thalia Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use bitvec::prelude::*;

use crate::syntax::TokenSeq;
use crate::ws::token::Token;

pub trait TokenVec {
    #[must_use]
    fn from_bits<T: BitStore, O: BitOrder>(bits: &BitSlice<T, O>) -> Self;
    fn append_bits<T: BitStore, O: BitOrder>(&mut self, bits: &BitSlice<T, O>);
}

impl TokenVec for Vec<Token> {
    #[inline]
    fn from_bits<T: BitStore, O: BitOrder>(bits: &BitSlice<T, O>) -> Self {
        let mut toks = Vec::new();
        toks.append_bits(bits);
        toks
    }

    #[inline]
    fn append_bits<T: BitStore, O: BitOrder>(&mut self, bits: &BitSlice<T, O>) {
        self.reserve(bits.len());
        for bit in bits {
            self.push(if *bit { Token::T } else { Token::S });
        }
    }
}

impl From<Vec<Token>> for TokenSeq<Token> {
    #[inline]
    fn from(toks: Vec<Token>) -> Self {
        let mut seq = TokenSeq::new();
        for tok in toks {
            seq.push(&tok);
        }
        seq
    }
}
