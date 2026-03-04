use crate::{TAPE_SIZE, compiler::CompilerOptions, optimizer::OptAction};
use anyhow::Result;
use inkwell::{
    AddressSpace, IntPredicate, OptimizationLevel,
    builder::{Builder, BuilderError},
    context::Context,
    module::Module,
    passes::PassBuilderOptions,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    types::{BasicMetadataTypeEnum, IntType, PointerType},
    values::{BasicValueEnum, FunctionValue, IntValue, PointerValue},
};
use std::{collections::BTreeMap, fs};
use target_lexicon::Triple;

pub fn llvm_opt_level(level: u8) -> OptimizationLevel {
    match level {
        0 => OptimizationLevel::None,
        1 => OptimizationLevel::Less,
        2 => OptimizationLevel::Default,
        _ => OptimizationLevel::Aggressive,
    }
}

pub fn jit_compile(actions: &Vec<OptAction>, opts: CompilerOptions) -> Result<()> {
    let cx = Context::create();
    let module = cx.create_module("_entry");

    let exec = module
        .create_jit_execution_engine(llvm_opt_level(opts.opt_level))
        .unwrap();

    let target = exec.get_target_data();
    let ptr = cx.ptr_sized_int_type(target, None);

    let compiler = Compiler::new(&cx, module, ptr, false, opts);

    compiler.compile(actions)?;

    let func = unsafe { exec.get_function::<unsafe extern "C" fn() -> ()>("_entry")? };

    unsafe {
        func.call();
    };

    Ok(())
}

// TODO: Cross-compilation
pub fn aot_compile(
    actions: &Vec<OptAction>,
    _target: &Triple,
    opts: CompilerOptions,
) -> Result<Vec<u8>> {
    let output_asm = opts.output_asm.clone();
    let cx = Context::create();
    let module = cx.create_module("_start");

    Target::initialize_all(&InitializationConfig::default());

    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).unwrap();

    let machine = target
        .create_target_machine(
            &triple,
            &TargetMachine::get_host_cpu_name().to_string(),
            &TargetMachine::get_host_cpu_features().to_string(),
            llvm_opt_level(opts.opt_level),
            RelocMode::PIC,
            CodeModel::default(),
        )
        .unwrap();

    let data = machine.get_target_data();
    let ptr = cx.ptr_sized_int_type(&data, None);

    module.set_triple(&triple);
    module.set_data_layout(&data.get_data_layout());

    let compiler = Compiler::new(&cx, module, ptr, true, opts);
    let (module, _func) = compiler.compile(actions)?;

    if let Some(path) = output_asm {
        let asm = machine
            .write_to_memory_buffer(&module, FileType::Assembly)
            .unwrap()
            .as_slice()
            .to_vec();

        fs::write(path, asm)?;
    }

    let obj = machine
        .write_to_memory_buffer(&module, FileType::Object)
        .unwrap();

    let obj = obj.as_slice().to_vec();

    Ok(obj)
}

pub struct Compiler<'c> {
    cx: &'c Context,
    module: Module<'c>,
    b: Builder<'c>,
    ptr: IntType<'c>,
    space: AddressSpace,
    needs_exit: bool,
    opts: CompilerOptions,
}

impl<'c> Compiler<'c> {
    pub fn new(
        cx: &'c Context,
        module: Module<'c>,
        ptr: IntType<'c>,
        needs_exit: bool,
        opts: CompilerOptions,
    ) -> Self {
        Self {
            cx,
            module,
            b: cx.create_builder(),
            needs_exit,
            ptr,
            space: AddressSpace::default(),
            opts,
        }
    }

    pub fn compile(
        self,
        actions: &Vec<OptAction>,
    ) -> Result<(Module<'c>, FunctionValue<'c>), BuilderError> {
        let mut cg = CodeGenerator::new(
            self.cx,
            self.module,
            self.b,
            self.ptr,
            self.space,
            self.needs_exit,
            self.opts,
        )?;

        cg.compile(actions)?;
        cg.write_debug_output();
        cg.verify();
        cg.optimize();
        cg.write_debug_output();

        Ok(cg.finish())
    }
}

#[allow(unused)]
pub struct CodeGenerator<'c> {
    // types
    ptr: IntType<'c>,
    ptr_ty: PointerType<'c>,
    byte: IntType<'c>,
    int: IntType<'c>,
    long: IntType<'c>,

    // cx
    cx: &'c Context,
    module: Module<'c>,
    b: Builder<'c>,
    needs_exit: bool,
    opts: CompilerOptions,

    // state
    func: FunctionValue<'c>, // the function we are building

    // brainfuck stuff
    // tape: StackSlot,
    tape_ptr_ptr: PointerValue<'c>,
    loop_id: usize,

    // functions
    putchar: FunctionValue<'c>,
    getchar: FunctionValue<'c>,
    exit: FunctionValue<'c>,
}

impl<'c> CodeGenerator<'c> {
    pub fn new(
        cx: &'c Context,
        module: Module<'c>,
        builder: Builder<'c>,
        ptr: IntType<'c>,
        space: AddressSpace,
        needs_exit: bool,
        opts: CompilerOptions,
    ) -> Result<Self, BuilderError> {
        let ptr_ty = cx.ptr_type(space);
        let byte = cx.i8_type();
        let int = cx.i32_type();
        let long = cx.i64_type();
        let void = cx.void_type();

        let func = void.fn_type(&[], false);
        let func = module.add_function("_start", func, None);
        let block = cx.append_basic_block(func, "_start");

        builder.position_at_end(block);

        let size = cx.i64_type().const_int(TAPE_SIZE as u64, false);
        let tape_ptr = builder.build_array_alloca(byte, size, "tape")?;
        let tape_ptr_ptr = builder.build_alloca(ptr, "tape_ptr")?;

        builder.build_store(tape_ptr_ptr, tape_ptr)?;

        let putchar = int.fn_type(&[BasicMetadataTypeEnum::IntType(int)], false);
        let putchar = module.add_function("putchar", putchar, None);

        let getchar = int.fn_type(&[], false);
        let getchar = module.add_function("getchar", getchar, None);

        let exit = void.fn_type(&[BasicMetadataTypeEnum::IntType(int)], false);
        let exit = module.add_function("exit", exit, None);

        Ok(Self {
            ptr,
            ptr_ty,
            byte,
            int,
            long,
            module,
            b: builder,
            cx,
            func,
            tape_ptr_ptr,
            putchar,
            getchar,
            exit,
            needs_exit,
            opts,

            loop_id: 0,
        })
    }

    pub fn finish(self) -> (Module<'c>, FunctionValue<'c>) {
        (self.module, self.func)
    }

    pub fn compile(&mut self, actions: &Vec<OptAction>) -> Result<(), BuilderError> {
        for insn in actions {
            self.translate(insn)?;
        }

        if self.needs_exit {
            self.b
                .build_call(self.exit, &[self.int.const_zero().into()], "exit")?;
        }

        self.b.build_return(None)?;

        Ok(())
    }

    pub fn optimize(&mut self) {
        self.module.strip_debug_info();

        if self.opts.opt_level == 0 {
            return;
        }

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).unwrap();

        let machine = target
            .create_target_machine(
                &triple,
                &TargetMachine::get_host_cpu_name().to_string(),
                &TargetMachine::get_host_cpu_features().to_string(),
                llvm_opt_level(self.opts.opt_level),
                RelocMode::PIC,
                CodeModel::default(),
            )
            .unwrap();

        let opts = PassBuilderOptions::create();

        opts.set_call_graph_profile(true);
        opts.set_forget_all_scev_in_loop_unroll(true);
        opts.set_loop_interleaving(true);
        opts.set_loop_slp_vectorization(true);
        opts.set_loop_unrolling(true);
        opts.set_loop_vectorization(true);
        opts.set_merge_functions(true);
        // opts.set_verify_each(true);

        let pass = [
            "always-inline",
            "constmerge",
            "mergefunc",
            "loop-simplify",
            "loop-sink",
            "loop-data-prefetch",
            "unify-loop-exits",
            "no-op-loop",
            "loop-unroll-full",
            "indvars",
            "mem2reg",
            "load-store-vectorizer",
            "interleaved-access",
            "gc-lowering",
            "extra-vector-passes",
            "vector-combine",
            "verify",
        ];

        let pass = pass.join(",");

        self.module.run_passes(&pass, &machine, opts).unwrap();
    }

    pub fn verify(&mut self) {
        if let Err(err) = self.module.verify() {
            panic!(
                "Failed to validate LLVM module! Error info:\n{}",
                err.to_string()
            );
        }
    }

    pub fn write_debug_output(&mut self) {
        if let Some(path) = &self.opts.output_ir {
            // let mut buf = String::new();

            // buf.push_str(&format!("target {}\n\n", self.module.isa().triple()));

            // for flag in self.module.isa().flags().iter() {
            //     buf.push_str(&format!("set {flag}\n"));
            // }

            // buf.push('\n');
            // write_function(&mut buf, &self.cx.func).unwrap();
            // buf.push('\n');

            fs::write(path, self.module.to_string()).unwrap();
        }

        // if let Some(path) = &self.opts.output_asm {
        //     let code = self
        //         .cx
        //         .compile(self.module.isa(), &mut ControlPlane::default())
        //         .unwrap();

        //     let isa = self.module.isa();
        //     let cap = isa.to_capstone().unwrap();
        //     let disas = cap.disasm_all(code.code_buffer(), 0).unwrap().to_string();

        //     fs::write(path, disas).unwrap();
        // }
    }

    fn translate(&mut self, insn: &OptAction) -> Result<(), BuilderError> {
        match insn {
            OptAction::Noop => Ok(()),
            OptAction::Output => self.print_slot(),
            OptAction::Input => self.input_slot(),
            OptAction::Loop(actions) => self.translate_loop(actions),
            OptAction::AddValue(v) => self.add_slot(*v as i64),
            OptAction::SetValue(v) => self.set_slot(*v as i64),
            OptAction::MovePtr(v) => self.move_ptr(*v),
            OptAction::SetAndMove(v, o) => self.set_move(*v, *o),
            OptAction::AddAndMove(v, o) => self.add_move(*v, *o),

            OptAction::CopyLoop(v) => {
                if self.opts.unsafe_mode {
                    self.unsafe_copy_loop(&v)
                } else {
                    panic!("The CopyLoop optimization is only supported in unsafe mode!")
                }
            }
        }
    }

    fn translate_loop(&mut self, actions: &Vec<OptAction>) -> Result<(), BuilderError> {
        let name = format!("_loop_block_{}", self.loop_id);
        let header = self.cx.append_basic_block(self.func, &name);
        let body = self.cx.append_basic_block(self.func, &name);
        let exit = self.cx.append_basic_block(self.func, &name);

        self.loop_id += 1;

        self.b.build_unconditional_branch(header)?;
        self.b.position_at_end(header);

        let value = self.read_from_arr()?;

        let cond = self.b.build_int_compare(
            IntPredicate::NE,
            value.into_int_value(),
            self.byte.const_zero(),
            "check_loop_zero",
        )?;

        self.b.build_conditional_branch(cond, body, exit)?;
        self.b.position_at_end(body);
        // self.fb.seal_block(body);

        for action in actions {
            self.translate(action)?;
        }

        self.b.build_unconditional_branch(header)?;
        self.b.position_at_end(exit);
        // self.fb.seal_block(header);
        // self.fb.seal_block(exit);

        Ok(())
    }

    // "Unsafe" methods use the tape_ptr as a literal pointer instead of an index into the tape array

    fn print_slot(&mut self) -> Result<(), BuilderError> {
        let value = self.read_from_arr()?;

        let value = self
            .b
            .build_int_z_extend(value.into_int_value(), self.int, "u8_to_i32")?;

        self.b
            .build_call(self.putchar, &[value.into()], "putchar")?;

        Ok(())
    }

    fn input_slot(&mut self) -> Result<(), BuilderError> {
        let call = self.b.build_call(self.getchar, &[], "getchar")?;
        let value = call.try_as_basic_value();

        let value = self.b.build_int_truncate(
            value.unwrap_basic().into_int_value(),
            self.byte,
            "i32_to_u8",
        )?;

        self.write_to_arr(value)?;

        Ok(())
    }

    fn write_to_arr(&mut self, value: IntValue<'c>) -> Result<(), BuilderError> {
        if self.opts.unsafe_mode {
            return self.unsafe_write_to_arr(value);
        }

        todo!("Safe output is currently not supported on the LLVM backend!");

        // let offset = self.fb.use_var(self.tape_ptr);
        // let base_addr = self.fb.ins().stack_addr(self.ptr, self.tape, 0);
        // let final_addr = self.fb.ins().iadd(base_addr, offset);

        // self.fb.ins().store(MemFlags::new(), value, final_addr, 0);

        // Ok(())
    }

    fn get_tape_ptr(&mut self, n: &str) -> Result<PointerValue<'c>, BuilderError> {
        Ok(self
            .b
            .build_load(self.ptr_ty, self.tape_ptr_ptr, n)?
            .into_pointer_value())
    }

    fn set_tape_ptr(&mut self, value: IntValue<'c>) -> Result<(), BuilderError> {
        self.b.build_store(self.tape_ptr_ptr, value)?;

        Ok(())
    }

    fn unsafe_write_to_arr(&mut self, value: IntValue<'c>) -> Result<(), BuilderError> {
        let ptr = self.get_tape_ptr("get_ptr__write_to_arr")?;

        self.b.build_store(ptr, value)?;

        Ok(())
    }

    fn add_slot(&mut self, amount: i64) -> Result<(), BuilderError> {
        if self.opts.unsafe_mode {
            return self.unsafe_add_slot(amount);
        }

        todo!("Safe output is currently not supported on the LLVM backend!");

        // let offset = self.fb.use_var(self.tape_ptr);
        // let base_addr = self.fb.ins().stack_addr(self.ptr, self.tape, 0);
        // let final_addr = self.fb.ins().iadd(base_addr, offset);

        // let value = self
        //     .fb
        //     .ins()
        //     .load(self.byte, MemFlags::new(), final_addr, 0);

        // let value = self.fb.ins().iadd_imm(value, amount);

        // self.fb.ins().store(MemFlags::new(), value, final_addr, 0);

        // Ok(())
    }

    fn unsafe_add_slot(&mut self, amount: i64) -> Result<(), BuilderError> {
        let val = self.byte.const_int(amount as u64, true);
        let ptr = self.get_tape_ptr("get_ptr__add_slot")?;
        let cur = self
            .b
            .build_load(self.byte, ptr, "add_slot")?
            .into_int_value();
        let new = self.b.build_int_add(cur, val, "add_slot")?;

        self.b.build_store(ptr, new)?;

        Ok(())
    }

    fn set_slot(&mut self, value: i64) -> Result<(), BuilderError> {
        if self.opts.unsafe_mode {
            return self.unsafe_set_slot(value);
        }

        todo!("Safe output is currently not supported on the LLVM backend!");

        // let offset = self.fb.use_var(self.tape_ptr);
        // let base_addr = self.fb.ins().stack_addr(self.ptr, self.tape, 0);
        // let final_addr = self.fb.ins().iadd(base_addr, offset);
        // let value = self.fb.ins().iconst(types::I64, value);
        // let value = self.fb.ins().ireduce(self.byte, value);

        // self.fb.ins().store(MemFlags::new(), value, final_addr, 0);

        // Ok(())
    }

    fn unsafe_set_slot(&mut self, value: i64) -> Result<(), BuilderError> {
        let ptr = self.get_tape_ptr("get_ptr__set_slot")?;
        let val = self.byte.const_int(value as u64, false);

        self.b.build_store(ptr, val)?;

        Ok(())
    }

    fn set_move(&mut self, value: i64, offset: i64) -> Result<(), BuilderError> {
        if self.opts.unsafe_mode {
            return self.unsafe_set_move(value, offset);
        }

        todo!("Safe output is currently not supported on the LLVM backend!");

        // let offset_v = self.fb.use_var(self.tape_ptr);
        // let post = self.fb.ins().iadd_imm(offset_v, offset);
        // let base_addr = self.fb.ins().stack_addr(self.ptr, self.tape, 0);
        // let final_addr = self.fb.ins().iadd(base_addr, offset_v);
        // let value = self.fb.ins().iconst(types::I64, value);
        // let value = self.fb.ins().ireduce(self.byte, value);

        // self.fb.ins().store(MemFlags::new(), value, final_addr, 0);
        // self.fb.def_var(self.tape_ptr, post);

        // Ok(())
    }

    fn unsafe_set_move(&mut self, value: i64, offset: i64) -> Result<(), BuilderError> {
        let ptr = self.get_tape_ptr("get_ptr__set_move")?;
        let val = self.byte.const_int(value as u64, false);

        self.b.build_store(ptr, val)?;

        let ptr_i = self.b.build_ptr_to_int(ptr, self.ptr, "p2i_set_move")?;
        let add = self.ptr.const_int(offset as u64, true);
        let new = self.b.build_int_add(ptr_i, add, "set_move")?;

        self.set_tape_ptr(new)?;

        Ok(())
    }

    fn add_move(&mut self, amount: i64, offset: i64) -> Result<(), BuilderError> {
        if self.opts.unsafe_mode {
            return self.unsafe_add_move(amount, offset);
        }

        todo!("Safe output is currently not supported on the LLVM backend!");

        // let offset_v = self.fb.use_var(self.tape_ptr);
        // let post = self.fb.ins().iadd_imm(offset_v, offset);
        // let base_addr = self.fb.ins().stack_addr(self.ptr, self.tape, 0);
        // let final_addr = self.fb.ins().iadd(base_addr, offset_v);

        // let value = self
        //     .fb
        //     .ins()
        //     .load(self.byte, MemFlags::new(), final_addr, 0);

        // let value = self.fb.ins().iadd_imm(value, amount);

        // self.fb.ins().store(MemFlags::new(), value, final_addr, 0);
        // self.fb.def_var(self.tape_ptr, post);

        // Ok(())
    }

    fn unsafe_add_move(&mut self, amount: i64, offset: i64) -> Result<(), BuilderError> {
        let ptr = self.get_tape_ptr("get_ptr__add_move")?;

        let cur = self
            .b
            .build_load(self.byte, ptr, "add_move")?
            .into_int_value();

        let val = self.byte.const_int(amount as u64, false);
        let val = self.b.build_int_add(cur, val, "add_move")?;

        self.b.build_store(ptr, val)?;

        let ptr_i = self.b.build_ptr_to_int(ptr, self.ptr, "p2i_add_move")?;
        let add = self.ptr.const_int(offset as u64, true);
        let new = self.b.build_int_add(ptr_i, add, "add_move")?;

        self.set_tape_ptr(new)?;

        Ok(())
    }

    fn read_from_arr(&mut self) -> Result<BasicValueEnum<'c>, BuilderError> {
        if self.opts.unsafe_mode {
            return self.unsafe_read_from_arr();
        }

        todo!("Safe output is currently not supported on the LLVM backend!");

        // let offset = self.fb.use_var(self.tape_ptr);
        // let base_addr = self.fb.ins().stack_addr(self.ptr, self.tape, 0);
        // let final_addr = self.fb.ins().iadd(base_addr, offset);

        // Ok(self.fb
        //     .ins()
        //     .load(self.byte, MemFlags::new(), final_addr, 0))
    }

    fn unsafe_read_from_arr(&mut self) -> Result<BasicValueEnum<'c>, BuilderError> {
        let ptr = self.get_tape_ptr("get_ptr__read_from_arr")?;

        self.b.build_load(self.byte, ptr, "read_from_arr")
    }

    fn move_ptr(&mut self, amount: i64) -> Result<(), BuilderError> {
        if self.opts.unsafe_mode {
            return self.unsafe_move_right(amount);
        }

        todo!("Safe output is currently not supported on the LLVM backend!");

        // let value = self.fb.use_var(self.tape_ptr);
        // let value = self.fb.ins().iadd_imm(value, amount as i64);

        // let did_hit_max = self.fb.ins().icmp_imm(
        //     IntCC::UnsignedGreaterThanOrEqual,
        //     value,
        //     (TAPE_SIZE - 1) as i64,
        // );

        // let did_hit_min = self
        //     .fb
        //     .ins()
        //     .icmp_imm(IntCC::UnsignedLessThanOrEqual, value, 0);

        // let max_wrap = self.fb.ins().iadd_imm(value, -(TAPE_SIZE as i64));
        // let min_wrap = self.fb.ins().iadd_imm(value, TAPE_SIZE as i64);

        // let max_clamp = self.fb.ins().select(did_hit_max, max_wrap, value);
        // let min_clamp = self.fb.ins().select(did_hit_min, min_wrap, max_clamp);

        // self.fb.def_var(self.tape_ptr, min_clamp);

        // Ok(())
    }

    fn unsafe_move_right(&mut self, amount: i64) -> Result<(), BuilderError> {
        let val = self.ptr.const_int(amount as u64, true);

        let ptr = self.get_tape_ptr("get_ptr__move_right")?;
        let ptr = self.b.build_ptr_to_int(ptr, self.ptr, "p2i_movr")?;
        let ptr = self.b.build_int_add(ptr, val, "move_right")?;

        self.set_tape_ptr(ptr)?;

        Ok(())
    }

    fn unsafe_copy_loop(&mut self, values: &BTreeMap<i64, i64>) -> Result<(), BuilderError> {
        let ptr = self.get_tape_ptr("get_ptr__copy_loop")?;
        let ptr_i = self.b.build_ptr_to_int(ptr, self.ptr, "p2i_copy_loop")?;

        let value = self
            .b
            .build_load(self.byte, ptr, "load__copy_loop")?
            .into_int_value();

        for (offset, mul) in values {
            let offset = self.ptr.const_int(*offset as u64, true);
            let mul = self.byte.const_int(*mul as u64, true);

            let addr = self.b.build_int_add(ptr_i, offset, "copy_loop_addr")?;

            let addr = self
                .b
                .build_int_to_ptr(addr, self.ptr_ty, "i2p__copy_loop_val")?;

            let cur = self
                .b
                .build_load(self.byte, addr, "copy_loop_get_byte")?
                .into_int_value();

            // let cur = self
            //     .b
            //     .build_int_z_extend(cur, self.long, "extend__copy_loop_val")?;

            let add = self.b.build_int_mul(value, mul, "copy_loop_mul")?;
            let res = self.b.build_int_add(cur, add, "copy_loop_add")?;

            let res = self
                .b
                .build_int_truncate(res, self.byte, "copy_loop_trunc")?;

            self.b.build_store(addr, res)?;
        }

        let zero = self.byte.const_zero();

        self.b.build_store(ptr, zero)?;

        Ok(())
    }
}
