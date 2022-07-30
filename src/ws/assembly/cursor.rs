// Copyright (C) 2022 Andrew Archibald
// Copyright (C) 2019-2021 The Rust Project Developers
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

// Copied and adapted from rustc_lexer:
// https://github.com/rust-lang/rust/blob/1202bbaf48a0a919a2e0cfd8b7dce97e8fc3030d/compiler/rustc_lexer/src/cursor.rs

use std::str::Chars;

/// Peekable iterator over a `char` sequence.
///
/// Characters can be peeked via `first` and `second` and the position can be
/// shifted forward via `bump`.
pub(crate) struct Cursor<'a> {
    initial_len: usize,
    /// Iterator over `char`s; slightly faster than a `&str`.
    chars: Chars<'a>,
    #[cfg(debug_assertions)]
    prev: char,
}

pub(crate) const EOF_CHAR: char = '\0';

impl<'a> Cursor<'a> {
    pub(crate) fn new(input: &'a str) -> Cursor<'a> {
        Cursor {
            initial_len: input.len(),
            chars: input.chars(),
            #[cfg(debug_assertions)]
            prev: EOF_CHAR,
        }
    }

    /// Returns the last-eaten symbol (or `'\0'` in release builds). For debug
    /// assertions only.
    pub(crate) fn prev(&self) -> char {
        if cfg!(debug_assertions) {
            self.prev
        } else {
            EOF_CHAR
        }
    }

    /// Peeks the next symbol from the input stream without consuming it. If the
    /// requested position doesn't exist, `EOF_CHAR` is returned; however,
    /// `EOF_CHAR` doesn't always mean the actual end of file and it should be
    /// checked with `is_eof`.
    pub(crate) fn first(&self) -> char {
        // `.next()` optimizes better than `.nth(0)`
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    /// Peeks the second symbol from the input stream without consuming it.
    pub(crate) fn second(&self) -> char {
        // `.next()` optimizes better than `.nth(1)`
        let mut iter = self.chars.clone();
        iter.next();
        iter.next().unwrap_or(EOF_CHAR)
    }

    /// Checks if there is nothing more to consume.
    pub(crate) fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    /// Returns the amount of already-consumed symbols.
    pub(crate) fn len_consumed(&self) -> usize {
        self.initial_len - self.chars.as_str().len()
    }

    /// Resets the number of bytes consumed to 0.
    pub(crate) fn reset_len_consumed(&mut self) {
        self.initial_len = self.chars.as_str().len();
    }

    /// Moves to the next character.
    pub(crate) fn bump(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        #[cfg(debug_assertions)]
        {
            self.prev = c;
        }
        Some(c)
    }

    /// Eats symbols while `predicate` returns `true` or until the end of file
    /// is reached.
    pub(crate) fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        // LLVM can inline all of this and compile it down to fast iteration
        // over bytes.
        while predicate(self.first()) && !self.is_eof() {
            self.bump();
        }
    }
}
