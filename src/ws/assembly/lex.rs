// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use crate::ws::assembly::Cursor;

/// Parsed token. It doesn't contain information about data that has been
/// parsed; only the type of the token and its size.
#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub len: u32,
}

impl Token {
    #[inline]
    const fn new(kind: TokenKind, len: u32) -> Token {
        Token { kind, len }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    LineComment {
        style: LineCommentStyle,
    },
    BlockComment {
        style: BlockCommentStyle,
        terminated: bool,
    },

    Word,
    /// Integer literal
    Int {
        base: Base,
        has_digits: bool,
    },
    /// String literal (`"…"`)
    String,
    /// Character literal (`'…'`)
    Char,

    /// `:`
    Colon,
    /// Line feed
    Lf,
    /// Whitespace character sequence, excluding line feed
    Whitespace,

    /// Unknown token not expected by the lexer
    Unknown,
}
use TokenKind::*;

// https://en.wikipedia.org/wiki/Comparison_of_programming_languages_(syntax)#Comments

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LineCommentStyle {
    /// `//` (C++)
    SlashSlash,
    /// `--` (Haskell)
    DashDash,
    /// `#` (shell, Ruby, and Python)
    Pound,
    /// `;` (assembly and Scheme)
    Semi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockCommentStyle {
    /// `/*` … `*/` (C)
    SlashStar,
    /// `{-` … `-}` (Haskell; nestable)
    BraceDash,
    /// `(` … `)` (Forth)
    Paren,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Base {
    Decimal,
    /// `0b` prefix
    Binary,
    /// `0o` prefix
    Octal,
    /// `0x` prefix
    Hexadecimal,
}

impl Cursor<'_> {
    /// Scans a token from the input string.
    fn advance_token(&mut self) -> Token {
        let first_char = self.bump().unwrap();
        let token_kind = match first_char {
            // Comments
            '/' => match self.first() {
                '/' => self.line_comment2('/', '/', LineCommentStyle::SlashSlash),
                '*' => self.block_comment2('/', '*', '*', '/', BlockCommentStyle::SlashStar),
                _ => self.word(),
            },
            '{' if self.first() == '-' => {
                self.block_comment2_nested('{', '-', '-', '}', BlockCommentStyle::BraceDash)
            }
            '-' if self.first() == '-' => self.line_comment2('-', '-', LineCommentStyle::DashDash),
            '(' => self.block_comment1('(', ')', BlockCommentStyle::Paren),
            '#' => self.line_comment1('#', LineCommentStyle::Pound),
            ';' => self.line_comment1(';', LineCommentStyle::Semi),

            // Literals
            '+' | '-' => self.signed_int_or_word(),
            '0'..='9' => self.unsigned_int(first_char),
            '"' => self.string(),
            '\'' => self.char(),

            ':' => Colon,
            '\n' => Lf,

            c if c.is_whitespace() => self.whitespace(),
            c if c.is_control() => Unknown,
            _ => self.word(),
        };
        Token::new(token_kind, self.len_consumed())
    }

    #[inline]
    fn word(&mut self) -> TokenKind {
        debug_assert!(!self.prev().is_whitespace() && !self.prev().is_control());
        self.eat_while(|c| !c.is_whitespace() && c != ':' && !c.is_control());
        Word
    }

    #[inline]
    fn signed_int_or_word(&mut self) -> TokenKind {
        debug_assert!(matches!(self.prev(), '+' | '-'));
        todo!()
    }

    #[inline]
    fn unsigned_int(&mut self, first_digit: char) -> TokenKind {
        debug_assert!(matches!(self.prev(), '0'..='9'));
        let mut base = Base::Decimal;
        let has_digits = if first_digit == '0' {
            // Attempt to parse encoding base.
            match self.first() {
                'b' => {
                    base = Base::Binary;
                    self.bump();
                    self.eat_decimal_digits()
                }
                'o' => {
                    base = Base::Octal;
                    self.bump();
                    self.eat_decimal_digits()
                }
                'x' => {
                    base = Base::Hexadecimal;
                    self.bump();
                    self.eat_hexadecimal_digits()
                }
                // Not a base prefix.
                '0'..='9' | '_' => {
                    self.eat_decimal_digits();
                    true
                }
                // Just a `0`.
                _ => true,
            }
        } else {
            // No base prefix; parse number in the usual way.
            self.eat_decimal_digits();
            true
        };
        Int { base, has_digits }
    }

    #[inline]
    fn string(&mut self) -> TokenKind {
        todo!()
    }

    #[inline]
    fn char(&mut self) -> TokenKind {
        todo!()
    }

    #[inline]
    fn line_comment1(&mut self, tag: char, style: LineCommentStyle) -> TokenKind {
        debug_assert!(self.prev() == tag);
        self.eat_while(|c| c != '\n');
        LineComment { style }
    }

    #[inline]
    fn line_comment2(&mut self, tag1: char, tag2: char, style: LineCommentStyle) -> TokenKind {
        debug_assert!(self.prev() == tag1 && self.first() == tag2);
        self.bump();
        self.eat_while(|c| c != '\n');
        LineComment { style }
    }

    #[inline]
    fn block_comment1(&mut self, open: char, close: char, style: BlockCommentStyle) -> TokenKind {
        debug_assert!(self.prev() == open);
        self.eat_while(|c| c != close);
        let terminated = self.first() == close;
        if terminated {
            self.bump();
        }
        BlockComment { style, terminated }
    }

    #[inline]
    fn block_comment2(
        &mut self,
        open1: char,
        open2: char,
        close1: char,
        close2: char,
        style: BlockCommentStyle,
    ) -> TokenKind {
        debug_assert!(self.prev() == open1 && self.first() == open2);
        self.bump();
        let mut terminated = false;
        while let Some(c) = self.bump() {
            if c == close1 && self.first() == close2 {
                self.bump();
                terminated = true;
                break;
            }
        }
        BlockComment { style, terminated }
    }

    #[inline]
    fn block_comment2_nested(
        &mut self,
        open1: char,
        open2: char,
        close1: char,
        close2: char,
        style: BlockCommentStyle,
    ) -> TokenKind {
        debug_assert!(self.prev() == open1 && self.first() == open2);
        self.bump();

        let mut depth = 1usize;
        while let Some(c) = self.bump() {
            match c {
                c if c == open1 && self.first() == open2 => {
                    self.bump();
                    depth += 1;
                }
                c if c == close1 && self.first() == close2 => {
                    self.bump();
                    depth -= 1;
                    if depth == 0 {
                        // This block comment is closed, so for a construction
                        // like `/* */ */`, there will be a successfully-parsed
                        // block comment `/* */` and ` */` will be processed
                        // separately.
                        break;
                    }
                }
                _ => (),
            }
        }
        BlockComment { style, terminated: depth == 0 }
    }

    #[inline]
    fn whitespace(&mut self) -> TokenKind {
        debug_assert!(self.prev().is_whitespace());
        self.eat_while(char::is_whitespace);
        Whitespace
    }

    #[inline]
    fn eat_decimal_digits(&mut self) -> bool {
        let mut has_digits = false;
        loop {
            match self.first() {
                '0'..='9' => has_digits = true,
                '_' => {}
                _ => break,
            }
            self.bump();
        }
        has_digits
    }

    #[inline]
    fn eat_hexadecimal_digits(&mut self) -> bool {
        let mut has_digits = false;
        loop {
            match self.first() {
                '_' => {}
                '0'..='9' | 'a'..='f' | 'A'..='F' => has_digits = true,
                _ => break,
            }
            self.bump();
        }
        has_digits
    }
}
