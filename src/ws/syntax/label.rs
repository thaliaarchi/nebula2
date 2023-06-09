// Copyright (C) 2022 Thalia Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::collections::{hash_map::Entry, HashMap};
use std::fmt::{self, Display, Formatter};
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ops::{Index, IndexMut};

use bitvec::vec::BitVec;
use rug::Integer;
use smallvec::SmallVec;
use static_assertions::assert_eq_size;

use crate::ws::inst::{Inst, InstArg, InstError, Opcode, RawInst};
use crate::ws::syntax::{convert, IntLiteral};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Program {
    insts: Vec<ProgramInst>,
    labels: Vec<LabelData>,
}

pub type ProgramInst = Inst<IntLiteral, LabelId>;

macro_rules! id_index(
    ($Id:ident($Int:ty) indexes $T:ident in $($Container:ty),+) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $Id(pub $Int);

        impl  From<usize> for $Id {
            #[inline]
            fn from(id: usize) -> Self {
                $Id(id as u32)
            }
        }

        impl  From<$Id> for usize {
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
pub struct LabelLiteral {
    bits: BitVec,
    uint: Option<Integer>,
    name: Option<String>,
}

impl LabelLiteral {
    #[inline]
    #[must_use]
    pub fn from_bits(bits: BitVec) -> Self {
        let uint = convert::integer_from_unsigned_bits_unambiguous(&bits);
        LabelLiteral { bits, uint, name: None }
    }
}

impl Display for LabelLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(name) = self.name.as_ref() {
            write!(f, "{}", name)
        } else if let Some(uint) = self.uint.as_ref() {
            write!(f, "{}", uint)
        } else {
            write!(f, "{}", self.bits)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LabelData {
    id: LabelId,
    /// Canonical bit representation.
    bits: BitVec,
    /// Unsigned integer representation, if the bit representation has no
    /// leading zeros and is non-empty.
    uint: Option<Integer>,
    /// Names for defs/uses that come from Whitespace assembly.
    names: Vec<(InstId, String)>,
    defs: SmallVec<[InstId; 4]>,
    uses: SmallVec<[InstId; 4]>,
}

impl LabelData {
    #[inline]
    #[must_use]
    pub fn new(id: LabelId, bits: BitVec) -> Self {
        let uint = convert::integer_from_unsigned_bits_unambiguous(&bits);
        LabelData {
            id,
            bits,
            uint,
            names: Vec::new(),
            defs: SmallVec::new(),
            uses: SmallVec::new(),
        }
    }

    #[inline]
    pub fn push_def_or_use(&mut self, inst: InstId, opcode: Opcode) {
        if opcode == Opcode::Label {
            self.push_def(inst);
        } else {
            self.push_use(inst);
        }
    }

    #[inline]
    pub fn push_def(&mut self, inst: InstId) {
        self.defs.push(inst);
    }

    #[inline]
    pub fn push_use(&mut self, inst: InstId) {
        self.uses.push(inst);
    }
}

#[derive(Clone, Debug, Default)]
pub struct LabelResolver {
    labels: Vec<LabelData>,
    bits_map: HashMap<BitVec, LabelId>,
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

impl LabelResolver {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        LabelResolver::default()
    }

    #[must_use]
    pub fn resolve_all(&mut self, insts: Vec<RawInst>, order: LabelOrder) -> Vec<ProgramInst> {
        match order {
            LabelOrder::Def => {
                assert_eq_size!(RawInst, Option<RawInst>);
                assert_eq_size!(Vec<RawInst>, Vec<Option<RawInst>>);
                // SAFETY: RawInst and Option<RawInst> have the same structure,
                // so it is safe to cast from Vec<RawInst> to
                // Vec<Option<RawInst>>.
                let mut insts: Vec<Option<RawInst>> = unsafe {
                    // See https://doc.rust-lang.org/std/mem/fn.transmute.html#alternatives
                    let mut insts = ManuallyDrop::new(insts);
                    Vec::from_raw_parts(
                        insts.as_mut_ptr() as *mut Option<RawInst>,
                        insts.len(),
                        insts.capacity(),
                    )
                };

                // Iterate insts twice: the first time to resolve label
                // definitions; the second to resolve label uses and map other
                // instructions.
                let len = insts.len();
                let mut resolved = Vec::with_capacity(len);
                let resolved_uninit = resolved.spare_capacity_mut();
                for i in 0..len {
                    // SAFETY: All Option values from i..len are Some(..).
                    let inst = &mut insts[i];
                    if let Inst::Label(_) = unsafe { inst.as_ref().unwrap_unchecked() } {
                        let inst = unsafe { inst.take().unwrap_unchecked() };
                        resolved_uninit[i] = MaybeUninit::new(self.resolve(inst, InstId::from(i)));
                    }
                }
                for (i, inst) in insts.into_iter().enumerate() {
                    if let Some(inst) = inst {
                        resolved_uninit[i] = MaybeUninit::new(self.resolve(inst, InstId::from(i)));
                    }
                }
                // SAFETY: The entire vector has now been initialized.
                unsafe { resolved.set_len(len) };
                resolved
            }
            LabelOrder::DefOrUse => {
                // Resolve labels in order of the first definition or use.
                let mut resolved = Vec::with_capacity(insts.len());
                for (i, inst) in insts.into_iter().enumerate() {
                    resolved.push(self.resolve(inst, InstId::from(i)));
                }
                resolved
            }
        }
    }

    #[must_use]
    pub fn resolve(&mut self, inst: RawInst, id: InstId) -> ProgramInst {
        inst.map_arg(|opcode, arg| -> Result<_, InstError> {
            match arg {
                InstArg::Int(n) => Ok(InstArg::Int(IntLiteral::from(n))),
                InstArg::Label(l) => Ok(InstArg::Label(self.insert(l, id, opcode))),
            }
        })
    }

    fn insert(&mut self, bits: BitVec, inst: InstId, opcode: Opcode) -> LabelId {
        match self.bits_map.entry(bits.clone()) {
            Entry::Occupied(entry) => {
                let id = *entry.get();
                self.labels[id].push_def_or_use(inst, opcode);
                id
            }
            Entry::Vacant(entry) => {
                let id = LabelId::from(self.labels.len());
                let mut label = LabelData::new(id, bits);
                label.push_def_or_use(inst, opcode);
                self.labels.push(label);
                entry.insert(id);
                id
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use static_assertions::{assert_eq_size, const_assert};

    use super::*;

    assert_eq_size!(Vec<InstId>, SmallVec<[InstId; 4]>);
    const_assert!(size_of::<SmallVec<[InstId; 4]>>() < size_of::<SmallVec<[InstId; 5]>>());
}
