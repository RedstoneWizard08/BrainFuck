mod copy;
mod io;
mod loops;
mod ptr;
mod simd;
mod value;

use crate::{
    backend::CompilerOptions,
    opt::{OptAction, ValueAction},
};
use wasm_encoder::{
    CodeSection, EntityType, ExportKind, ExportSection, Function, FunctionSection, ImportSection,
    InstructionSink, MemorySection, MemoryType, Module, TypeSection, ValType,
};

const PAGE_SIZE: i32 = 65536;

#[allow(unused)]
pub struct CodeGenerator<'a> {
    module: Module,
    opts: &'a CompilerOptions,

    // brainfuck stuff
    tape_ptr: u32,

    // functions
    putchar: u32,
    getchar: u32,
}

impl<'a> CodeGenerator<'a> {
    pub fn run(opts: &'a CompilerOptions, actions: &Vec<OptAction>) -> Vec<u8> {
        let mut module = Module::new();

        let mut types = TypeSection::new();
        let mut imports = ImportSection::new();
        let mut funcs = FunctionSection::new();
        let mut exports = ExportSection::new();
        let mut mem = MemorySection::new();

        types.ty().function([ValType::I32], []);
        imports.import("bf", "putchar", EntityType::Function(0));

        types.ty().function([], [ValType::I32]);
        imports.import("bf", "getchar", EntityType::Function(1));

        types.ty().function([], []);
        funcs.function(2);
        exports.export("_start", ExportKind::Func, 2);

        mem.memory(MemoryType {
            minimum: 1,
            maximum: Some(1),
            memory64: false,
            page_size_log2: None,
            shared: false,
        });

        module.section(&types);
        module.section(&imports);
        module.section(&funcs);
        module.section(&mem);
        module.section(&exports);

        let mut me = Self {
            module,
            tape_ptr: 0,
            putchar: 0,
            getchar: 1,
            opts,
        };

        me.compile(actions);

        let code = me.module.finish();

        wasmparser::validate(&code).unwrap();

        code
    }

    fn compile(&mut self, actions: &Vec<OptAction>) {
        let mut code = CodeSection::new();
        let mut func = Function::new_with_locals_types([ValType::I32]);
        let mut b = func.instructions();

        b.i32_const(0)
            .i32_const(0)
            .i32_const(PAGE_SIZE)
            .memory_fill(0);

        b.i32_const(0).local_set(self.tape_ptr);

        for insn in actions {
            self.translate(&mut b, insn);
        }

        b.end();
        code.function(&func);
        self.module.section(&code);
    }

    fn ptr<'i, 's>(&self, b: &'s mut InstructionSink<'i>) -> &'s mut InstructionSink<'i> {
        b.local_get(self.tape_ptr)
    }

    fn ptr_offset<'i, 's>(
        &self,
        b: &'s mut InstructionSink<'i>,
        offset: i64,
    ) -> &'s mut InstructionSink<'i> {
        b.local_get(self.tape_ptr);

        if offset > 0 {
            b.i32_const(offset as i32).i32_add();
        } else if offset < 0 {
            b.i32_const(-offset as i32).i32_sub();
        }

        b
    }

    fn translate<'i>(&mut self, b: &mut InstructionSink<'i>, insn: &OptAction) {
        match insn {
            OptAction::Noop => (),

            OptAction::Value(it) => match it {
                ValueAction::Output => self.print_slot(b),
                ValueAction::Input => self.input_slot(b),
                ValueAction::AddValue(v) => self.add_slot(b, *v as i64),
                ValueAction::SetValue(v) => self.set_slot(b, *v as i64),
                ValueAction::BulkPrint(n) => self.bulk_print(b, *n),
            },

            OptAction::OffsetValue(it, offset) => match it {
                ValueAction::Output => self.print_slot_offset(b, *offset),
                ValueAction::Input => self.input_slot_offset(b, *offset),
                ValueAction::AddValue(v) => self.add_slot_offset(b, *v as i64, *offset),
                ValueAction::SetValue(v) => self.set_slot_offset(b, *v as i64, *offset),
                ValueAction::BulkPrint(n) => self.bulk_print_offset(b, *n, *offset),
            },

            OptAction::Loop(actions) => self.translate_loop(b, actions),
            OptAction::MovePtr(v) => self.move_ptr(b, *v),
            OptAction::SetAndMove(v, o) => self.set_move(b, *v, *o),
            OptAction::AddAndMove(v, o) => self.add_move(b, *v, *o),
            OptAction::SimdAddMove(a, o) => self.unsafe_simd_add_arr_move(b, a, *o),
            OptAction::CopyLoop(v) => self.copy_loop(b, &v),
        }
    }
}
