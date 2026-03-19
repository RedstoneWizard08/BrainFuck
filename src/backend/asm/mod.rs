mod copy;
mod insn;
mod io;
mod loops;
mod ptr;
mod simd;
mod value;

use crate::{
    backend::{
        CompilerOptions,
        asm::insn::{AsmBuilder, Data, Insn, Reg, TargetArch},
    },
    opt::{OptAction, ValueAction},
};

const TAPE_DATA_NAME: &str = "tape";

#[allow(unused)]
pub struct CodeGenerator<'a> {
    insns: Vec<Insn>,
    opts: &'a CompilerOptions,
    ptr: Reg,
    ptr_32: Reg,
    block: usize,
}

impl<'a> CodeGenerator<'a> {
    pub fn run(opts: &'a CompilerOptions, actions: &Vec<OptAction>) -> String {
        let mut me = Self {
            opts,
            insns: Vec::new(),
            ptr: Reg::Rbx,
            ptr_32: Reg::Esi,
            block: 0,
        };

        let arch = TargetArch::X86_64; // TODO: Options

        me.compile(actions);

        let mut s = me
            .insns
            .into_iter()
            .map(|it| it.stringify(arch))
            .collect::<Vec<_>>()
            .join("\n");

        s.insert_str(0, ".intel_syntax noprefix\n");

        // GNU assembler likes a new line at the end and shows a warning otherwise XD
        s.push('\n');

        s
    }

    fn compile(&mut self, actions: &Vec<OptAction>) {
        self.sect("bss");
        self.resb(TAPE_DATA_NAME, self.opts.tape_size as i64);
        self.sect("text");
        self.global("_start");
        self.label("_start");
        self.lea(self.ptr, Data::Label(TAPE_DATA_NAME));

        // overflow protection - move the cursor to the center to prevent
        // any potential overflow problems
        self.add(self.ptr, self.opts.tape_size as i64 / 2);

        for insn in actions {
            self.translate(insn);
        }

        self.mov(Reg::Rax, 60);
        self.mov(Reg::Rdi, 0);
        self.syscall();
    }

    fn translate(&mut self, insn: &OptAction) {
        match insn {
            OptAction::Noop => (),

            OptAction::Value(it) => match it {
                ValueAction::Output => self.print_slot(),
                ValueAction::Input => self.input_slot(),
                ValueAction::AddValue(v) => self.add_slot(*v),
                ValueAction::SetValue(v) => self.set_slot(*v),
                ValueAction::BulkPrint(n) => self.bulk_print(*n),
            },

            OptAction::OffsetValue(it, offset) => match it {
                ValueAction::Output => self.print_slot_offset(*offset),
                ValueAction::Input => self.input_slot_offset(*offset),
                ValueAction::AddValue(v) => self.add_slot_offset(*v, *offset),
                ValueAction::SetValue(v) => self.set_slot_offset(*v, *offset),
                ValueAction::BulkPrint(n) => self.bulk_print_offset(*n, *offset),
            },

            OptAction::Loop(actions) => self.translate_loop(actions),
            OptAction::MovePtr(v) => self.move_ptr(*v),
            OptAction::SetAndMove(v, o) => self.set_move(*v, *o),
            OptAction::AddAndMove(v, o) => self.add_move(*v, *o),
            OptAction::SimdAddMove(a, o) => self.unsafe_simd_add_arr_move(a, *o),
            OptAction::CopyLoop(v) => self.copy_loop(&v),
        }
    }
}

impl<'a> AsmBuilder for CodeGenerator<'a> {
    fn insns(&mut self) -> &mut Vec<Insn> {
        &mut self.insns
    }
}
