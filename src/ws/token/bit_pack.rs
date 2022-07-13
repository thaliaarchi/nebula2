// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

//! Routines to pack and unpack tokens using a compact bitwise encoding.
//!
//! Tokens are encoded with a variable number of bits: S maps to 0, T maps to
//! 10, and L maps to 11. To resolve the ambiguity in the last byte of whether
//! the trailing zeros are S, the second half of T, or unset, an extra 1 is
//! appended, which is ignored when unpacking.
//!
//! This continues my tradition of including bit packing in each of my major
//! Whitespace implementations, following
//! [Respace](https://github.com/andrewarchi/respace/blob/master/src/binary.h),
//! [Nebula](https://github.com/andrewarchi/nebula/blob/master/ws/pack.go), and
//! [yspace](https://github.com/andrewarchi/yspace/blob/main/src/bit_pack.rs).
//! Unlike the others, which were `Msb0`, this has configurable bit order. As
//! far as I can tell, Respace was the first implementation of this algorithm
//! and had been discovered independently, though it had been mentioned
//! theoretically [in 2012](https://github.com/wspace/corpus/tree/main/python/res-trans32).

use bitvec::{
    order::{BitOrder, Lsb0, Msb0},
    slice::BitSlice,
    store::BitStore,
    vec::BitVec,
};
use strum::{Display, EnumString};

use crate::ws::token::Token;

/// Packs tokens into a compact bitwise representation. The length of the
/// returned BitVec may not be at a byte boundary.
pub fn bit_pack<T: BitStore, O: BitOrder>(toks: &[Token]) -> BitVec<T, O> {
    // TODO: Survey programs to find better size ratio estimate.
    let mut bits = BitVec::with_capacity(toks.len() * 2);
    for &tok in toks {
        match tok {
            Token::S => bits.push(false),
            Token::T => {
                bits.push(true);
                bits.push(false);
            }
            Token::L => {
                bits.push(true);
                bits.push(true);
            }
        }
    }
    bits
}

/// Unpacks tokens from a compact bitwise representation. If the last bit is an
/// unpaired 1 bit, it is ignored.
pub fn bit_unpack<T: BitStore, O: BitOrder>(bits: &BitSlice<T, O>) -> Vec<Token> {
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

/// Packs tokens into a compact bitwise representation. If the last token is an
/// S or T, a marker 1 bit is appended to avoid ambiguity between a zero in the
/// encodings of S and T and unset trailing zeros.
pub fn bit_pack_aligned<T: BitStore, O: BitOrder>(toks: &[Token]) -> Vec<T> {
    let mut bits = bit_pack::<T, O>(toks);
    // In the last byte, any trailing zeros would be treated as S, so to avoid
    // this, a marker 1 bit is appended.
    if bits.last().as_deref() == Some(&false) {
        bits.push(true);
    }
    bits.set_uninitialized(false);
    bits.into_vec()
}

/// Unpacks tokens from a compact bitwise representation.
pub fn bit_unpack_aligned<T: BitStore, O: BitOrder>(bits: &[T]) -> Vec<Token> {
    let mut bits = BitSlice::<T, O>::from_slice(bits);
    // Trim trailing zeros in the last byte.
    let tz = bits.trailing_zeros();
    if 0 < tz && tz <= 8 {
        bits = &bits[..bits.len() - tz];
    }
    bit_unpack(bits)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(EnumString, Display)]
pub enum BitOrderDynamic {
    #[strum(to_string = "lsb0", serialize = "lsf", serialize = "le")]
    Lsb0,
    #[strum(to_string = "msb0", serialize = "msf", serialize = "me")]
    Msb0,
}

/// Packs tokens into a compact bitwise representation with a dynamically-set
/// bit order. This is intended for cases where the bit order is configurable at
/// runtime.
pub fn bit_pack_dynamic(toks: &[Token], order: BitOrderDynamic) -> Vec<u8> {
    match order {
        BitOrderDynamic::Lsb0 => bit_pack_aligned::<u8, Lsb0>(toks),
        BitOrderDynamic::Msb0 => bit_pack_aligned::<u8, Msb0>(toks),
    }
}

/// Unpacks tokens from a compact bitwise representation with a dynamically-set
/// bit order. This is intended for cases where the bit order is configurable at
/// runtime.
pub fn bit_unpack_dynamic(bits: &[u8], order: BitOrderDynamic) -> Vec<Token> {
    match order {
        BitOrderDynamic::Lsb0 => bit_unpack_aligned::<u8, Lsb0>(bits),
        BitOrderDynamic::Msb0 => bit_unpack_aligned::<u8, Msb0>(bits),
    }
}
