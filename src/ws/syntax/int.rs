// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::fmt::{self, Display, Formatter};
use std::intrinsics::unlikely;
use std::ops::{Deref, DerefMut};
use std::str;

use bitvec::prelude::*;
use compact_str::CompactString;
use rug::Integer;
use static_assertions::assert_type_eq_all;

use crate::ws::syntax::convert;

assert_type_eq_all!(BitSlice, BitSlice<usize, Lsb0>);
assert_type_eq_all!(BitVec, BitVec<usize, Lsb0>);
assert_type_eq_all!(BitBox, BitBox<usize, Lsb0>);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IntLiteral {
    /// Bit representation with the sign in the first bit (if nonempty) and
    /// possible leading zeros.
    bits: BitVec,
    /// String representation from Whitespace assembly source.
    string: Option<CompactString>,
    /// Numeric representation.
    int: Integer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sign {
    Pos,
    Neg,
    Empty,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ParseError {
    InvalidRadix,
    InvalidDigit { ch: char, offset: usize },
    LeadingUnderscore,
    NoDigits,
}

impl IntLiteral {
    /// Parses an integer with the given radix. A radix in 2..=36 uses the
    /// case-insensitive alphabet 0-9A-Z, so upper- and lowercase letters are
    /// equivalent. A radix in 37..=62 uses the case-sensitive alphabet
    /// 0-9A-Za-z.
    pub fn parse_radix<B: AsRef<[u8]>>(b: B, radix: u32) -> Result<Self, ParseError> {
        let b = b.as_ref();
        let (sign, offset) = Self::parse_sign(b);
        if offset == b.len() {
            return Err(ParseError::NoDigits);
        }
        Self::parse_digits(b, offset, sign, radix)
    }

    /// Parses an Erlang-like integer literal of the form `base#value`, where
    /// `base` is in 2..=62, or just `value` (for base 10). Underscores may
    /// separate digits.
    ///
    /// ## Differences from Erlang
    ///
    /// - Bases 37..=62 are also allowed, which use the case-sensitive alphabet
    ///   0-9A-Za-z
    /// - Letter aliases may be used for `base`: `b`/`B` for binary, `o`/`O` for
    ///   octal, and `x`/`X` for hexadecimal
    /// - `value` for base 2 may be empty, to allow for expressing all forms of
    ///   Whitespace bit patterns
    ///
    /// ## Grammar
    ///
    /// - Base 2: `[+-]?[2bB]#[01_]*` (note the optional digits)
    /// - Base 8: `[+-]?[8oO]#[0-7_]+`
    /// - Base 10: `[+-]?(10#[0-9_]+|[0-9][0-9_]*)`
    /// - Base 16: `[+-]?(16|[xX])#[0-9A-Fa-f_]+`
    /// - Other bases: `[+-]?[0-9]{1,2}#[0-9A-Za-z_]+`, where the base and
    ///   digits must be in range
    pub fn parse_erlang_style<B: AsRef<[u8]>>(b: B) -> Result<Self, ParseError> {
        let b = b.as_ref();
        let (sign, offset) = Self::parse_sign(b);
        let (radix, offset) = match b[offset..] {
            [r, b'#', ..] => (
                match r {
                    b'b' | b'B' => 2,
                    b'o' | b'O' => 8,
                    b'x' | b'X' => 16,
                    b'2'..=b'9' => (r - b'0') as u32,
                    _ => return Err(ParseError::InvalidRadix),
                },
                offset + 2,
            ),
            [r0, r1, b'#', ..] => (
                match (r0, r1) {
                    (b'1'..=b'6', b'0'..=b'9') => ((r0 - b'0') * 10 + r1 - b'0') as u32,
                    _ => return Err(ParseError::InvalidRadix),
                },
                offset + 3,
            ),
            [b'_', ..] => return Err(ParseError::LeadingUnderscore),
            // Allow the digits to be omitted only with an explicit radix.
            [] => return Err(ParseError::NoDigits),
            _ => (10, offset),
        };
        match Self::parse_digits(b, offset, sign, radix) {
            Err(ParseError::InvalidDigit { ch: '#', .. }) => Err(ParseError::InvalidRadix),
            result => result,
        }
    }

    /// Parses a C-like integer literal with an optional prefix that denotes the
    /// base: `0b`/`0B` for binary, `0`/`0o`/`0O` for octal, `0x`/`0X` for
    /// hexadecimal, or decimal otherwise. Underscores may separate digits.
    ///
    /// ## Grammar
    ///
    /// - Base 2: `[+-]?0[bB][01_]+`
    /// - Base 8: `[+-]?0[oO]?[0-7_]+`
    /// - Base 10: `[+-]?[0-9][0-9_]*`
    /// - Base 16: `[+-]?0[xX]?[0-9A-Fa-f_]+`
    pub fn parse_c_style<B: AsRef<[u8]>>(b: B) -> Result<Self, ParseError> {
        let b = b.as_ref();
        let (sign, offset) = Self::parse_sign(b);
        let (radix, offset) = match b[offset..] {
            [b'0', b'b' | b'B', ..] => (2, offset + 2),
            [b'0', b'o' | b'O', ..] => (8, offset + 2),
            [b'0', b'x' | b'X', ..] => (16, offset + 2),
            [b'0', _, ..] => (8, offset + 1),
            [b'_', ..] => return Err(ParseError::LeadingUnderscore),
            _ => (10, offset),
        };
        if offset == b.len() {
            return Err(ParseError::NoDigits);
        }
        Self::parse_digits(b, offset, sign, radix)
    }

    #[inline]
    fn parse_sign(b: &[u8]) -> (Sign, usize) {
        match b.first() {
            Some(b'+') => (Sign::Pos, 1),
            Some(b'-') => (Sign::Neg, 1),
            _ => (Sign::Empty, 0),
        }
    }

    fn parse_digits(
        b: &[u8],
        mut offset: usize,
        sign: Sign,
        radix: u32,
    ) -> Result<Self, ParseError> {
        let table: &[u8; 256] = match radix {
            2..=36 => RADIX_DIGIT_VALUES.split_array_ref().0,
            37..=62 => RADIX_DIGIT_VALUES[208..].try_into().unwrap(),
            _ => return Err(ParseError::InvalidRadix),
        };

        // Skip leading zeros
        let mut leading_zeros = 0usize;
        while offset < b.len() {
            match b[offset] {
                b'0' => leading_zeros += 1,
                b'_' => {}
                _ => break,
            }
            offset += 1;
        }

        let mut digits = Vec::with_capacity(b.len() - offset);
        for i in offset..b.len() {
            let ch = b[i];
            let digit = table[ch as usize];
            if unlikely(digit as u32 >= radix) {
                if ch == b'_' {
                    continue;
                }
                // The invalid digit may be non-ASCII; decode it
                let ch = bstr::decode_utf8(&b[i..]).0.unwrap_or('\u{FFFD}');
                return Err(ParseError::InvalidDigit { ch, offset: i });
            }
            digits.push(digit);
        }

        // Only use leading zeros for base 2 and zero
        let leading_zeros = if radix == 2 {
            leading_zeros
        } else if digits.is_empty() && leading_zeros != 0 {
            1
        } else {
            0
        };
        let int = convert::integer_from_digits_radix(digits, sign, radix);
        let bits = convert::signed_bits_from_integer(&int, sign, leading_zeros);
        // SAFETY: The syntax only allows for ASCII characters
        let string = Some(unsafe { str::from_utf8_unchecked(b) }.into());
        Ok(IntLiteral { bits, string, int })
    }

    #[inline]
    pub fn sign(&self) -> Sign {
        match self.bits.first().as_deref() {
            Some(true) => Sign::Neg,
            Some(false) => Sign::Pos,
            None => Sign::Empty,
        }
    }
}

const X: u8 = 0xff;
/// Table indexed by ASCII character, to get its numeric value. The first part
/// of the table (0..256) is for radices 2..=36 that use the case-insensitive
/// alphabet 0-9A-Z, so upper- and lowercase letters map to the same digit. The
/// second part (208..=464) is for radices 37..=62 that use the case-sensitive
/// alphabet 0-9A-Za-z.
///
/// Copied from __gmp_digit_value_tab in gmp-6.2.1/mp_dv_tab.c
#[rustfmt::skip]
static RADIX_DIGIT_VALUES: [u8; 464] = [
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, X, X, X, X, X, X,
    X,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,
    25,26,27,28,29,30,31,32,33,34,35,X, X, X, X, X,
    X,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,
    25,26,27,28,29,30,31,32,33,34,35,X, X, X, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, X, X, X, X, X, X,
    X,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,
    25,26,27,28,29,30,31,32,33,34,35,X, X, X, X, X,
    X,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,
    51,52,53,54,55,56,57,58,59,60,61,X, X, X, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X,
];

impl From<BitVec> for IntLiteral {
    #[inline]
    fn from(bits: BitVec) -> Self {
        let int = convert::integer_from_signed_bits(&bits);
        IntLiteral { bits, string: None, int }
    }
}

impl Deref for IntLiteral {
    type Target = Integer;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.int
    }
}

impl DerefMut for IntLiteral {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.int
    }
}

impl Display for IntLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(ref s) = self.string {
            f.write_str(s.as_str())
        } else {
            if self.bits.get(1).as_deref() == Some(&true) || self.bits.len() == 2 {
                write!(f, "{}", self.int)
            } else {
                // Write numbers with leading zeros in base 2
                let sign = if self.bits.get(0).as_deref() == Some(&true) {
                    "-"
                } else if self.bits.len() == 1 {
                    // Sign-only numbers need an explicit positive sign
                    "+"
                } else {
                    ""
                };
                let bin = self.bits[1..]
                    .iter()
                    .map(|b| if *b { '1' } else { '0' })
                    .collect::<String>();
                write!(f, "{sign}b#{bin}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ParseTest {
        syntax: SyntaxStyle,
        string: &'static str,
        result: Result<IntLiteral, ParseError>,
    }
    #[derive(Debug)]
    enum SyntaxStyle {
        Erlang,
        C,
    }

    macro_rules! parse_test_ok(
        ($syntax:ident, $s:literal, $int:literal, [$($bit:literal)*]) => {
            ParseTest {
                syntax: SyntaxStyle::$syntax,
                string: $s,
                result: Ok(IntLiteral {
                    bits: bitvec![$($bit),*],
                    string: Some($s.into()),
                    int: Integer::parse($int).unwrap().into(),
                }),
            }
        }
    );
    macro_rules! parse_test_err(
        ($syntax:ident, $s:literal, $err:expr) => {
            ParseTest {
                syntax: SyntaxStyle::$syntax,
                string: $s.into(),
                result: Err($err),
            }
        }
    );

    #[test]
    fn parse() {
        use ParseError::*;
        for test in [
            parse_test_ok!(Erlang, "0", "0", [0 0]),
            parse_test_ok!(Erlang, "00", "00", [0 0]),
            parse_test_ok!(Erlang, "000", "000", [0 0]),
            parse_test_ok!(Erlang, "b#", "0", []),
            parse_test_ok!(Erlang, "+b#", "0", [0]),
            parse_test_ok!(Erlang, "-b#", "0", [1]),
            parse_test_ok!(Erlang, "b#0", "0", [0 0]),
            parse_test_ok!(Erlang, "+b#0", "0", [0 0]),
            parse_test_ok!(Erlang, "-b#0", "0", [1 0]),
            parse_test_ok!(Erlang, "42", "42", [0 1 0 1 0 1 0]),
            parse_test_ok!(Erlang, "16#123", "291", [0 1 0 0 1 0 0 0 1 1]),
            parse_test_ok!(Erlang, "16#dead_BEEF", "3735928559", [0 1 1 0 1 1 1 1 0 1 0 1 0 1 1 0 1 1 0 1 1 1 1 1 0 1 1 1 0 1 1 1 1]),
            parse_test_ok!(Erlang, "-60#100", "-3600", [1 1 1 1 0 0 0 0 1 0 0 0 0]),
            parse_test_ok!(Erlang, "3#_0", "0", [0 0]),
            parse_test_ok!(Erlang, "-21#4_", "-4", [1 1 0 0]),
            parse_test_ok!(Erlang, "42#7__K", "314", [0 1 0 0 1 1 1 0 1 0]),
            parse_test_err!(Erlang, "b", InvalidDigit { ch: 'b', offset: 0 }),
            parse_test_err!(Erlang, "B", InvalidDigit { ch: 'B', offset: 0 }),
            parse_test_err!(Erlang, "o", InvalidDigit { ch: 'o', offset: 0 }),
            parse_test_err!(Erlang, "O", InvalidDigit { ch: 'O', offset: 0 }),
            parse_test_err!(Erlang, "x", InvalidDigit { ch: 'x', offset: 0 }),
            parse_test_err!(Erlang, "X", InvalidDigit { ch: 'X', offset: 0 }),
            parse_test_err!(Erlang, "#", InvalidRadix),
            parse_test_err!(Erlang, "a#", InvalidRadix),
            parse_test_err!(Erlang, "ab#", InvalidRadix),
            parse_test_err!(Erlang, "abc#", InvalidDigit { ch: 'a', offset: 0 }),
            parse_test_err!(Erlang, "0#", InvalidRadix),
            parse_test_err!(Erlang, "1#", InvalidRadix),
            parse_test_err!(Erlang, "63#", InvalidRadix),
            parse_test_err!(Erlang, "_", LeadingUnderscore),
            parse_test_err!(Erlang, "-_", LeadingUnderscore),
            parse_test_err!(Erlang, "_1__2__3_", LeadingUnderscore),
            parse_test_ok!(C, "0", "0", [0 0]),
            parse_test_ok!(C, "00", "00", [0 0]),
            parse_test_ok!(C, "000", "000", [0 0]),
            parse_test_ok!(C, "0755", "493", [0 1 1 1 1 0 1 1 0 1]),
            parse_test_ok!(C, "0_755", "493", [0 1 1 1 1 0 1 1 0 1]),
            parse_test_ok!(C, "-0x_fe", "-254", [1 1 1 1 1 1 1 1 0]),
            parse_test_err!(C, "0b", NoDigits),
            parse_test_err!(C, "0x", NoDigits),
            parse_test_err!(C, "0a", InvalidDigit { ch: 'a', offset: 1 }),
            parse_test_err!(C, "_", LeadingUnderscore),
            parse_test_err!(C, "-_", LeadingUnderscore),
            parse_test_err!(C, "_1__2__3_", LeadingUnderscore),
        ] {
            let result = match test.syntax {
                SyntaxStyle::Erlang => IntLiteral::parse_erlang_style(test.string),
                SyntaxStyle::C => IntLiteral::parse_c_style(test.string),
            };
            assert_eq!(
                test.result, result,
                "parse {} with {:?} syntax",
                test.string, test.syntax,
            );
        }
    }
}
