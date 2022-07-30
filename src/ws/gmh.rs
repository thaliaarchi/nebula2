// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

//! GrassMudHorse is an extension to Whitespace syntax.
//!
//! # Syntax
//!
//! GrassMudHorse uses [草 “grass”](Token::G), [泥 “mud”](Token::M), and
//! [马 “horse”](Token::H) for [`S`](ws::Token::S), [`T`][ws::Token::T], and
//! [`L`](ws::Token::L) tokens, respectively, and adds two more tokens:
//! [河 “river”](Token::R) and [蟹 “crab”](Token::C). It extends the grammar by
//! allowing [`end`](crate::ws::inst::Inst::End) to be written equivalently as
//! either “river crab” or “horse horse horse” (i.e., `L` `L` `L`). Only some
//! implementations support river and crab.
//!
//! # Error handling
//!
//! There are two behaviors for handling unpaired river and crab tokens:
//! ignoring them (as in the original [Java implementation](https://github.com/wspace/bearice-grassmudhorse/blob/main/src/cn/icybear/GrassMudHorse/JOTCompiler.java))
//! or erroring (as in the [Erlang implementation](https://github.com/wspace/bearice-grassmudhorse/tree/main/erlang)
//! by the same author).
//!
//! # Name
//!
//! GrassMudHorse is a reference to a couple of Chinese internet memes: The
//! [Grass Mud Horse](https://en.wikipedia.org/wiki/Grass_Mud_Horse) (a profane
//! homophonic pun) is said to be a species of alpaca from the Mahler Gobi
//! Desert, whose existence is threatened by [river crabs](https://en.wikipedia.org/wiki/Euphemisms_for_Internet_censorship_in_China)
//! (a pun criticizing internet censorship).

// TODO: Parse river crab syntax for `end`. A mechanism for token and opcode
// extensions would avoid code duplication from wrapping `Parser` and allow for
// other simple extensions. Alternatively, river and crab could be handled in a
// comment callback or analysis.

use crate::ws;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    /// Grass (草 U+8349)
    G,
    /// Mud (泥 U+6CE5)
    M,
    /// Horse (马 U+9A6C)
    H,
    /// River (河 U+6CB3)
    R,
    /// Crab (蟹 U+87F9)
    C,
}

impl Token {
    #[inline]
    pub const fn as_ws_token(&self) -> Option<ws::Token> {
        match self {
            Token::G => Some(ws::Token::S),
            Token::M => Some(ws::Token::T),
            Token::H => Some(ws::Token::L),
            Token::R | Token::C => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Mapping {
    g: char,
    m: char,
    h: char,
    r: char,
    c: char,
}

impl Mapping {
    #[inline]
    pub const fn new(g: char, m: char, h: char, r: char, c: char) -> Option<Self> {
        if g == m
            || g == h
            || g == r
            || g == c
            || m == h
            || m == r
            || m == c
            || h == r
            || h == c
            || r == c
        {
            return None;
        }
        Some(Mapping { g, m, h, r, c })
    }

    #[inline]
    pub const fn map(&self, ch: char) -> Option<Token> {
        match ch {
            _ if ch == self.g => Some(Token::G),
            _ if ch == self.m => Some(Token::M),
            _ if ch == self.h => Some(Token::H),
            _ if ch == self.r => Some(Token::R),
            _ if ch == self.c => Some(Token::C),
            _ => None,
        }
    }

    #[inline]
    pub const fn map_token(&self, tok: Token) -> char {
        match tok {
            Token::G => self.g,
            Token::M => self.m,
            Token::H => self.h,
            Token::R => self.r,
            Token::C => self.c,
        }
    }
}

impl const Default for Mapping {
    #[inline]
    fn default() -> Self {
        Mapping {
            g: '草',
            m: '泥',
            h: '马',
            r: '河',
            c: '蟹',
        }
    }
}
