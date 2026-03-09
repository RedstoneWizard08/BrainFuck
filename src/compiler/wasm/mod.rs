mod simd;

use crate::{compiler::CompilerOptions, opt::OptAction};
use std::collections::BTreeMap;
use wasm_encoder::{
    BlockType, CodeSection, EntityType, ExportKind, ExportSection, Function, FunctionSection,
    ImportSection, InstructionSink, MemArg, MemorySection, MemoryType, Module, TypeSection,
    ValType,
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
            OptAction::Output => self.print_slot(b),
            OptAction::Input => self.input_slot(b),
            OptAction::Loop(actions) => self.translate_loop(b, actions),
            OptAction::AddValue(v) => self.add_slot(b, *v as i64),
            OptAction::SetValue(v) => self.set_slot(b, *v as i64),
            OptAction::MovePtr(v) => self.move_ptr(b, *v),
            OptAction::SetAndMove(v, o) => self.set_move(b, *v, *o),
            OptAction::AddAndMove(v, o) => self.add_move(b, *v, *o),
            OptAction::SimdAddMove(a, o) => self.unsafe_simd_add_arr_move(b, a, *o),
            OptAction::BulkPrint(n) => self.bulk_print(b, *n),
            OptAction::CopyLoop(v) => self.copy_loop(b, &v),
        }
    }

    fn translate_loop<'i>(&mut self, b: &mut InstructionSink<'i>, actions: &Vec<OptAction>) {
        b.block(BlockType::Empty);
        b.loop_(BlockType::Empty);

        self.ptr(b)
            .i32_load8_u(MemArg {
                align: 0,
                memory_index: 0,
                offset: 0,
            })
            .i32_eqz()
            .br_if(1);

        for insn in actions {
            self.translate(b, insn);
        }

        b.br(0).end().end();
    }

    fn print_slot<'i>(&self, b: &mut InstructionSink<'i>) {
        self.ptr(b)
            .i32_load8_u(MemArg {
                align: 0,
                memory_index: 0,
                offset: 0,
            })
            .call(self.putchar);
    }

    fn bulk_print<'i>(&self, b: &mut InstructionSink<'i>, n: i64) {
        // WASM doesn't support reusing arguments, to my knowledge
        for _ in 0..n {
            self.ptr(b)
                .i32_load8_u(MemArg {
                    align: 0,
                    memory_index: 0,
                    offset: 0,
                })
                .call(self.putchar);
        }
    }

    fn input_slot<'i>(&self, b: &mut InstructionSink<'i>) {
        self.ptr(b).call(self.getchar).i32_store8(MemArg {
            align: 0,
            memory_index: 0,
            offset: 0,
        });
    }

    fn add_slot<'i>(&self, b: &mut InstructionSink<'i>, amount: i64) {
        self.ptr(b);

        self.ptr(b)
            .i32_load8_u(MemArg {
                align: 0,
                memory_index: 0,
                offset: 0,
            })
            .i32_const(amount as i32)
            .i32_add()
            .i32_store8(MemArg {
                align: 0,
                memory_index: 0,
                offset: 0,
            });
    }

    fn set_slot<'i>(&self, b: &mut InstructionSink<'i>, value: i64) {
        self.ptr(b).i32_const(value as i32).i32_store8(MemArg {
            align: 0,
            memory_index: 0,
            offset: 0,
        });
    }

    fn set_move<'i>(&self, b: &mut InstructionSink<'i>, value: i64, offset: i64) {
        self.ptr(b).i32_const(value as i32).i32_store8(MemArg {
            align: 0,
            memory_index: 0,
            offset: 0,
        });

        self.ptr_offset(b, offset).local_set(self.tape_ptr);
    }

    fn add_move<'i>(&self, b: &mut InstructionSink<'i>, amount: i64, offset: i64) {
        self.ptr(b);

        self.ptr(b)
            .i32_load8_u(MemArg {
                align: 0,
                memory_index: 0,
                offset: 0,
            })
            .i32_const(amount as i32)
            .i32_add()
            .i32_store8(MemArg {
                align: 0,
                memory_index: 0,
                offset: 0,
            });

        self.ptr_offset(b, offset).local_set(self.tape_ptr);
    }

    fn move_ptr<'i>(&self, b: &mut InstructionSink<'i>, amount: i64) {
        self.ptr_offset(b, amount).local_set(self.tape_ptr);
    }

    fn copy_loop<'i>(&self, b: &mut InstructionSink<'i>, values: &BTreeMap<i64, i64>) {
        for (offset, mul) in values {
            self.ptr_offset(b, *offset)
                .i32_const(PAGE_SIZE - 1)
                .i32_and();

            self.ptr_offset(b, *offset)
                .i32_const(PAGE_SIZE - 1)
                .i32_and()
                .i32_load8_u(MemArg {
                    offset: 0,
                    align: 0,
                    memory_index: 0,
                });

            self.ptr(b).i32_load8_u(MemArg {
                offset: 0,
                align: 0,
                memory_index: 0,
            });

            b.i32_const(*mul as i32);
            b.i32_mul();
            b.i32_add();

            b.i32_store8(MemArg {
                align: 0,
                memory_index: 0,
                offset: 0,
            });
        }

        self.ptr(b).i32_const(0).i32_store8(MemArg {
            align: 0,
            memory_index: 0,
            offset: 0,
        });
    }
}
