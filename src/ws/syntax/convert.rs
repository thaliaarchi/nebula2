// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

//! Converts between integer and bit vector types.
//!
//! To convert between Rug and `bitvec` types, the bit order, endianness, and
//! limb size need to correspond:
//!
//! In the GMP `mpz_import` function, which Rug calls in `Integer::from_digits`,
//! it copies the input data to its internal format, which appears to be `Lsf`.
//! If the input order is `Lsf*` and the endianness matches the host, the data
//! is simply copied. If the endianness does not match the host, it swaps the
//! bytes. If the input order is `Msf*`, the bits are reversed.
//!
//! `bitvec` strongly recommends using `Lsb0` as the `BitOrder`, even if it
//! doesn't match the host endianness, because it provides the best codegen for
//! bit manipulation. Since there is no equivalent to `Lsf` in `bitvec` and
//! big-endian systems are rare, `LsfLe`/`Lsb0` is the best option.
//!
//! Whitespace integers are big endian, but are parsed and pushed to a `BitVec`
//! in little-endian order, so the slice of bits needs to be reversed (i.e., not
//! reversing or swapping words) before converting to an `Integer`.
//!
//! GMP uses a machine word as the limb size and `bitvec` uses `usize` as the
//! default `BitStore`.
//!
//! | Rug   | bitvec    | Bit order                   | Endianness      |
//! | ----- | --------- | --------------------------- | --------------- |
//! | Lsf   |           | least-significant bit first | host endianness |
//! | LsfLe | Lsb0      | least-significant bit first | little-endian   |
//! | LsfBe |           | least-significant bit first | big-endian      |
//! | Msf   |           | most-significant bit first  | host endianness |
//! | MsfLe |           | most-significant bit first  | little-endian   |
//! | MsfBe | Msb0      | most-significant bit first  | big-endian      |
//! |       | LocalBits | alias to Lsb0 or Msb0       | host endianness |

use std::cmp::Ordering;

use bitvec::prelude::*;
use gmp_mpfr_sys::gmp;
use rug::{integer::Order, ops::NegAssign, Integer};

use crate::ws::syntax::Sign;

/// Constructs an Integer from a Vec of digits, where each digit is in the range
/// 0..radix, i.e., not ASCII characters.
///
/// Adapted from Rug internal functions:
/// `<rug::integer::ParseIncomplete as rug::Assign>::assign`
/// and `rug::ext::xmpz::realloc_for_mpn_set_str`.
///
/// Compensates for Rug missing a higher-level API for using `mpn_set_str`:
/// https://gitlab.com/tspiteri/rug/-/issues/41
pub fn integer_from_digits_radix(digits: Vec<u8>, sign: Sign, radix: u32) -> Integer {
    if digits.len() == 0 {
        return Integer::ZERO;
    }
    let mut int = Integer::new();
    let raw = int.as_raw_mut();

    // Add 1 to make the floored integer log be ceiling
    let bits = (radix.log2() as usize + 1) * digits.len();
    // Use integer ceiling division
    let limb_bits = gmp::LIMB_BITS as usize;
    let limbs = (bits + limb_bits - 1) / limb_bits;
    unsafe {
        // Add 1, because `mpn_set_str` requires an extra limb
        gmp::_mpz_realloc(raw, limbs as gmp::size_t + 1);

        let size = gmp::mpn_set_str(
            (*raw).d.as_ptr(),
            digits.as_ptr(),
            digits.len(),
            radix as i32,
        );
        (*raw).size = if sign == Sign::Neg { -size } else { size } as i32;
    }
    int
}

pub fn integer_from_signed_bits(bits: &BitSlice) -> Integer {
    match bits.split_first() {
        None => Integer::ZERO,
        Some((sign, bits)) => {
            let mut int = integer_from_unsigned_bits(bits);
            if *sign == true {
                int.neg_assign();
            }
            int
        }
    }
}

pub fn integer_from_unsigned_bits(bits: &BitSlice) -> Integer {
    let len = bits.len();
    if len < usize::BITS as usize * 4 {
        let mut arr = BitArray::<_, Lsb0>::new([0usize; 4]);
        let slice = &mut arr[..len];
        slice.copy_from_bitslice(bits);
        slice.reverse();
        Integer::from_digits(arr.as_raw_slice(), Order::LsfLe)
    } else {
        let mut boxed = BitBox::<usize, Lsb0>::from(bits);
        boxed.force_align();
        boxed.fill_uninitialized(false);
        boxed.reverse();
        Integer::from_digits(boxed.as_raw_slice(), Order::LsfLe)
    }
}

#[inline]
pub fn integer_from_unsigned_bits_unambiguous(bits: &BitSlice) -> Option<Integer> {
    if bits.first().as_deref() == Some(&true) {
        Some(integer_from_unsigned_bits(bits))
    } else {
        None
    }
}

pub fn unsigned_bits_from_integer(int: &Integer) -> BitVec {
    let mut bits = BitVec::<usize, Lsb0>::from_vec(int.to_digits(Order::LsfLe));
    bits.truncate(bits.last_one().map_or(0, |i| i + 1));
    bits.reverse();
    bits
}

pub fn signed_bits_from_integer(int: &Integer, sign: Sign, leading_zeros: usize) -> BitVec {
    let mut bits;
    if int.cmp0() == Ordering::Equal {
        if leading_zeros == 0 && sign == Sign::Empty {
            bits = BitVec::new();
        } else {
            bits = BitVec::repeat(false, leading_zeros + 1);
            if sign == Sign::Neg {
                bits.set(0, true);
            }
        }
    } else {
        bits = unsigned_bits_from_integer(int);
        let len = bits.len();
        // Newly-reserved bits are guaranteed to be allocated to zero
        bits.reserve(leading_zeros + 1);
        unsafe { bits.set_len(len + leading_zeros + 1) };
        // Panics when shifting by the length
        if len != 0 {
            bits.shift_right(leading_zeros + 1);
        }
        if sign == Sign::Neg {
            bits.set(0, true);
        }
    }
    bits
}
