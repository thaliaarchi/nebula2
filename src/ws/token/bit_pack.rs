// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

//! `pack_bits`/`unpack_bits` is a continuation of my tradition of including bit
//! packing in each of my major Whitespace implementations, following
//! [Respace](https://github.com/andrewarchi/respace/blob/master/src/binary.h),
//! [Nebula](https://github.com/andrewarchi/nebula/blob/master/ws/pack.go), and
//! [yspace](https://github.com/andrewarchi/yspace/blob/main/src/bit_pack.rs).
//! Unlike the others, which were `Msb0`, this has configurable bit order. As
//! far as I can tell, Respace was the first implementation of this algorithm
//! and had been discovered independently, though it had been mentioned
//! theoretically [in 2012](https://github.com/wspace/corpus/tree/main/python/res-trans32).

use bitvec::{order::BitOrder, slice::BitSlice};

use crate::ws::token::Token;

pub fn unpack_bits<O: BitOrder>(packed: &[u8]) -> Vec<Token> {
    let mut bits = BitSlice::<_, O>::from_slice(packed);
    // In the last byte, any trailing zeros would be treated as S, so to avoid
    // this, a marker 1 bit is appended, when the bits do not fit at a byte
    // boundary.
    let tz = bits.trailing_zeros();
    if 0 < tz && tz <= 8 {
        bits = &bits[..bits.len() - tz];
    }
    // TODO: Survey programs to find better size ratio estimate.
    // TODO: Use TokenVec here, once it can extend its capacity.
    let mut toks = Vec::with_capacity(bits.len());

    let mut bits = bits.into_iter();
    loop {
        toks.push(match bits.next().as_deref() {
            Some(true) => match bits.next().as_deref() {
                Some(true) => Token::L,
                Some(false) => Token::T,
                None => break, // Marker bit
            },
            Some(false) => Token::S,
            None => break, // EOF
        });
    }
    toks
}
