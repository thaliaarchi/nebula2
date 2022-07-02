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
    S,
    T,
    L,
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
