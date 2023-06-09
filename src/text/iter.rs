// Copyright (C) 2022 Thalia Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::iter::FusedIterator;

use arrayvec::ArrayVec;
use bstr::decode_utf8;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EncodingError {
    InvalidUtf8(ArrayVec<u8, 3>),
}

#[derive(Clone, Debug)]
pub struct Utf8Iterator<'a> {
    src: &'a [u8],
    offset: usize,
    valid_to: Option<usize>,
    error_once: bool,
}

impl<'a> Utf8Iterator<'a> {
    #[inline]
    #[must_use]
    pub const fn new<B>(src: &'a B, error_once: bool) -> Self
    where
        B: ~const AsRef<[u8]> + ?Sized,
    {
        Utf8Iterator {
            src: src.as_ref(),
            offset: 0,
            valid_to: None,
            error_once,
        }
    }

    #[inline]
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    #[must_use]
    pub const fn valid_to(&self) -> usize {
        match self.valid_to {
            Some(valid_to) => valid_to,
            None => self.offset,
        }
    }
}

impl Iterator for Utf8Iterator<'_> {
    type Item = Result<char, EncodingError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.src.len() {
            return None;
        }
        let offset = self.offset;
        let (ch, size) = decode_utf8(&self.src[offset..]);
        self.offset += size;
        match ch {
            Some(ch) => Some(Ok(ch)),
            None if self.valid_to == None || !self.error_once => {
                self.valid_to = Some(offset);
                // Size is guaranteed to be between 1 and 3, inclusive, for
                // an unsuccessful decode.
                let mut bad = ArrayVec::new();
                bad.try_extend_from_slice(&self.src[offset..offset + size])
                    .unwrap();
                Some(Err(EncodingError::InvalidUtf8(bad)))
            }
            None => None,
        }
    }
}

impl const FusedIterator for Utf8Iterator<'_> {}

#[derive(Clone, Debug)]
pub struct ByteIterator<'a> {
    src: &'a [u8],
    offset: usize,
}

impl<'a> ByteIterator<'a> {
    #[inline]
    #[must_use]
    pub const fn new<B: ~const AsRef<[u8]> + ?Sized>(src: &'a B) -> Self {
        ByteIterator { src: src.as_ref(), offset: 0 }
    }

    #[inline]
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }
}

impl Iterator for ByteIterator<'_> {
    type Item = Result<u8, EncodingError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.src.len() {
            return None;
        }
        let b = self.src[self.offset];
        self.offset += 1;
        Some(Ok(b))
    }
}

impl const FusedIterator for ByteIterator<'_> {}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use static_assertions::const_assert;

    use super::*;

    const_assert!(size_of::<ArrayVec<u8, 3>>() < size_of::<Vec<u8>>());
}
