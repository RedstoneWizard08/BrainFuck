//! Builder traits for conveniently constructing x86-64 instructions.

use crate::{
    buf::InsnBuf,
    data::RegDataRef,
    insn::{
        Insn,
        add::AddInsn,
        cmp::CmpInsn,
        dec::DecInsn,
        imul::ImulInsn,
        inc::IncInsn,
        jmp::{JmpCond, JmpInsn},
        lea::LeaInsn,
        mov::MovInsn,
        repne::{RepneInsn, Scan},
        sub::SubInsn,
        syscall::SyscallInsn,
        xor::XorInsn,
    },
    reg::Reg,
};

pub trait InsnRecv {
    fn push(&mut self, insn: impl Into<Insn>);
    fn extend(&mut self, insns: impl IntoIterator<Item = Insn>);
}

pub trait InsnBuilder: InsnRecv {
    fn mov_to_reg(&mut self, src: impl Into<RegDataRef>, dst: impl Into<RegDataRef>) {
        self.push(MovInsn::DataToReg(src.into(), dst.into()));
    }

    fn mov_byte_to_reg(&mut self, src: impl Into<RegDataRef>, dst: Reg) {
        self.push(MovInsn::ByteToReg(src.into(), dst));
    }

    fn mov_from_reg(&mut self, src: impl Into<RegDataRef>, dst: impl Into<RegDataRef>) {
        self.push(MovInsn::RegToData(src.into(), dst.into()));
    }

    fn lea(&mut self, src: impl Into<RegDataRef>, dst: Reg) {
        self.push(LeaInsn(src.into(), dst));
    }

    fn add(&mut self, reg: impl Into<RegDataRef>, data: impl Into<RegDataRef>) {
        self.push(AddInsn(reg.into(), data.into()));
    }

    fn sub(&mut self, reg: impl Into<RegDataRef>, data: impl Into<RegDataRef>) {
        self.push(SubInsn(reg.into(), data.into()));
    }

    fn inc(&mut self, target: impl Into<RegDataRef>) {
        self.push(IncInsn(target.into()));
    }

    fn dec(&mut self, target: impl Into<RegDataRef>) {
        self.push(DecInsn(target.into()));
    }

    fn xor(&mut self, a: Reg, b: impl Into<RegDataRef>) {
        self.push(XorInsn(a, b.into()));
    }

    fn cmp(&mut self, a: impl Into<RegDataRef>, b: impl Into<RegDataRef>) {
        self.push(CmpInsn(a.into(), b.into()));
    }

    fn jmp_rel8(&mut self, dist: i8) {
        self.push(JmpInsn::Short(dist));
    }

    fn jmp_rel32(&mut self, dist: i32) {
        self.push(JmpInsn::Near(dist));
    }

    fn jmp_indirect(&mut self, dst: impl Into<RegDataRef>) {
        self.push(JmpInsn::Indirect(dst.into()));
    }

    fn jmp_cond_rel8(&mut self, cond: JmpCond, dist: i8) {
        self.push(JmpInsn::Cond8(cond, dist));
    }

    fn jmp_cond_rel32(&mut self, cond: JmpCond, dist: i32) {
        self.push(JmpInsn::Cond32(cond, dist));
    }

    fn repne(&mut self, scan: Scan) {
        self.push(RepneInsn(scan));
    }

    fn imul_imm(&mut self, dst: Reg, src: impl Into<RegDataRef>, mul: impl Into<RegDataRef>) {
        self.push(ImulInsn::Immediate {
            dst,
            src: src.into(),
            mul: mul.into(),
        });
    }

    fn syscall(&mut self) {
        self.push(SyscallInsn);
    }
}

impl<T: InsnRecv> InsnBuilder for T {}

impl InsnRecv for InsnBuf {
    fn push(&mut self, insn: impl Into<Insn>) {
        self.buf.push(insn.into());
    }

    fn extend(&mut self, insns: impl IntoIterator<Item = Insn>) {
        self.buf.extend(insns);
    }
}
