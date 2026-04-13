//! Register definitions and utilities.
//!
//! This module defines all available x86-64 registers using the registers!
//! procedural macro, providing methods for querying register properties.

use asmbin_macros::registers;

registers! {
    0 = a & [seg16 = es; st];
    1 = c & [seg16 = cs; st];
    2 = d & [seg16 = ss; st];
    3 = b & [seg16 = ds; st];

    4 = {
        gp8 = rex ? spl : ah;
        gp.. = [sp, esp, rsp];
        st = true;
        seg16 = fs;
    };

    5 = {
        gp8 = rex ? bpl : ch;
        gp.. = [bp, ebp, rbp];
        st = true;
        seg16 = gs;
    };

    6 = {
        gp8 = rex ? sil : dh;
        gp.. = [si, esi, rsi];
        st = true;
    };

    7 = {
        gp8 = rex ? dil : bh;
        gp.. = [di, edi, rdi];
        st = true;
    };

    8 = #r & [seg16 = es; mmx = 0];
    9 = #r & [seg16 = cs; mmx = 1];
    10 = #r & [seg16 = ss; mmx = 2];
    11 = #r & [seg16 = ds; mmx = 3];
    12 = #r & [seg16 = fs; mmx = 4];
    13 = #r & [seg16 = gs; mmx = 5];
    14 = #r & [mmx = 6];
    15 = #r & [mmx = 7];
}

impl Reg {
    #[inline(always)]
    pub const fn needs_64(&self) -> bool {
        match self.prefix() {
            Prefix::Rex => true,
            _ => false,
        }
    }

    #[inline(always)]
    pub const fn needs_rex(&self) -> bool {
        self.needs_64() || self.bit_width() == 64
    }
}
