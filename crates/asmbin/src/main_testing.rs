use asmbin::{
    data::RegDataRef,
    insn::{
        InsnEncode,
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

pub fn main() {
    // asmbin::example::hello_world_no_reloc().unwrap();

    let mut buf = Vec::new();

    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(69), Reg::Rax).encode());
    buf.extend(LeaInsn(RegDataRef::RegOffset32(Reg::Rbx, 24), Reg::Rax).encode());
    buf.extend(JmpInsn::Cond8(JmpCond::Equal, 24).encode());
    buf.extend(AddInsn(Reg::Rax, RegDataRef::Value32(24)).encode());
    buf.extend(CmpInsn(Reg::Rax, RegDataRef::Value32(24)).encode());
    buf.extend(SubInsn(Reg::Bl, RegDataRef::Value8(24)).encode());
    buf.extend(IncInsn(RegDataRef::RegOffset32(Reg::Rax, 32)).encode());
    buf.extend(DecInsn(RegDataRef::RegOffset32(Reg::Rax, 60)).encode());
    buf.extend(XorInsn(Reg::Rax, RegDataRef::RegOffset32(Reg::Rax, 60)).encode());
    buf.extend(RepneInsn(Scan::ScanStringByte).encode());
    buf.extend(SyscallInsn.encode());

    buf.extend(
        ImulInsn::Immediate {
            dst: Reg::Rax,
            src: RegDataRef::Direct(Reg::Rdi),
            mul: RegDataRef::Value32(64),
        }
        .encode(),
    );

    buf.extend(
        ImulInsn::Immediate {
            dst: Reg::Eax,
            src: RegDataRef::Direct(Reg::Edi),
            mul: RegDataRef::Value32(64),
        }
        .encode(),
    );

    buf.extend(
        ImulInsn::Immediate {
            dst: Reg::Eax,
            src: RegDataRef::RegOffset32(Reg::Edi, 18),
            mul: RegDataRef::Value32(64),
        }
        .encode(),
    );

    buf.extend(
        ImulInsn::Registers {
            reg: Reg::Rax,
            mul: RegDataRef::Direct(Reg::Rdi),
        }
        .encode(),
    );

    buf.extend(
        ImulInsn::Registers {
            reg: Reg::Eax,
            mul: RegDataRef::RegOffset32(Reg::Edi, 24),
        }
        .encode(),
    );

    std::fs::write("test.o", buf).unwrap();
}
