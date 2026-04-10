mod copy;
mod io;
mod loops;
mod ptr;
mod value;

use asmbin::{
    buf::InsnBuf,
    builders::{InsnBuilder, InsnRecv},
    data::RegDataRef,
    insn::{Insn, mov::MovInsn},
    reg::Reg,
};
use object::{
    Endianness,
    build::elf::{Builder, SectionData},
    elf,
};

use crate::{
    backend::CompilerOptions,
    opt::action::{OptAction, ValueAction},
};

// Magic constant used for "relocation".
const RELOCATOR: u32 = 0xCAFEBABE;
const TAPE_PTR: Reg = Reg::Rbx;

#[allow(unused)]
pub struct CodeGenerator<'a> {
    opts: &'a CompilerOptions,
    known_nonzero: bool,
    known_zero: bool,
}

impl<'a> CodeGenerator<'a> {
    pub fn run(opts: &'a CompilerOptions, actions: &Vec<OptAction>) -> Vec<u8> {
        let mut me = Self {
            opts,
            known_nonzero: false,
            known_zero: false,
        };

        let mut obj = Builder::new(Endianness::Little, true);
        let bss_size = me.opts.tape_size as u64;

        let mut buf = InsnBuf::new();
        let mut start = InsnBuf::new();
        let prog = me.compile(actions);

        start.mov_to_reg(RELOCATOR, TAPE_PTR);
        start.add(TAPE_PTR, me.opts.tape_size as u32 / 2);

        let total_len = start.calculate_length() + prog.calculate_length();

        let header_size =
            obj.file_header_size() as u64 + 2 * obj.class().program_header_size() as u64;

        let bss_offset = 0;
        let text_offset = header_size;

        let base_addr = 0x400000_u64;
        let text_addr = base_addr + text_offset;
        let bss_addr = text_addr + total_len;

        start[0] = Insn::Mov(MovInsn::DataToReg(
            RegDataRef::Value32(bss_addr as u32),
            TAPE_PTR.into(),
        ));

        buf.extend(start);
        buf.extend(prog);

        obj.header.e_type = elf::ET_EXEC;
        obj.header.e_phoff = obj.class().file_header_size() as u64;
        obj.header.e_machine = elf::EM_X86_64;
        obj.header.e_entry = text_addr;

        let text = obj.sections.add();

        text.name = b".text"[..].into();
        text.sh_type = elf::SHT_PROGBITS;
        text.sh_flags = (elf::SHF_ALLOC | elf::SHF_EXECINSTR) as u64;
        text.sh_addralign = 1;
        text.sh_offset = text_offset;
        text.sh_size = total_len;
        text.sh_addr = text_addr;
        text.data = SectionData::Data(buf.encode().into());

        let text_id = text.id();
        let bss = obj.sections.add();

        bss.name = b".bss"[..].into();
        bss.sh_type = elf::SHT_NOBITS;
        bss.sh_flags = (elf::SHF_ALLOC | elf::SHF_WRITE) as u64;
        bss.sh_addralign = 1;
        bss.sh_offset = bss_offset;
        bss.sh_size = bss_size;
        bss.sh_addr = bss_addr;
        bss.data = SectionData::UninitializedData(bss_size);

        let bss_id = bss.id();
        let shstrtab = obj.sections.add();

        shstrtab.name = b".shstrtab"[..].into();
        shstrtab.sh_type = elf::SHT_STRTAB;
        shstrtab.data = SectionData::SectionString;
        shstrtab.sh_addralign = 1;

        obj.set_section_sizes();

        let text_seg = obj.segments.add();

        text_seg.p_type = elf::PT_LOAD;
        text_seg.p_flags = elf::PF_R | elf::PF_W | elf::PF_X;
        text_seg.p_vaddr = text_addr;
        text_seg.p_paddr = text_addr;
        text_seg.p_offset = text_offset;
        text_seg.p_align = 1;
        text_seg.p_filesz = total_len;
        text_seg.p_memsz = total_len + bss_size;

        text_seg.sections.push(bss_id);
        text_seg.sections.push(text_id);

        let mut data = Vec::new();

        obj.write(&mut data).unwrap();

        data
    }

    fn compile(&mut self, actions: &Vec<OptAction>) -> InsnBuf {
        let mut buf = InsnBuf::new();

        for insn in actions {
            self.translate(&mut buf, insn);
        }

        buf.mov_to_reg(60_u32, Reg::Rax);
        buf.mov_to_reg(0_u32, Reg::Rdi);
        buf.syscall();

        buf
    }

    fn translate(&mut self, buf: &mut InsnBuf, insn: &OptAction) {
        match insn {
            OptAction::Noop => (),

            OptAction::Value(it) => match it {
                ValueAction::Output => self.print_slot(buf),
                ValueAction::Input => self.input_slot(buf),
                ValueAction::AddValue(v) => self.add_slot(buf, *v),
                ValueAction::BulkPrint(n) => self.bulk_print(buf, *n),

                ValueAction::SetValue(v) => {
                    if *v == 0 {
                        self.known_zero = true;
                        self.known_nonzero = false;
                    } else {
                        self.known_zero = false;
                        self.known_nonzero = true;
                    }

                    self.set_slot(buf, *v)
                }
            },

            OptAction::OffsetValue(it, offset) => match it {
                ValueAction::Output => self.print_slot_offset(buf, *offset),
                ValueAction::Input => self.input_slot_offset(buf, *offset),
                ValueAction::AddValue(v) => self.add_slot_offset(buf, *v, *offset),
                ValueAction::SetValue(v) => self.set_slot_offset(buf, *v, *offset),
                ValueAction::BulkPrint(n) => self.bulk_print_offset(buf, *n, *offset),
            },

            OptAction::Loop(actions) => self.translate_loop(buf, actions),
            OptAction::MovePtr(v) => self.move_ptr(buf, *v),
            OptAction::SetAndMove(v, o) => self.set_move(buf, *v, *o),
            OptAction::AddAndMove(v, o) => self.add_move(buf, *v, *o),
            OptAction::CopyLoop(v) => self.copy_loop(buf, &v),
            OptAction::Scan(s) => self.scan(buf, *s),
        };

        match insn {
            OptAction::Value(ValueAction::SetValue(_)) => {}

            _ => {
                self.known_zero = false;
                self.known_nonzero = false;
            }
        }
    }
}
