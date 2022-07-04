// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use bitvec::prelude::*;
use std::collections::HashMap;

use crate::ws::inst::{Features, Inst, Int, Opcode, Sign, Uint};
use crate::ws::token::{Token, Token::*, TokenSeq};

#[derive(Clone, Debug)]
pub struct Parser {
    table: ParseTable,
    toks: Vec<Token>,
    offset: usize,
    features: Features,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    UnknownInst(TokenSeq),
    IncompleteInst(Vec<Opcode>),
    SignlessInt(Opcode),
    UnterminatedInt(Opcode),
    UnterminatedLabel(Opcode),
}

impl Parser {
    pub fn new(toks: Vec<Token>, features: Features) -> Self {
        let mut table = ParseTable::new();
        table.insert_insts().unwrap();
        Parser {
            table,
            toks,
            offset: 0,
            features,
        }
    }

    fn next_tok(&mut self) -> Option<Token> {
        if self.offset >= self.toks.len() {
            return None;
        }
        let tok = self.toks[self.offset];
        self.offset += 1;
        Some(tok)
    }

    fn parse_arg(&mut self, opcode: Opcode) -> Result<Inst, ParseError> {
        opcode.parse_arg(self)
    }

    pub(crate) fn parse_int(&mut self, opcode: Opcode) -> Result<Int, ParseError> {
        let sign = match self.next_tok().ok_or(ParseError::UnterminatedInt(opcode))? {
            S => Sign::Pos,
            T => Sign::Neg,
            L => return Err(ParseError::SignlessInt(opcode)),
        };
        let bits = self
            .parse_bitvec()
            .ok_or(ParseError::UnterminatedInt(opcode))?;
        Ok(Int { sign, bits })
    }

    pub(crate) fn parse_uint(&mut self, opcode: Opcode) -> Result<Uint, ParseError> {
        let bits = self
            .parse_bitvec()
            .ok_or(ParseError::UnterminatedLabel(opcode))?;
        Ok(Uint { bits })
    }

    fn parse_bitvec(&mut self) -> Option<BitVec> {
        let mut bits = BitVec::new();
        while let Some(tok) = self.next_tok() {
            match tok {
                S => bits.push(false),
                T => bits.push(true),
                L => return Some(bits),
            }
        }
        None
    }
}

impl Iterator for Parser {
    type Item = Result<Inst, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.toks.len() {
            return None;
        }
        let mut seq = TokenSeq::new();
        loop {
            seq = seq.push(self.toks[self.offset]);
            self.offset += 1;
            match self.table.get(seq) {
                ParseEntry::None => return Some(Err(ParseError::UnknownInst(seq))),
                ParseEntry::Prefix(opcodes) => {
                    if self.offset >= self.toks.len() {
                        return Some(Err(ParseError::IncompleteInst(opcodes.clone())));
                    }
                }
                ParseEntry::Terminal(opcode) => {
                    return Some(self.parse_arg(*opcode));
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ParseTable {
    dense: Box<[ParseEntry; Self::DENSE_LEN]>,
    sparse: HashMap<TokenSeq, ParseEntry>,
}

#[derive(Clone, Debug, Default)]
pub enum ParseEntry {
    #[default]
    None,
    Prefix(Vec<Opcode>),
    Terminal(Opcode),
}

#[derive(Clone, Debug)]
pub enum ParserError {
    Conflict { seq: TokenSeq, opcodes: Vec<Opcode> },
    EmptyTokenSeq,
}

impl ParseTable {
    const DENSE_MAX: TokenSeq = TokenSeq::from_tokens(&[L, L, L]);
    const DENSE_LEN: usize = Self::DENSE_MAX.as_usize() + 1;

    pub fn new() -> Self {
        ParseTable {
            dense: vec![ParseEntry::None; Self::DENSE_LEN]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            sparse: HashMap::new(),
        }
    }

    pub fn get(&self, seq: TokenSeq) -> &ParseEntry {
        if seq <= Self::DENSE_MAX {
            &self.dense[seq.as_usize()]
        } else {
            &self.sparse[&seq]
        }
    }

    pub fn get_mut(&mut self, seq: TokenSeq) -> &mut ParseEntry {
        if seq <= Self::DENSE_MAX {
            &mut self.dense[seq.as_usize()]
        } else {
            self.sparse.entry(seq).or_default()
        }
    }

    pub fn insert(&mut self, toks: &[Token], opcode: Opcode) -> Result<(), ParserError> {
        if toks.len() == 0 {
            return Err(ParserError::EmptyTokenSeq);
        }
        let mut seq = TokenSeq::new();
        for &tok in toks {
            let entry = self.get_mut(seq);
            match entry {
                ParseEntry::None => *entry = ParseEntry::Prefix(vec![opcode]),
                ParseEntry::Prefix(opcodes) => opcodes.push(opcode),
                ParseEntry::Terminal(terminal) => {
                    let opcodes = vec![*terminal, opcode];
                    return Err(ParserError::Conflict { seq, opcodes });
                }
            }
            seq = seq.push(tok);
        }
        let entry = self.get_mut(seq);
        match entry {
            ParseEntry::None => *entry = ParseEntry::Terminal(opcode),
            ParseEntry::Prefix(opcodes) => {
                let mut opcodes = opcodes.clone();
                opcodes.push(opcode);
                return Err(ParserError::Conflict { seq, opcodes });
            }
            ParseEntry::Terminal(terminal) => {
                let opcodes = vec![*terminal, opcode];
                return Err(ParserError::Conflict { seq, opcodes });
            }
        }
        Ok(())
    }
}

#[test]
fn test_parse_tutorial() -> Result<(), ParseError> {
    let toks = vec![
        S, S, S, T, L, // push 1
        L, S, S, S, T, S, S, S, S, T, T, L, // label_C:
        S, L, S, // dup
        T, L, S, T, // printi
        S, S, S, T, S, T, S, L, // push 10
        T, L, S, S, // printc
        S, S, S, T, L, // push 1
        T, S, S, S, // add
        S, L, S, // dup
        S, S, S, T, S, T, T, L, // push 11
        T, S, S, T, // sub
        L, T, S, S, T, S, S, S, T, S, T, L, // jz label_E
        L, S, L, S, T, S, S, S, S, T, T, L, // jmp label_C
        L, S, S, S, T, S, S, S, T, S, T, L, // label_E:
        S, L, L, // drop
        L, L, L, // end
    ];
    let label_c = Uint {
        bits: bitvec![0, 1, 0, 0, 0, 0, 1, 1],
    };
    let label_e = Uint {
        bits: bitvec![0, 1, 0, 0, 0, 1, 0, 1],
    };
    let insts = vec![
        Inst::Push(Int {
            sign: Sign::Pos,
            bits: bitvec![1],
        }),
        Inst::Label(label_c.clone()),
        Inst::Dup,
        Inst::Printi,
        Inst::Push(Int {
            sign: Sign::Pos,
            bits: bitvec![1, 0, 1, 0],
        }),
        Inst::Printc,
        Inst::Push(Int {
            sign: Sign::Pos,
            bits: bitvec![1],
        }),
        Inst::Add,
        Inst::Dup,
        Inst::Push(Int {
            sign: Sign::Pos,
            bits: bitvec![1, 0, 1, 1],
        }),
        Inst::Sub,
        Inst::Jz(label_e.clone()),
        Inst::Jmp(label_c),
        Inst::Label(label_e),
        Inst::Drop,
        Inst::End,
    ];
    let parser = Parser::new(toks, Features::all());
    let insts2 = parser.collect::<Result<Vec<_>, ParseError>>()?;
    assert_eq!(insts, insts2);
    Ok(())
}
