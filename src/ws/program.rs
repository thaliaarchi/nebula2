// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use bitvec::prelude::BitVec;
use rug::Integer;

use crate::ws::inst;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Program {
    insts: Vec<Inst>,
    labels: Vec<Label>,
}

pub type Inst = inst::Inst<Int, usize>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Int {
    bits: BitVec,
    int: Integer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sign {
    Pos,
    Neg,
    Empty,
}

impl Int {
    #[inline]
    pub fn sign(&self) -> Sign {
        if self.bits.len() == 0 {
            Sign::Empty
        } else if self.bits[0] {
            Sign::Neg
        } else {
            Sign::Pos
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Label {
    bits: BitVec,
    num: Option<Integer>,
    name: Option<String>,
    id: usize,
    defs: Vec<usize>,
    uses: Vec<usize>,
}

/// The resolution strategy for duplicate labels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LabelDupes {
    /// Duplicate labels are not allowed
    Unique,
    /// The first definition is used (wspace)
    First,
    /// The last definition is used
    Last,
}

/// The ordering to use for assigning label ids.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LabelOrder {
    /// In order of definition
    #[default]
    Def,
    /// In order of first definition or use
    DefOrUse,
}
