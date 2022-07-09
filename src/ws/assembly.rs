// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::collections::HashMap;

use crate::ws::inst::Opcode;

macro_rules! mnemonics_map {
    (@insert $map:ident $opcode:ident [$($left:tt),+] * [$($right:tt),+], $($rest:tt)*) => {
        $map.insert_prod(
            Opcode::$opcode,
            &[$(mnemonics_map!(@stringify $left)),+],
            &[$(mnemonics_map!(@stringify $right)),+],
        );
        mnemonics_map!(@insert $map $opcode $($rest)*)
    };
    (@insert $map:ident $opcode:ident) => {};
    (@insert $map:ident $opcode:ident $mnemonic:tt,) => {
        $map.insert(Opcode::$opcode, mnemonics_map!(@stringify $mnemonic).to_owned());
    };
    (@insert $map:ident $opcode:ident $($mnemonic:tt),+,) => {
        $map.insert_all(Opcode::$opcode, &[$(mnemonics_map!(@stringify $mnemonic)),+]);
    };
    (@stringify $mnemonic:ident) => { stringify!($mnemonic) };
    (@stringify $mnemonic:literal) => { $mnemonic };
    ($($opcode:ident: [$($mnemonics:tt)+],)+) => {{
        let mut map = MnemonicMap::new();
        $(mnemonics_map!(@insert map $opcode $($mnemonics)*);)+
        map
    }};
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MnemonicMap {
    mnemonics: HashMap<String, Opcode>,
}

impl MnemonicMap {
    #[inline]
    pub fn new() -> Self {
        MnemonicMap { mnemonics: HashMap::new() }
    }

    pub fn insert(&mut self, opcode: Opcode, mnemonic: String) {
        if mnemonic.contains("_") {
            if let Err(err) = self.mnemonics.try_insert(mnemonic.replace("_", ""), opcode) {
                if *err.entry.get() != opcode {
                    panic!("{:?}", err);
                }
            }
        }
        if let Err(err) = self.mnemonics.try_insert(mnemonic, opcode) {
            if *err.entry.get() != opcode {
                panic!("{:?}", err);
            }
        }
    }

    pub fn insert_all(&mut self, opcode: Opcode, mnemonics: &[&str]) {
        for &mnemonic in mnemonics {
            self.insert(opcode, mnemonic.to_owned());
        }
    }

    pub fn insert_prod(&mut self, opcode: Opcode, first: &[&str], second: &[&str]) {
        for &first in first {
            for &second in second {
                self.insert(opcode, first.to_owned() + "_" + second);
            }
        }
    }

    pub fn with_permissive() -> Self {
        mnemonics_map! {
            Push: [
                push, psh, pus,
                push_number, push_num,
                push_char, push_ch,
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
                copy_n, copy_nth, copy_at,
                dup_n, dup_nth, dup_at,
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
                slide_n,
                slide_off,
                "<unsigned>slide",
            ],
            Add: [
                add,
                addition,
                adding,
                plus,
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
                integer_division, int_div,
                "/",
            ],
            Mod: [
                modulo, mod,
                remainder, rem,
                division_part,
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
                mark, mrk, mark_sub, mark_label, mark_location,
                defun, def, def_label,
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
                call_subroutine, call_sub, call_s, ca_s,
                j_sr,
                go_sub,
                subroutine,
            ],
            Jmp: [
                jump, jmp, jm, jp, j,
                branch, br, b,
                goto,
            ],
            Jz: [
                [jump, jmp, jm, jp, j, branch, br, b, goto] * [zero, zer, ze, z, null, nil, ez, "0"],
                [jump, jmp, branch, goto] * [if_zero, if_0, if0, i_z],
                zero,
            ],
            Jn: [
                [jump, jmp, jm, jp, j, branch, br, b, goto] * [negative, nega, neg, ne, n, ltz, lz, l0],
                [jump, jmp, branch, goto] * [if_negative, if_neg, if_n, i_n],
                negative,
            ],
            Ret: [
                return, ret, rt_s,
                end_subroutine, end_sub, end_s, en_s,
                subroutine_end, sub_end,
                end_function, end_func,
                exit_sub,
                control_back, back,
                leave,
            ],
            End: [
                end_program, end_prog, end_p, end,
                exit,
                halt, hlt,
                terminate,
                quit,
                die,
                finish_program, finish,
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
                dump_stack,
                debug_print_stack, debug_printstack,
            ],
            DumpHeap: [
                dump_heap,
                debug_print_heap, debug_printheap,
            ],
            DumpTrace: [
                dump_trace,
                trace,
            ],
        }
    }
}
