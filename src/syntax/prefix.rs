// Copyright (C) 2022 Thalia Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};

use crate::syntax::{TokenSeq, VariantIndex};
use crate::text::EncodingError;

#[derive(Clone)]
pub struct PrefixTable<T, O> {
    dense: Box<[Option<PrefixEntry<O>>]>,
    sparse: HashMap<TokenSeq<T>, Option<PrefixEntry<O>>>,
}

#[derive(Clone, Debug)]
pub enum PrefixEntry<O> {
    Terminal(O),
    Prefix(Vec<O>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictError<T: VariantIndex, O> {
    prefix: TokenSeq<T>,
    opcodes: Vec<O>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrefixError<T: VariantIndex, O> {
    EncodingError(EncodingError, TokenSeq<T>),
    UnknownOpcode(TokenSeq<T>),
    IncompleteOpcode(TokenSeq<T>, Vec<O>),
}

impl<T, O> PrefixTable<T, O>
where
    T: VariantIndex,
    O: Copy,
{
    #[inline]
    #[must_use]
    pub fn new(dense_len: usize) -> Self {
        PrefixTable {
            dense: vec![None; dense_len].into_boxed_slice(),
            sparse: HashMap::new(),
        }
    }

    #[inline]
    #[must_use]
    pub fn with_dense_width(width: usize) -> Self {
        Self::new(TokenSeq::<T>::size_for(width))
    }

    #[inline]
    #[must_use]
    pub fn get(&self, seq: TokenSeq<T>) -> Option<&PrefixEntry<O>> {
        if seq.as_usize() < self.dense.len() {
            self.dense[seq.as_usize()].as_ref()
        } else {
            self.sparse.get(&seq).and_then(Option::as_ref)
        }
    }

    #[inline]
    #[must_use]
    pub fn get_mut(&mut self, seq: TokenSeq<T>) -> &mut Option<PrefixEntry<O>> {
        if seq.as_usize() < self.dense.len() {
            &mut self.dense[seq.as_usize()]
        } else {
            self.sparse.entry(seq).or_default()
        }
    }

    pub fn insert(&mut self, toks: &[T], opcode: O) -> Result<(), ConflictError<T, O>> {
        let mut seq = TokenSeq::new();
        for tok in toks {
            let entry = self.get_mut(seq);
            match entry {
                Some(PrefixEntry::Terminal(terminal)) => {
                    return Err(ConflictError::new(seq, vec![*terminal, opcode]));
                }
                Some(PrefixEntry::Prefix(opcodes)) => opcodes.push(opcode),
                None => *entry = Some(PrefixEntry::Prefix(vec![opcode])),
            }
            seq.push(tok);
        }
        let entry = self.get_mut(seq);
        match entry {
            Some(PrefixEntry::Terminal(terminal)) => {
                return Err(ConflictError::new(seq, vec![*terminal, opcode]));
            }
            Some(PrefixEntry::Prefix(opcodes)) => {
                let mut opcodes = opcodes.clone();
                opcodes.push(opcode);
                return Err(ConflictError::new(seq, opcodes));
            }
            None => *entry = Some(PrefixEntry::Terminal(opcode)),
        }
        Ok(())
    }

    #[inline]
    #[must_use]
    pub fn parse<L>(&self, lex: &mut L) -> Option<Result<O, PrefixError<T, O>>>
    where
        L: Iterator<Item = Result<T, EncodingError>>,
    {
        self.parse_at(lex, TokenSeq::new())
    }

    #[must_use]
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
                Some(Ok(tok)) => seq.push(&tok),
                Some(Err(err)) => return Some(Err(PrefixError::EncodingError(err, seq))),
                None if seq.is_empty() => return None,
                None => {
                    let prefix = match self.get(seq) {
                        Some(PrefixEntry::Terminal(opcode)) => return Some(Ok(*opcode)),
                        Some(PrefixEntry::Prefix(opcodes)) => opcodes.clone(),
                        None => Vec::new(),
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

impl<T, O> PrefixTable<T, O>
where
    T: Debug + VariantIndex + 'static,
    O: Copy + Debug + Tokens<Token = T> + VariantIndex,
{
    pub fn insert_all(&mut self) {
        for opcode in O::iter() {
            self.insert(opcode.tokens(), opcode).unwrap();
        }
    }
}

impl<T, O> Debug for PrefixTable<T, O>
where
    T: Debug + VariantIndex,
    O: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        struct EntryDebug<'a, T, O>(TokenSeq<T>, &'a Option<PrefixEntry<O>>);
        impl<T: Debug + VariantIndex, O: Debug> Debug for EntryDebug<'_, T, O> {
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

impl<T: VariantIndex, O> ConflictError<T, O> {
    #[inline]
    #[must_use]
    const fn new(prefix: TokenSeq<T>, opcodes: Vec<O>) -> Self {
        ConflictError { prefix, opcodes }
    }
}

pub trait Tokens {
    type Token;

    #[must_use]
    fn tokens(&self) -> &'static [Self::Token];
}
