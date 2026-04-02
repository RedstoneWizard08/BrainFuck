pub mod mov;
pub mod syscall;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Insn {
    Mov(mov::MovInsn),
    Syscall(syscall::SyscallInsn),
}

impl Insn {
    pub const fn predict_size(&self) -> usize {
        match self {
            Insn::Mov(it) => it.predict_size(),
            Insn::Syscall(it) => it.predict_size(),
        }
    }

    pub fn encode(self) -> Vec<u8> {
        match self {
            Insn::Mov(it) => it.encode(),
            Insn::Syscall(it) => it.encode(),
        }
    }
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

const fn encode_rex(a: Reg, b: RegDataRef) -> u8 {
    let res = 0b01000000;

    let w = 1 << 3;

    let r = if b.is_value() {
        0
    } else {
        (a.id_bits() & 0b1000) >> 1
    };

    let x = 0 >> 2;
    let b = (b.id_bits() & 0b1000) >> 3;

    let wrxb = w | r | x | b;

    res | wrxb
}

const fn modrm(b: RegDataRef) -> u8 {
    match b {
        RegDataRef::Direct(_)
        | RegDataRef::Value8(_)
        | RegDataRef::Value16(_)
        | RegDataRef::Value32(_)
        | RegDataRef::Value64(_) => 0b11,
        RegDataRef::RegOffset8(_, _) => 0b01,
        RegDataRef::RegOffset32(_, _) => 0b10,
    }
}

const fn encode_modrm(a: Reg, b: RegDataRef) -> u8 {
    let mod_ = modrm(b) << 6;
    let mut reg = a.id_bits() & 0b111;
    let mut rm = b.id_bits() & 0b111;

    if b.is_value() {
        std::mem::swap(&mut reg, &mut rm);
    }

    reg <<= 3;

    mod_ | reg | rm
}

pub fn encode_insn(op: u8, a: Reg, mut b: RegDataRef, skip_modrm: bool) -> Vec<u8> {
    b.simplify();

    let needs_rex = a.needs_64() || b.needs_64() || a.bit_width() == 64 || b.bit_width() == 64;
    let mut buf = Vec::new();

    if needs_rex {
        buf.push(encode_rex(a, b));
    }

    buf.push(op);

    if !skip_modrm {
        buf.push(encode_modrm(a, b));
    }

    match b {
        RegDataRef::Direct(_) => {}
        RegDataRef::RegOffset8(_, o) => buf.push(o),
        RegDataRef::RegOffset32(_, o) => buf.extend(o.to_le_bytes()),
        RegDataRef::Value8(v) => buf.push(v),
        RegDataRef::Value16(v) => buf.extend(v.to_le_bytes()),
        RegDataRef::Value32(v) => buf.extend(v.to_le_bytes()),
        RegDataRef::Value64(v) => buf.extend(v.to_le_bytes()),
    };

    buf
}
