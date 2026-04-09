use crate::{
    buf::InsnBuf,
    builders::InsnRecv,
    data::RegDataRef,
    insn::{
        Insn, InsnEncode,
        cmp::CmpInsn,
        jmp::{JmpCond, JmpInsn},
        mov::MovInsn,
        syscall::SyscallInsn,
    },
    reg::Reg,
};
use anyhow::Result;
use object::{
    Architecture, BinaryFormat, Endianness, RelocationEncoding, RelocationFlags, RelocationKind,
    SymbolFlags, SymbolKind, SymbolScope,
    build::elf::{Builder, SectionData},
    elf::{
        EM_X86_64, ET_EXEC, PF_R, PF_W, PF_X, PT_LOAD, SHF_ALLOC, SHF_EXECINSTR, SHF_WRITE,
        SHT_NOBITS, SHT_PROGBITS, SHT_STRTAB,
    },
    write::{Object, Relocation, StandardSection, Symbol, SymbolSection},
};
use std::fs;

pub fn hello_world_with_reloc() -> Result<()> {
    let mut obj = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    let data = obj.add_subsection(StandardSection::Data, &[]);
    let text = obj.add_subsection(StandardSection::Text, &[]);

    let msg = "Hello, world!\n";
    let len = msg.len() as u64;

    let offset = obj.append_section_data(data, msg.as_bytes(), len.next_power_of_two() as u64);

    let msg_id = obj.add_symbol(Symbol {
        name: "msg".as_bytes().to_vec(),
        value: offset,
        size: len,
        kind: SymbolKind::Data,
        scope: SymbolScope::Compilation,
        weak: false,
        section: SymbolSection::Section(data),
        flags: SymbolFlags::None,
    });

    let mut buf = Vec::new();

    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(1), Reg::Rax.into()).encode());
    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(1), Reg::Rdi.into()).encode());

    obj.add_relocation(
        text,
        Relocation {
            offset: buf.len() as u64 + 3,
            addend: 0,
            symbol: msg_id,
            flags: RelocationFlags::Generic {
                kind: RelocationKind::Absolute,
                encoding: RelocationEncoding::Generic,
                size: 32,
            },
        },
    )?;

    // 0xABCDEF gets replaced by the relocation
    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(0xABCDEF), Reg::Rsi.into()).encode());
    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(len as u32), Reg::Rdx.into()).encode());
    buf.extend(SyscallInsn.encode());

    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(60), Reg::Rax.into()).encode());
    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(0), Reg::Rdi.into()).encode());
    buf.extend(SyscallInsn.encode());

    obj.section_mut(text)
        .set_data(&buf, buf.len().next_power_of_two() as u64);

    obj.add_symbol(Symbol {
        name: "_start".as_bytes().to_vec(),
        value: 0,
        size: buf.len() as u64,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(text),
        flags: SymbolFlags::None,
    });

    let data = obj.write()?;

    fs::write("test.o", data)?;

    Ok(())
}

pub fn hello_world_no_reloc() -> Result<()> {
    let mut obj = Builder::new(Endianness::Little, true);

    let msg = "Hello, world!\n";
    let len = msg.len() as u64;

    let header_size = obj.file_header_size() as u64 + 2 * obj.class().program_header_size() as u64;
    let data_offset = header_size;
    let text_offset = data_offset + msg.len() as u64;

    let base_addr = 0x400000_u64;
    let data_addr = base_addr + data_offset;
    let text_addr = data_addr + msg.len() as u64;

    let mut buf = Vec::new();

    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(1), Reg::Rax.into()).encode());
    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(1), Reg::Rdi.into()).encode());
    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(data_addr as u32), Reg::Rsi.into()).encode());
    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(len as u32), Reg::Rdx.into()).encode());
    buf.extend(SyscallInsn.encode());

    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(60), Reg::Rax.into()).encode());
    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(0), Reg::Rdi.into()).encode());
    buf.extend(SyscallInsn.encode());

    obj.header.e_type = ET_EXEC;
    obj.header.e_phoff = obj.class().file_header_size() as u64;
    obj.header.e_machine = EM_X86_64;
    obj.header.e_entry = text_addr;

    let text = obj.sections.add();

    text.name = b".text"[..].into();
    text.sh_type = SHT_PROGBITS;
    text.sh_flags = (SHF_ALLOC | SHF_EXECINSTR) as u64;
    text.sh_addralign = 1;
    text.sh_offset = text_offset;
    text.sh_addr = text_addr;
    text.data = SectionData::Data(buf.clone().into());

    let text_id = text.id();

    let data = obj.sections.add();

    data.name = b".data"[..].into();
    data.sh_type = SHT_PROGBITS;
    data.sh_flags = (SHF_ALLOC | SHF_WRITE) as u64;
    data.sh_addralign = 1;
    data.sh_offset = data_offset;
    data.sh_addr = data_addr;
    data.data = SectionData::Data(msg.as_bytes().into());

    let data_id = data.id();

    let shstrtab = obj.sections.add();

    shstrtab.name = b".shstrtab"[..].into();
    shstrtab.sh_type = SHT_STRTAB;
    shstrtab.data = SectionData::SectionString;
    shstrtab.sh_addralign = 1;

    obj.set_section_sizes();

    let data_seg = obj.segments.add();

    data_seg.p_type = PT_LOAD;
    data_seg.p_flags = PF_R | PF_X;
    data_seg.p_vaddr = data_addr;
    data_seg.p_paddr = data_addr;
    data_seg.p_offset = data_offset;
    data_seg.p_align = 8;

    data_seg.append_section(obj.sections.get_mut(data_id));
    data_seg.append_section(obj.sections.get_mut(text_id));

    let mut data = Vec::new();

    obj.write(&mut data)?;
    fs::write("test.o", data)?;

    Ok(())
}

pub fn echo_no_reloc() -> Result<()> {
    let mut obj = Builder::new(Endianness::Little, true);

    let bss_size = 1024_u64;

    let header_size = obj.file_header_size() as u64 + 2 * obj.class().program_header_size() as u64;
    let bss_offset = header_size;
    let text_offset = (header_size + bss_size).next_multiple_of(0x1000);

    let base_addr = 0x400000_u64;
    let bss_addr = base_addr;
    let text_addr = (bss_addr + bss_size).next_multiple_of(0x1000);

    let mut buf = Vec::new();
    let mut read = Vec::new();
    let mut cmp = Vec::new();
    let mut write = Vec::new();

    read.extend(MovInsn::DataToReg(RegDataRef::Value32(0), Reg::Rax.into()).encode());
    read.extend(MovInsn::DataToReg(RegDataRef::Value32(0), Reg::Rdi.into()).encode());
    read.extend(MovInsn::DataToReg(RegDataRef::Value32(bss_addr as u32), Reg::Rsi.into()).encode());
    read.extend(MovInsn::DataToReg(RegDataRef::Value32(bss_size as u32), Reg::Rdx.into()).encode());
    read.extend(SyscallInsn.encode());

    write.extend(MovInsn::DataToReg(RegDataRef::Direct(Reg::Rax), Reg::Rdi.into()).encode());
    write.extend(MovInsn::DataToReg(RegDataRef::Value32(1), Reg::Rax.into()).encode());
    write
        .extend(MovInsn::DataToReg(RegDataRef::Value32(bss_addr as u32), Reg::Rsi.into()).encode());
    write.extend(MovInsn::DataToReg(RegDataRef::Direct(Reg::Rdi), Reg::Rdx.into()).encode());
    write.extend(MovInsn::DataToReg(RegDataRef::Value32(1), Reg::Rdi.into()).encode());
    write.extend(SyscallInsn.encode());

    cmp.extend(CmpInsn(Reg::Eax.into(), RegDataRef::Value32(0)).encode());
    cmp.extend(JmpInsn::Cond32(JmpCond::LessEqual, write.len() as i32).encode());

    buf.extend(read);
    buf.extend(cmp);
    buf.extend(write);

    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(60), Reg::Rax.into()).encode());
    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(0), Reg::Rdi.into()).encode());
    buf.extend(SyscallInsn.encode());

    obj.header.e_type = ET_EXEC;
    obj.header.e_phoff = obj.class().file_header_size() as u64;
    obj.header.e_machine = EM_X86_64;
    obj.header.e_entry = text_addr;

    let text = obj.sections.add();

    text.name = b".text"[..].into();
    text.sh_type = SHT_PROGBITS;
    text.sh_flags = (SHF_ALLOC | SHF_EXECINSTR) as u64;
    text.sh_addralign = 1;
    text.sh_offset = text_offset;
    text.sh_size = buf.len() as u64;
    text.sh_addr = text_addr;
    text.data = SectionData::Data(buf.clone().into());

    let text_id = text.id();
    let bss = obj.sections.add();

    bss.name = b".bss"[..].into();
    bss.sh_type = SHT_NOBITS;
    bss.sh_flags = (SHF_ALLOC | SHF_WRITE) as u64;
    bss.sh_addralign = 1;
    bss.sh_offset = bss_offset;
    bss.sh_size = bss_size;
    bss.sh_addr = bss_addr;
    bss.data = SectionData::UninitializedData(bss_size);

    let bss_id = bss.id();
    let shstrtab = obj.sections.add();

    shstrtab.name = b".shstrtab"[..].into();
    shstrtab.sh_type = SHT_STRTAB;
    shstrtab.data = SectionData::SectionString;
    shstrtab.sh_addralign = 1;

    obj.set_section_sizes();

    let bss_seg = obj.segments.add();

    bss_seg.p_type = PT_LOAD;
    bss_seg.p_flags = PF_R | PF_W;
    bss_seg.p_vaddr = bss_addr;
    bss_seg.p_paddr = bss_addr;
    bss_seg.p_offset = bss_offset;
    bss_seg.p_align = 16;
    bss_seg.p_filesz = 0;
    bss_seg.p_memsz = bss_size;

    bss_seg.sections.push(bss_id);

    let text_seg = obj.segments.add();

    text_seg.p_type = PT_LOAD;
    text_seg.p_flags = PF_R | PF_X;
    text_seg.p_vaddr = text_addr;
    text_seg.p_paddr = text_addr;
    text_seg.p_offset = text_offset;
    text_seg.p_align = 0x1000;

    text_seg.append_section(obj.sections.get_mut(text_id));

    let mut data = Vec::new();

    obj.write(&mut data)?;
    fs::write("test.o", data)?;

    Ok(())
}

pub fn echo_no_reloc_compact() -> Result<()> {
    let mut obj = Builder::new(Endianness::Little, true);

    let bss_size = 1024_u64;

    let mut buf = InsnBuf::new();
    let mut read = InsnBuf::new();
    let mut cmp = InsnBuf::new();
    let mut write = InsnBuf::new();
    let mut end = InsnBuf::new();

    let bss_addr = 0;
    let bss_len = bss_size as u32;

    read.push(MovInsn::DataToReg(RegDataRef::Value32(0), Reg::Rax.into()));
    read.push(MovInsn::DataToReg(RegDataRef::Value32(0), Reg::Rdi.into()));
    read.push(MovInsn::DataToReg(
        RegDataRef::Value32(bss_addr),
        Reg::Rsi.into(),
    ));
    read.push(MovInsn::DataToReg(
        RegDataRef::Value32(bss_len),
        Reg::Rdx.into(),
    ));
    read.push(SyscallInsn);

    write.push(MovInsn::DataToReg(
        RegDataRef::Direct(Reg::Rax),
        Reg::Rdi.into(),
    ));
    write.push(MovInsn::DataToReg(RegDataRef::Value32(1), Reg::Rax.into()));
    write.push(MovInsn::DataToReg(
        RegDataRef::Value32(bss_addr),
        Reg::Rsi.into(),
    ));
    write.push(MovInsn::DataToReg(
        RegDataRef::Direct(Reg::Rdi),
        Reg::Rdx.into(),
    ));
    write.push(MovInsn::DataToReg(RegDataRef::Value32(1), Reg::Rdi.into()));
    write.push(SyscallInsn);

    let write_len = write.calculate_length();

    cmp.push(CmpInsn(Reg::Eax.into(), RegDataRef::Value32(0)));
    cmp.push(JmpInsn::Cond32(JmpCond::LessEqual, write_len as i32));

    end.push(MovInsn::DataToReg(RegDataRef::Value32(60), Reg::Rax.into()));
    end.push(MovInsn::DataToReg(RegDataRef::Value32(0), Reg::Rdi.into()));
    end.push(SyscallInsn);

    let total_len = read.calculate_length()
        + cmp.calculate_length()
        + write.calculate_length()
        + end.calculate_length();

    let header_size = obj.file_header_size() as u64 + 2 * obj.class().program_header_size() as u64;
    let bss_offset = 0;
    let text_offset = header_size;

    let base_addr = 0x400000_u64;
    let text_addr = base_addr + text_offset;
    let bss_addr = text_addr + total_len;

    read[2] = Insn::Mov(MovInsn::DataToReg(
        RegDataRef::Value32(bss_addr as u32),
        Reg::Rsi.into(),
    ));

    write[2] = Insn::Mov(MovInsn::DataToReg(
        RegDataRef::Value32(bss_addr as u32),
        Reg::Rsi.into(),
    ));

    buf.extend(read);
    buf.extend(cmp);
    buf.extend(write);
    buf.extend(end);

    obj.header.e_type = ET_EXEC;
    obj.header.e_phoff = obj.class().file_header_size() as u64;
    obj.header.e_machine = EM_X86_64;
    obj.header.e_entry = text_addr;

    let text = obj.sections.add();

    text.name = b".text"[..].into();
    text.sh_type = SHT_PROGBITS;
    text.sh_flags = (SHF_ALLOC | SHF_EXECINSTR) as u64;
    text.sh_addralign = 1;
    text.sh_offset = text_offset;
    text.sh_size = total_len;
    text.sh_addr = text_addr;
    text.data = SectionData::Data(buf.encode().into());

    let text_id = text.id();
    let bss = obj.sections.add();

    bss.name = b".bss"[..].into();
    bss.sh_type = SHT_NOBITS;
    bss.sh_flags = (SHF_ALLOC | SHF_WRITE) as u64;
    bss.sh_addralign = 1;
    bss.sh_offset = bss_offset;
    bss.sh_size = bss_size;
    bss.sh_addr = bss_addr;
    bss.data = SectionData::UninitializedData(bss_size);

    let bss_id = bss.id();
    let shstrtab = obj.sections.add();

    shstrtab.name = b".shstrtab"[..].into();
    shstrtab.sh_type = SHT_STRTAB;
    shstrtab.data = SectionData::SectionString;
    shstrtab.sh_addralign = 1;

    obj.set_section_sizes();

    let text_seg = obj.segments.add();

    text_seg.p_type = PT_LOAD;
    text_seg.p_flags = PF_R | PF_W | PF_X;
    text_seg.p_vaddr = text_addr;
    text_seg.p_paddr = text_addr;
    text_seg.p_offset = text_offset;
    text_seg.p_align = 1;
    text_seg.p_filesz = total_len;
    text_seg.p_memsz = total_len + bss_size;

    text_seg.sections.push(bss_id);
    text_seg.sections.push(text_id);

    let mut data = Vec::new();

    obj.write(&mut data)?;
    fs::write("test.o", data)?;

    Ok(())
}
