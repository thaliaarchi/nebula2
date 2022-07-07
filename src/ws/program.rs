// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use std::collections::{hash_map::Entry, HashMap};
use std::ops::{Index, IndexMut};

use bitvec::prelude::BitVec;
use rug::Integer;
use smallvec::SmallVec;

use crate::ws::inst::{Inst, InstArg, InstError, Opcode, RawInst};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Program {
    insts: Vec<ProgramInst>,
    labels: Vec<LabelData>,
}

pub type ProgramInst = Inst<Int, LabelId>;

macro_rules! id_index(
    ($Id:ident($Int:ty) indexes $T:ident in $($Container:ty),+) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $Id(pub $Int);

        impl const From<usize> for $Id {
            #[inline]
            fn from(id: usize) -> Self {
                $Id(id as u32)
            }
        }

        impl const From<$Id> for usize {
            #[inline]
            fn from(id: $Id) -> Self {
                id.0 as usize
            }
        }

        $(impl Index<$Id> for $Container {
            type Output = $T;

            #[inline]
            fn index(&self, id: $Id) -> &Self::Output {
                &self[id.0 as usize]
            }
        }

        impl IndexMut<$Id> for $Container {
            #[inline]
            fn index_mut(&mut self, id: $Id) -> &mut Self::Output {
                &mut self[id.0 as usize]
            }
        })+
    }
);

id_index!(InstId(u32) indexes ProgramInst in Vec<ProgramInst>, [ProgramInst]);
id_index!(LabelId(u32) indexes LabelData in Vec<LabelData>, [LabelData]);

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

impl From<BitVec> for Int {
    #[inline]
    fn from(bits: BitVec) -> Self {
        Int { bits, int: Integer::new() } // TODO
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LabelData {
    bits: BitVec,
    num: Option<Integer>,
    name: Option<String>,
    id: LabelId,
    defs: SmallVec<[InstId; 4]>,
    uses: SmallVec<[InstId; 4]>,
}

#[cfg(test)]
static_assertions::assert_eq_size!(Vec<LabelId>, SmallVec<[LabelId; 4]>);

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

impl LabelData {
    pub fn new(bits: BitVec, id: LabelId, inst: InstId, is_def: bool) -> Self {
        let mut label = LabelData {
            bits,
            num: None,  // TODO
            name: None, // TODO
            id,
            defs: SmallVec::new(),
            uses: SmallVec::new(),
        };
        if is_def {
            label.defs.push(inst);
        } else {
            label.uses.push(inst);
        };
        label
    }
}

#[derive(Clone, Debug)]
pub struct LabelResolver {
    labels: Vec<LabelData>,
    bits_map: HashMap<BitVec, LabelId>,
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
            resolved.push(self.resolve(inst, InstId::from(i)));
        }
        resolved
    }

    pub fn resolve(&mut self, inst: RawInst, id: InstId) -> ProgramInst {
        inst.map_arg(|opcode, arg| -> Result<_, InstError> {
            match arg {
                InstArg::Int(n) => Ok(InstArg::Int(Int::from(n))),
                InstArg::Label(l) => Ok(InstArg::Label(self.insert(l, id, opcode))),
            }
        })
    }

    fn insert(&mut self, bits: BitVec, inst: InstId, opcode: Opcode) -> LabelId {
        match self.bits_map.entry(bits.clone()) {
            Entry::Occupied(entry) => {
                let id = *entry.get();
                if opcode == Opcode::Label {
                    self.labels[id].defs.push(inst);
                } else {
                    self.labels[id].uses.push(inst);
                }
                id
            }
            Entry::Vacant(entry) => {
                let id = LabelId(self.labels.len() as u32);
                let label = LabelData::new(bits, id, inst, opcode == Opcode::Label);
                entry.insert(id);
                self.labels.push(label);
                id
            }
        }
    }
}
