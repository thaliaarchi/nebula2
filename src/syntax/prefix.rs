// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};

use smallvec::{smallvec, SmallVec};

use crate::syntax::{EnumIndex, TokenSeq};
use crate::text::EncodingError;

#[derive(Clone)]
pub struct PrefixTable<T, O> {
    dense: Box<[Option<PrefixEntry<O>>]>,
    sparse: HashMap<TokenSeq<T>, Option<PrefixEntry<O>>>,
}

#[derive(Clone, Debug)]
pub enum PrefixEntry<O> {
    Terminal(O),
    Prefix(SmallVec<[O; 16]>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableError<T: Copy + EnumIndex, O> {
    Conflict(TokenSeq<T>, SmallVec<[O; 16]>),
    NoTokens(O),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrefixError<T: Copy + EnumIndex, O> {
    EncodingError(EncodingError, TokenSeq<T>),
    UnknownOpcode(TokenSeq<T>),
    IncompleteOpcode(TokenSeq<T>, SmallVec<[O; 16]>),
}

impl<T, O> PrefixTable<T, O>
where
    T: Copy + EnumIndex + Eq,
    O: Copy,
{
    #[inline]
    pub fn new(dense_len: usize) -> Self {
        PrefixTable {
            dense: vec![None; dense_len].into_boxed_slice(),
            sparse: HashMap::new(),
        }
    }

    #[inline]
    pub fn with_dense_width(width: usize) -> Self {
        Self::new(TokenSeq::<T>::size_for(width))
    }

    #[inline]
    pub fn get(&self, seq: TokenSeq<T>) -> Option<&PrefixEntry<O>> {
        if seq.as_usize() < self.dense.len() {
            self.dense[seq.as_usize()].as_ref()
        } else {
            self.sparse.get(&seq).map(Option::as_ref).flatten()
        }
    }

    #[inline]
    pub fn get_mut(&mut self, seq: TokenSeq<T>) -> &mut Option<PrefixEntry<O>> {
        if seq.as_usize() < self.dense.len() {
            &mut self.dense[seq.as_usize()]
        } else {
            self.sparse.entry(seq).or_default()
        }
    }

    pub fn insert(&mut self, toks: &[T], opcode: O) -> Result<(), TableError<T, O>> {
        if toks.len() == 0 {
            return Err(TableError::NoTokens(opcode));
        }
        let mut seq = TokenSeq::new();
        for &tok in toks {
            let entry = self.get_mut(seq);
            match entry {
                Some(PrefixEntry::Terminal(terminal)) => {
                    return Err(TableError::Conflict(seq, smallvec![*terminal, opcode]));
                }
                Some(PrefixEntry::Prefix(opcodes)) => opcodes.push(opcode),
                None => *entry = Some(PrefixEntry::Prefix(smallvec![opcode])),
            }
            seq.push(tok);
        }
        let entry = self.get_mut(seq);
        match entry {
            Some(PrefixEntry::Terminal(terminal)) => {
                return Err(TableError::Conflict(seq, smallvec![*terminal, opcode]));
            }
            Some(PrefixEntry::Prefix(opcodes)) => {
                let mut opcodes = opcodes.clone();
                opcodes.push(opcode);
                return Err(TableError::Conflict(seq, opcodes));
            }
            None => *entry = Some(PrefixEntry::Terminal(opcode)),
        }
        Ok(())
    }

    pub fn parse<L>(&self, lex: &mut L) -> Option<Result<O, PrefixError<T, O>>>
    where
        L: Iterator<Item = Result<T, EncodingError>>,
    {
        self.parse_at(lex, TokenSeq::new())
    }

    pub fn parse_at<L>(
        &self,
        lex: &mut L,
        partial: TokenSeq<T>,
    ) -> Option<Result<O, PrefixError<T, O>>>
    where
        L: Iterator<Item = Result<T, EncodingError>>,
    {
        let mut seq = partial;
        loop {
            match lex.next() {
                Some(Ok(tok)) => seq.push(tok),
                Some(Err(err)) => return Some(Err(PrefixError::EncodingError(err, seq))),
                None if seq.is_empty() => return None,
                None => {
                    let prefix = match self.get(seq) {
                        Some(PrefixEntry::Terminal(opcode)) => return Some(Ok(*opcode)),
                        Some(PrefixEntry::Prefix(opcodes)) => opcodes.clone(),
                        None => SmallVec::new(),
                    };
                    return Some(Err(PrefixError::IncompleteOpcode(seq, prefix)));
                }
            }
            match self.get(seq) {
                Some(PrefixEntry::Terminal(opcode)) => return Some(Ok(*opcode)),
                Some(PrefixEntry::Prefix(_)) => {}
                None => return Some(Err(PrefixError::UnknownOpcode(seq))),
            }
        }
    }
}

impl<T, O> Debug for PrefixTable<T, O>
where
    T: Copy + Debug + EnumIndex + Ord,
    O: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        struct EntryDebug<'a, T, O>(TokenSeq<T>, &'a Option<PrefixEntry<O>>);
        impl<'a, T: Copy + Debug + EnumIndex, O: Debug> Debug for EntryDebug<'a, T, O> {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}: {:?}", self.0, self.1)
            }
        }

        let dense = self
            .dense
            .iter()
            .enumerate()
            .map(|(i, e)| EntryDebug(TokenSeq::<T>::from(i), e))
            .collect::<Vec<_>>();
        let mut sparse = self
            .sparse
            .iter()
            .map(|(&seq, e)| EntryDebug(seq, e))
            .collect::<Vec<_>>();
        sparse.sort_by(|a, b| a.0.cmp(&b.0));

        f.debug_struct("PrefixTable")
            .field("dense", &dense)
            .field("dense.len", &self.dense.len())
            .field("sparse", &sparse)
            .field("sparse.len", &self.sparse.len())
            .field("sparse.capacity", &self.sparse.capacity())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use static_assertions::{assert_eq_size, const_assert};

    use super::*;
    use crate::ws::inst::Opcode;

    #[test]
    fn optimal_size() {
        #[allow(dead_code)]
        enum PrefixEntryOf<T, P> {
            Terminal(T),
            Prefix(P),
        }

        assert_eq_size!(
            PrefixEntry<Opcode>,
            Option<PrefixEntry<Opcode>>,
            PrefixEntryOf<Opcode, Vec<Opcode>>,
            PrefixEntryOf<Opcode, SmallVec<[Opcode; 16]>>,
        );
        assert_eq_size!(Vec<Opcode>, SmallVec<[Opcode; 16]>);
        const_assert!(size_of::<SmallVec<[Opcode; 16]>>() < size_of::<SmallVec<[Opcode; 17]>>());
    }
}
