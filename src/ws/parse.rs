// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use crate::ws::inst::Inst;
use crate::ws::token::TokenSeq;

pub struct Parser {
    entries: Vec<ParseEntry>,
}

enum ParseEntry {
    None,
    Prefix(Vec<TokenSeq>),
    Terminal(Box<dyn Fn(&mut Parser) -> Option<Inst>>),
}

impl Parser {
    #[inline]
    pub const fn new() -> Self {
        Parser {
            entries: Vec::new(),
        }
    }

    pub fn register(&mut self, _seq: TokenSeq, _handle: Box<dyn Fn(&mut Parser) -> Option<Inst>>) {
        todo!();
    }
}
