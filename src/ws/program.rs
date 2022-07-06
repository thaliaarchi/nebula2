// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use std::collections::{hash_map::Entry, HashMap};

use bitvec::prelude::BitVec;
use rug::Integer;

use crate::ws::inst::{Inst, Opcode, RawInst};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Program {
    insts: Vec<ProgramInst>,
    labels: Vec<Label>,
}

pub type ProgramInst = Inst<Int, usize>;

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

/// The ordering to use for assigning label ids when serializing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LabelOrder {
    /// In order of definition
    #[default]
    Def,
    /// In order of first definition or use
    DefOrUse,
}

#[derive(Clone, Debug)]
pub struct LabelResolver {
    labels: Vec<Label>,
    bits_map: HashMap<BitVec, usize>,
}

impl LabelResolver {
    #[inline]
    pub fn new() -> Self {
        LabelResolver {
            labels: Vec::new(),
            bits_map: HashMap::new(),
        }
    }

    pub fn resolve_all(&mut self, insts: Vec<RawInst>) -> Vec<ProgramInst> {
        let mut resolved = Vec::with_capacity(insts.len());
        for (i, inst) in insts.into_iter().enumerate() {
            resolved.push(self.resolve(inst, i));
        }
        resolved
    }

    pub fn resolve(&mut self, inst: RawInst, pc: usize) -> ProgramInst {
        inst.map(
            |_opcode, n| Int { bits: n, int: Integer::new() }, // TODO
            |opcode, l| self.insert(l, pc, opcode),
        )
    }

    fn insert(&mut self, bits: BitVec, pc: usize, opcode: Opcode) -> usize {
        match self.bits_map.entry(bits.clone()) {
            Entry::Occupied(entry) => {
                let id = *entry.get();
                if opcode == Opcode::Label {
                    self.labels[id].defs.push(pc);
                } else {
                    self.labels[id].uses.push(pc);
                }
                id
            }
            Entry::Vacant(entry) => {
                entry.insert(self.labels.len());
                let (defs, uses) = if opcode == Opcode::Label {
                    (vec![pc], Vec::new())
                } else {
                    (Vec::new(), vec![pc])
                };
                self.labels.push(Label {
                    bits,
                    num: None,  // TODO
                    name: None, // TODO
                    id: pc,
                    defs,
                    uses,
                });
                pc
            }
        }
    }
}
