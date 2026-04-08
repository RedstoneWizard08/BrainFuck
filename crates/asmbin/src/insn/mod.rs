pub mod add;
pub mod cmp;
pub mod dec;
pub mod imul;
pub mod inc;
pub mod jmp;
pub mod lea;
pub mod mov;
pub mod repne;
pub mod sub;
pub mod syscall;
pub mod xor;

macro_rules! insn_wrapper {
    ($($insn: ident),* $(,)?) => {
        pastey::paste! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub enum Insn {
                $([<$insn:upper_camel>]($insn::[<$insn:upper_camel Insn>])),*
            }

            impl const InsnInfo for Insn {
                fn predict_size(&self) -> usize {
                    match self {
                        $(Self::[<$insn:upper_camel>](it) => it.predict_size()),*
                    }
                }
            }

            impl InsnEncode for Insn {
                fn encode(self) -> Vec<u8> {
                    match self {
                        $(Self::[<$insn:upper_camel>](it) => it.encode()),*
                    }
                }
            }

            $(
                impl From<$insn::[<$insn:upper_camel Insn>]> for Insn {
                    fn from(insn: $insn::[<$insn:upper_camel Insn>]) -> Insn {
                        Insn::[<$insn:upper_camel>](insn)
                    }
                }
            )*
        }
    };
}

insn_wrapper! {
    add,
    cmp,
    dec,
    imul,
    inc,
    jmp,
    lea,
    mov,
    repne,
    sub,
    syscall,
    xor,
}

pub const trait InsnInfo {
    fn predict_size(&self) -> usize;
}

pub trait InsnEncode: InsnInfo {
    fn encode(self) -> Vec<u8>;
}

// rdi = 0.111
// r8 = 1.000

// reg = 000
// r/m = 111
// REX.B = 0 (from `rdi`)
// REX.R = 1 (from `r8`)

// r8 -> REX.R [.] reg
// rdi -> REX.B [.] r/m

use crate::{data::RegDataRef, reg::Reg};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModRm {
    pub mod_: u8,
    pub reg: u8,
    pub rm: u8,
}

impl ModRm {
    pub const fn encode(&self) -> u8 {
        let mod_ = (self.mod_ & 0b11) << 6;
        let reg = (self.reg & 0b111) << 3;
        let rm = self.rm & 0b111;

        mod_ | reg | rm
    }
}

pub fn encode_rex(a: &Option<Reg>, b: &Option<RegDataRef>) -> u8 {
    let res = 0b01000000;
    let w = 1 << 3;

    let r = if b.is_some_and(|it| it.is_value()) {
        0
    } else {
        (a.as_ref().map(|it| it.id_bits()).unwrap_or(0) & 0b1000) >> 1
    };

    // I don't think this is ever used lol
    let x = 0 >> 2;
    let b = (b.as_ref().map(|it| it.id_bits()).unwrap_or(0) & 0b1000) >> 3;

    let wrxb = w | r | x | b;

    res | wrxb
}

pub const fn encode_rex_for_reg(reg: Reg) -> u8 {
    let res = 0b01000000;
    let w = 1 << 3;
    let r = (reg.id_bits() & 0b1000) >> 1;
    let x = 0 >> 2;
    let b = 0;

    let wrxb = w | r | x | b;

    res | wrxb
}

pub const fn modrm(b: Option<RegDataRef>) -> u8 {
    match b {
        Some(
            RegDataRef::Direct(_)
            | RegDataRef::Value8(_)
            | RegDataRef::Value16(_)
            | RegDataRef::Value32(_)
            | RegDataRef::Value64(_),
        )
        | None => 0b11,

        Some(RegDataRef::RegOffset8(_, _)) => 0b01,
        Some(RegDataRef::RegOffset32(_, _)) => 0b10,

        Some(RegDataRef::DirectValue(_)) => 0b00,
    }
}

pub struct EncodeOpts {
    opcode: u8,
    reg: Reg,
    data: Option<RegDataRef>,
    skip_modrm: bool,

    /// Pass a custom value to modrm's reg field, moving the register to the rm
    /// field and excluding the data from it.
    modrm_reg: Option<u8>,
}

pub fn encode_insn(opcode: u8, reg: Reg, data: Option<RegDataRef>, skip_modrm: bool) -> Vec<u8> {
    encode_insn_with(EncodeOpts {
        opcode,
        reg,
        data,
        skip_modrm,
        modrm_reg: None,
    })
}

pub fn encode_insn_with(mut opts: EncodeOpts) -> Vec<u8> {
    opts.data = opts.data.map(|mut it| {
        it.simplify();
        it
    });

    let needs_rex = opts.reg.needs_rex() || opts.data.is_some_and(|it| it.needs_rex());
    let mut buf = Vec::new();

    if needs_rex {
        buf.push(encode_rex(&Some(opts.reg), &opts.data));
    }

    buf.push(opts.opcode);

    if !opts.skip_modrm {
        buf.push(
            if let Some(reg) = opts.modrm_reg {
                ModRm {
                    mod_: modrm(opts.data),
                    reg,
                    rm: opts.reg.id_bits(),
                }
            } else if opts.data.is_some_and(|it| it.is_value()) {
                ModRm {
                    mod_: modrm(opts.data),
                    reg: opts.data.as_ref().unwrap().id_bits(),
                    rm: opts.reg.id_bits(),
                }
            } else {
                ModRm {
                    mod_: modrm(opts.data),
                    reg: opts.reg.id_bits(),
                    rm: opts.data.map(|it| it.id_bits()).unwrap_or(0),
                }
            }
            .encode(),
        );
    }

    if let Some(data) = opts.data {
        buf.extend(data.extra_bytes());
    }

    buf
}
