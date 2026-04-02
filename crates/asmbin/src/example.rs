use crate::{
    data::RegDataRef,
    insn::{InsnEncode, mov::MovInsn, syscall::SyscallInsn},
    reg::Reg,
};
use anyhow::Result;
use object::{
    Architecture, BinaryFormat, Endianness, RelocationEncoding, RelocationFlags, RelocationKind,
    SymbolFlags, SymbolKind, SymbolScope,
    build::elf::{Builder, SectionData},
    elf::{
        EM_X86_64, ET_EXEC, PF_R, PF_X, PT_LOAD, SHF_ALLOC, SHF_EXECINSTR, SHF_WRITE, SHT_PROGBITS,
        SHT_STRTAB, SHT_SYMTAB, STB_GLOBAL, STT_NOTYPE,
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

    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(1), Reg::Rax).encode());
    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(1), Reg::Rdi).encode());

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
    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(0xABCDEF), Reg::Rsi).encode());
    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(len as u32), Reg::Rdx).encode());
    buf.extend(SyscallInsn.encode());

    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(60), Reg::Rax).encode());
    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(0), Reg::Rdi).encode());
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

    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(1), Reg::Rax).encode());
    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(1), Reg::Rdi).encode());
    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(data_addr as u32), Reg::Rsi).encode());
    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(len as u32), Reg::Rdx).encode());
    buf.extend(SyscallInsn.encode());

    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(60), Reg::Rax).encode());
    buf.extend(MovInsn::DataToReg(RegDataRef::Value32(0), Reg::Rdi).encode());
    buf.extend(SyscallInsn.encode());

    obj.header.e_type = ET_EXEC;
    obj.header.e_phoff = 0x40;
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

    let symtab = obj.sections.add();

    symtab.name = b".symtab"[..].into();
    symtab.sh_type = SHT_SYMTAB;
    symtab.data = SectionData::Symbol;
    symtab.sh_addralign = 8;

    let strtab = obj.sections.add();

    strtab.name = b".strtab"[..].into();
    strtab.sh_type = SHT_STRTAB;
    strtab.data = SectionData::String;
    strtab.sh_addralign = 1;

    let shstrtab = obj.sections.add();

    shstrtab.name = b".shstrtab"[..].into();
    shstrtab.sh_type = SHT_STRTAB;
    shstrtab.data = SectionData::SectionString;
    shstrtab.sh_addralign = 1;

    let sym = obj.symbols.add();

    sym.name = b"_start"[..].into();
    sym.set_st_info(STB_GLOBAL, STT_NOTYPE);
    sym.section = Some(text_id);
    sym.st_value = text_addr;

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
