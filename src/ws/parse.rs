// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::iter::FusedIterator;
use std::sync::LazyLock;

use bitvec::vec::BitVec;
use smallvec::SmallVec;

use crate::syntax::{PrefixError, PrefixTable, TokenSeq, Tokens};
use crate::text::EncodingError;
use crate::ws::inst::{Inst, InstArg, Opcode, RawInst};
use crate::ws::token::{Lexer, Token, TokenVec};

/// Prefix table for parsing Whitespace opcodes.
pub static TABLE: LazyLock<PrefixTable<Token, Opcode>> = LazyLock::new(|| PrefixTable::with_all(3));

#[derive(Clone, Debug)]
pub struct Parser<'a, L> {
    table: &'a PrefixTable<Token, Opcode>,
    lex: L,
    partial: Option<PartialState>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ParseError {
    EncodingError(EncodingError, Vec<Token>),
    UnknownOpcode(TokenSeq<Token>),
    IncompleteInst(TokenSeq<Token>, SmallVec<[Opcode; 16]>),
    UnterminatedArg(Opcode, BitVec),
}

#[derive(Clone, Debug)]
enum PartialState {
    ParsingOpcode(TokenSeq<Token>),
    ParsingArg(Opcode, BitVec),
}

impl<L: Lexer> Parser<'static, L> {
    #[inline]
    #[must_use]
    pub fn new(lex: L) -> Self {
        Parser {
            table: &TABLE,
            lex,
            partial: None,
        }
    }
}

impl<'a, L: Lexer> Parser<'a, L> {
    #[inline]
    #[must_use]
    pub fn with_table(table: &'a PrefixTable<Token, Opcode>, lex: L) -> Self {
        Parser { table, lex, partial: None }
    }

    fn parse_arg(&mut self, opcode: Opcode, partial: Option<BitVec>) -> RawInst {
        Inst::from(opcode).map_arg(|opcode, arg| {
            let mut bits = partial.unwrap_or_else(|| BitVec::with_capacity(64));
            loop {
                match self.lex.next() {
                    Some(Ok(Token::S)) => bits.push(false),
                    Some(Ok(Token::T)) => bits.push(true),
                    Some(Ok(Token::L)) => break,
                    Some(Err(err)) => {
                        let mut toks = Vec::from(opcode.tokens());
                        toks.append_bits(&bits);
                        self.partial = Some(PartialState::ParsingArg(opcode, bits));
                        return Err(ParseError::EncodingError(err, toks));
                    }
                    None => return Err(ParseError::UnterminatedArg(opcode, bits)),
                }
            }
            match arg {
                InstArg::Int(()) => Ok(InstArg::Int(bits)),
                InstArg::Label(()) => Ok(InstArg::Label(bits)),
            }
        })
    }
}

impl<L: Lexer> Iterator for Parser<'_, L> {
    type Item = RawInst;

    fn next(&mut self) -> Option<Self::Item> {
        // Restore state, if an instruction was interrupted with a lex error
        // after being partially parsed.
        let partial_seq = match self.partial.take() {
            Some(PartialState::ParsingOpcode(partial)) => partial,
            Some(PartialState::ParsingArg(opcode, bits)) => {
                return Some(self.parse_arg(opcode, Some(bits)));
            }
            None => TokenSeq::new(),
        };
        match self.table.parse_at(&mut self.lex, partial_seq)? {
            Ok(opcode) => Some(self.parse_arg(opcode, None)),
            Err(err) => {
                if let PrefixError::EncodingError(_, seq) = err {
                    self.partial = Some(PartialState::ParsingOpcode(seq));
                }
                Some(Inst::from(ParseError::from(err)))
            }
        }
    }
}

impl<L: Lexer + FusedIterator> const FusedIterator for Parser<'_, L> {}

impl From<PrefixError<Token, Opcode>> for ParseError {
    fn from(err: PrefixError<Token, Opcode>) -> Self {
        match err {
            PrefixError::EncodingError(err, seq) => ParseError::EncodingError(err, seq.into()),
            PrefixError::UnknownOpcode(seq) => ParseError::UnknownOpcode(seq),
            PrefixError::IncompleteOpcode(seq, prefix) => ParseError::IncompleteInst(seq, prefix),
        }
    }
}
