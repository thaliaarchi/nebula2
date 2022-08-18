// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::collections::HashMap;

use compact_str::{CompactString, ToCompactString};

use crate::ws::inst::Opcode;

macro_rules! mnemonics_map {
    (@insert $map:ident $opcode:ident [$($left:tt),+] * [$($right:tt),+], $($rest:tt)*) => {
        $map.insert_prod(
            Opcode::$opcode,
            &[$(mnemonics_map!(@stringify $left)),+],
            &[$(mnemonics_map!(@stringify $right)),+],
        ).unwrap();
        mnemonics_map!(@insert $map $opcode $($rest)*)
    };
    (@insert $map:ident $opcode:ident) => {};
    (@insert $map:ident $opcode:ident $mnemonic:tt,) => {
        $map.insert(Opcode::$opcode, mnemonics_map!(@stringify $mnemonic)).unwrap();
    };
    (@insert $map:ident $opcode:ident $($mnemonic:tt),+,) => {
        $map.insert_all(Opcode::$opcode, &[$(mnemonics_map!(@stringify $mnemonic)),+]).unwrap();
    };
    (@stringify $mnemonic:ident) => { stringify!($mnemonic) };
    (@stringify $mnemonic:literal) => { $mnemonic };
    ($($opcode:ident: [$($mnemonics:tt)+],)+) => {{
        let mut map = MnemonicMap::new();
        $(mnemonics_map!(@insert map $opcode $($mnemonics)*);)+
        map
    }};
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MnemonicMap {
    mnemonics: HashMap<CompactString, Opcode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MnemonicError {
    mnemonic: CompactString,
    old: Opcode,
    new: Opcode,
}

impl MnemonicMap {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        MnemonicMap::default()
    }

    pub fn insert_compact(
        &mut self,
        opcode: Opcode,
        mnemonic: CompactString,
    ) -> Result<(), MnemonicError> {
        if let Err(err) = self.mnemonics.try_insert(mnemonic, opcode) {
            let old = *err.entry.get();
            if old != opcode {
                return Err(MnemonicError {
                    mnemonic: err.entry.key().clone(),
                    old,
                    new: opcode,
                });
            }
        }
        Ok(())
    }

    #[inline]
    pub fn insert(&mut self, opcode: Opcode, mnemonic: &str) -> Result<(), MnemonicError> {
        self.insert_compact(opcode, mnemonic.to_compact_string())
    }

    #[inline]
    pub fn insert_all(&mut self, opcode: Opcode, mnemonics: &[&str]) -> Result<(), MnemonicError> {
        for &mnemonic in mnemonics {
            self.insert(opcode, mnemonic)?;
        }
        Ok(())
    }

    #[inline]
    pub fn insert_prod(
        &mut self,
        opcode: Opcode,
        first: &[&str],
        second: &[&str],
    ) -> Result<(), MnemonicError> {
        for &first in first {
            for &second in second {
                self.insert_compact(opcode, first.to_compact_string() + second)?;
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn with_permissive() -> Self {
        mnemonics_map! {
            Push: [
                push, psh, pus,
                pushnumber, pushnum,
                pushchar, pushch,
                append,
                "<number>", "<char>",
            ],
            Dup: [
                duplicate, dupli, dupl, dup,
                dupe,
                doub,
                "^",
            ],
            Copy: [
                copy,
                copyn, copynth, copyat,
                dupn, dupnth, dupat,
                duplicaten, duplicatenth, duplicateat,
                pick,
                ref,
                take,
                pull,
                "^<number>",
            ],
            Swap: [
                swap, swp, swa,
                exchange, exch, xchg,
                switch,
            ],
            Drop: [
                drop,
                discard, disc, dsc,
                pop,
                away,
                delete, del,
            ],
            Slide: [
                slide, slid,
                sliden,
                slideoff,
                "<unsigned>slide",
            ],
            Add: [
                add,
                addition,
                adding,
                plus,
                sum,
                "+",
            ],
            Sub: [
                subtract, subt, sub,
                subtraction,
                minus,
                "-",
            ],
            Mul: [
                multiply, multi, mult, mul,
                multiplication,
                multiple,
                times,
                "*",
            ],
            Div: [
                divide, div,
                division,
                integerdivision, intdiv,
                "/",
            ],
            Mod: [
                modulo, mod,
                remainder, rem,
                divisionpart,
                "%",
            ],
            Store: [
                store, stor, sto, st,
                set,
                put,
            ],
            Retrieve: [
                retrieve, retrive, retri, retrv, retr, reti,
                load, lod, ld,
                fetch,
                get,
                recall, rcl,
            ],
            Label: [
                label, lbl,
                mark, mrk, marksub, marklabel, marklocation,
                defun, def, deflabel,
                part,
                "<label>:",
                "%<label>:",
                "@<label>",
                "<<label>>:",
                "L<number>:",
                "label_<number>:", "label_<number>",
            ],
            Call: [
                call, cll,
                callsubroutine, callsub, calls, cas,
                jsr,
                gosub,
                subroutine,
            ],
            Jmp: [
                jump, jmp, jm, jp, j,
                branch, br, b,
                goto,
            ],
            Jz: [
                [jump, jmp, jm, jp, j, branch, br, b, goto] * [zero, zer, ze, z, null, nil, ez, "0"],
                [jump, jmp, branch, goto] * [ifzero, if0, iz],
                zero,
            ],
            Jn: [
                [jump, jmp, jm, jp, j, branch, br, b, goto] * [negative, nega, neg, ne, n, ltz, lz, l0],
                [jump, jmp, branch, goto] * [ifnegative, ifneg, ifn, in],
                negative,
            ],
            Ret: [
                return, ret, rts,
                endsubroutine, endsub, ends, ens,
                subroutineend, subend,
                endfunction, endfunc,
                exitsub,
                controlback, back,
                leave,
            ],
            End: [
                endprogram, endprog, endp, end,
                exit,
                halt, hlt,
                terminate,
                quit,
                die,
                finishprogram, finish,
            ],
            Printc: [
                [print, output, out, write] * [character, char, chr, ch, c],
                [put, p, o, w] * [char, chr, ch, c],
                [prt, ot, wr, wt] * [chr, ch, c],
                [char, chr, ch, c] * [out],
            ],
            Printi: [
                [print, output, out, write] * [integer, int, i, number, num, n],
                [put, prt, p, o, w] * [int, i, num, n],
                [ot, wr, wt] * [i, n],
                [int, i, num, n] * [out],
            ],
            Readc: [
                [read, input, get] * [character, char, chr, ch, c],
                [in, r, i] * [char, chr, ch, c],
                [red, re, rd, inp] * [chr, ch, c],
                [char, chr, ch, c] * [in],
            ],
            Readi: [
                [read, input, get] * [integer, int, i, number, num, n],
                [red, re, rd, r, inp, in] * [int, i, num, n],
                [i] * [int, i, num],
                [int, i, num, n] * [in],
            ],
            Shuffle: [
                shuffle,
                permr,
            ],
            DumpStack: [
                dumpstack,
                debugprintstack,
            ],
            DumpHeap: [
                dumpheap,
                debugprintheap,
            ],
            DumpTrace: [
                dumptrace,
                trace,
            ],
        }
    }
}
