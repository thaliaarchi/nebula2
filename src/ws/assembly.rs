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
                jump_zero, jmp_zero, jm_zero, jp_zero, j_zero, branch_zero, br_zero, b_zero, goto_zero,
                jump_null, jmp_null, jm_null, jp_null, j_null, branch_null, br_null, b_null, goto_null,
                jump_zer, jmp_zer, jm_zer, jp_zer, j_zer, branch_zer, br_zer, b_zer, goto_zer,
                jump_nil, jmp_nil, jm_nil, jp_nil, j_nil, branch_nil, br_nil, b_nil, goto_nil,
                jump_ze, jmp_ze, jm_ze, jp_ze, j_ze, branch_ze, br_ze, b_ze, goto_ze,
                jump_ez, jmp_ez, jm_ez, jp_ez, j_ez, branch_ez, br_ez, b_ez, goto_ez,
                jump_z, jmp_z, jm_z, jp_z, j_z, branch_z, br_z, b_z, goto_z,
                jump_0, jmp_0, jm_0, jp_0, j_0, branch_0, br_0, b_0, goto_0,
                jump_if_zero, jmp_if_zero, branch_if_zero, goto_if_zero,
                jump_if_0, jmp_if_0, branch_if_0, goto_if_0,
                jump_if0, jmp_if0, branch_if0, goto_if0,
                jump_i_z, jmp_i_z, branch_i_z, goto_i_z,
                zero,
            ],
            Jn: [
                jump_negative, jmp_negative, branch_negative, goto_negative,
                jump_nega, jmp_nega, jm_nega, jp_nega, j_nega, branch_nega, br_nega, b_nega, goto_nega,
                jump_neg, jmp_neg, jm_neg, jp_neg, j_neg, branch_neg, br_neg, b_neg, goto_neg,
                jump_ltz, jmp_ltz, jm_ltz, jp_ltz, j_ltz, branch_ltz, br_ltz, b_ltz, goto_ltz,
                jump_lz, jmp_lz, jm_lz, jp_lz, j_lz, branch_lz, br_lz, b_lz, goto_lz,
                jump_l0, jmp_l0, jm_l0, jp_l0, j_l0, branch_l0, br_l0, b_l0, goto_l0,
                jump_ne, jmp_ne, jm_ne, jp_ne, j_ne, branch_ne, br_ne, b_ne, goto_ne,
                jump_n, jmp_n, jm_n, jp_n, j_n, branch_n, br_n, b_n, goto_n,
                jump_if_negative, jmp_if_negative, branch_if_negative, goto_if_negative,
                jump_if_neg, jmp_if_neg, branch_if_neg, goto_if_neg,
                jump_if_n, jmp_if_n, branch_if_n, goto_if_n,
                jump_i_n, jmp_i_n, branch_i_n, goto_i_n,
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
                print_character, print_char, print_chr, print_ch, print_c,
                prt_chr, prt_ch, prt_c, p_char, p_chr, p_ch, p_c,
                put_char, put_chr, put_ch, put_c,
                output_character, output_char, output_chr, output_ch, output_c,
                out_character, out_char, out_chr, out_ch, out_c,
                ot_c, o_char, o_chr, o_ch, o_c,
                char_out, chr_out, ch_out, c_out,
                write_character, write_char, write_chr, write_ch, write_c,
                w_char, w_chr, w_ch, w_c, wr_c, wt_c,
            ],
            Printi: [
                print_integer, print_int, print_i,
                print_number, print_num, print_n,
                prt_int, prt_i, p_int, p_i,
                prt_num, prt_n, p_num, p_n,
                put_int, put_i,
                put_num, put_n,
                out_integer, out_int, out_i, ot_i, o_int, o_i,
                out_number, out_num, out_n, ot_n, o_num, o_n,
                output_integer, output_int, output_i,
                output_number, output_num, output_n,
                int_out, i_out,
                num_out, n_out,
                write_integer, write_int, write_i, wr_i, wt_i, w_int, w_i,
                write_number, write_num, write_n, wr_n, wt_n, w_num, w_n,
            ],
            Readc: [
                read_character, read_char, read_chr, read_ch, read_c,
                red_c, re_c, rd_c, r_char, r_chr, r_ch, r_c,
                get_char, get_chr, get_ch, get_c,
                in_char, in_chr, in_ch, in_c,
                i_char, i_chr, i_ch, i_c,
                char_in, chr_in, ch_in, c_in,
                input_char, input_chr, input_ch, input_c, inp_c,
            ],
            Readi: [
                read_integer, read_int, read_i,
                read_number, read_num, read_n,
                red_i, re_i, rd_i, r_int, r_i,
                red_n, re_n, rd_n, r_num, r_n,
                get_integer, get_int, get_i,
                get_number, get_num, get_n,
                in_int, in_i, i_int,
                in_num, in_n, i_num,
                int_in, i_in,
                num_in, n_in,
                input_integer, input_int, input_i, inp_i,
                input_number, input_num, input_n, inp_n,
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
