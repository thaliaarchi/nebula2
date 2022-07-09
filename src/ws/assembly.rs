// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::collections::HashMap;

use crate::ws::inst::Opcode;

macro_rules! mnemonics {
    (@mnemonic $mnemonic:ident) => { stringify!($mnemonic) };
    (@mnemonic $mnemonic:literal) => { $mnemonic };
    ($($opcode:ident: [$($mnemonic:tt),+,],)+) => {{
        let mut mnemonics = MnemonicMap::new();
        $($(mnemonics.insert(mnemonics!(@mnemonic $mnemonic).to_owned(), Opcode::$opcode);)+)+
        mnemonics
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

    pub fn insert(&mut self, mnemonic: String, opcode: Opcode) {
        // TODO: Normalize
        self.mnemonics.insert(mnemonic, opcode);
    }

    pub fn with_permissive() -> Self {
        mnemonics! {
            Push: [
                push, psh, pus,
                push_number, push_num,
                push_char, push_ch,
                append,
                "<number>", "<char>",
            ],
            Dup: [
                dup, duplicate, dupli, dupl,
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
                xchg, exchange, exch,
                switch,
            ],
            Drop: [
                drop,
                discard, disc, dsc,
                pop,
                away,
                del, delete,
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
                sub, subtract, subt,
                subtraction,
                minus,
                "-",
            ],
            Mul: [
                mul, multiply, multi, mult,
                multiplication,
                multiple,
                times,
                "*",
            ],
            Div: [
                div, divide,
                division,
                int_div, integer_division,
                "/",
            ],
            Mod: [
                mod, modulo,
                rem, remainder,
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
                rcl,
            ],
            Label: [
                label, lbl,
                mark, mrk,
                defun, def,
                part,
                mark_sub, marks,
                mark_label,
                mark_location,
                def_label,
                "<label>:",
                "%<label>:",
                "@<label>",
                "<<label>>:",
                "L<number>:",
                "label_<number>:",
                "label_<number>",
            ],
            Call: [
                call, cll,
                call_sub, call_subroutine, calls, cas,
                call_label,
                jsr,
                go_sub,
                subroutine,
            ],
            Jmp: [
                jmp, jump, jp, j,
                go_to,
                b,
                jumps,
                jump_label,
                unconditional_jump,
            ],
            Jz: [
                jz, jumpz, jmpz, jpz, jmz,
                jump_if_zero, jump_zero, jzero, jzer, jze,
                jez,
                jmp_if0, jp0,
                zero,
                jump_null,
                jnil,
                branchz, branchzs, brz, bz,
                bzero,
                gotoiz,
                if0goto,
            ],
            Jn: [
                jn, jumpn, jmpn, jmn, jpn,
                jump_negative, jump_nega, jump_neg, jmp_neg, jneg, jne,
                jump_if_negative, jump_if_neg,
                jumplz, jltz, jlz,
                jpl0,
                gotoin,
                bneg,
                branchltz, branchltzs, bltz,
            ],
            Ret: [
                ret, return,
                end_subroutine, subroutine_end, end_sub, ends, ens,
                end_func,
                exit_sub,
                control_back, back,
                leave,
                rts,
            ],
            End: [
                end, end_program, end_prog, endp,
                exit,
                halt, hlt,
                terminate,
                quit,
                die,
                finish, finish_program,
            ],
            Printc: [
                printc, print_char, print_c, prtc, pchr, pc,
                put_char, putc, pchar,
                out_character, out_char, out_ch, outc, otc,
                output_character, output_char, output_c,
                o_char, o_chr,
                char_out, cout,
                write_character, write_char, write_ch, write_c, w_char, wtc, wrc,
            ],
            Printi: [
                printi, print_int, print_i,
                print_number, print_num, print_n, prtn, pnum, pn,
                out_integer, out_int, out_i, o_int,
                out_num, out_n, o_num, otn,
                put_int, puti,
                put_num, putn, pnum,
                output_number, output_num, output_n,
                write_number, write_num, writen, wnum, wrn, wtn,
                write_int,
                num_out, nout,
                iout,
            ],
            Readc: [
                readc, read_character, read_char, read_chr, read_ch, read_c, redc, rec, rdc,
                rchar, rchr, rc,
                get_char, getc,
                in_char, in_ch, in_c,
                i_char, i_chr,
                inpc,
                char_in, cin,
            ],
            Readi: [
                readi, read_integer, read_int, read_i,
                read_number, read_num, read_n, r_num, redn, ren, rdn, rn,
                get_int, get_i,
                get_num, get_n,
                in_int, i_int, in_i,
                in_num, i_num, in_n,
                int_in, i_in,
                num_in, n_in,
                inpn,
            ],
            Shuffle: [
                shuffle,
                permr,
            ],
            DumpStack: [
                dumpstack, dump_stack,
                debug_printstack, debug_print_stack,
            ],
            DumpHeap: [
                dumpheap, dump_heap,
                debug_printheap, debug_print_heap,
            ],
            DumpTrace: [
                dumptrace, dump_trace,
                trace,
            ],
        }
    }
}
