use crate::{TAPE_SIZE, optimizer::OptAction};
use cranelift::{
    codegen::{
        Context,
        ir::{FuncRef, StackSlot},
        write_function,
    },
    prelude::{
        AbiParam, Configurable, FunctionBuilder, FunctionBuilderContext, InstBuilder, IntCC,
        MemFlags, StackSlotData, StackSlotKind, Type, Value, Variable,
        settings::{self, Flags},
        types,
    },
};
use cranelift_codegen::control::ControlPlane;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module, default_libcall_names};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::{fs, mem, path::PathBuf};
use target_lexicon::Triple;

pub struct CompilerOptions {
    pub unsafe_mode: bool,
    pub output_clif: Option<PathBuf>,
    pub output_asm: Option<PathBuf>,
}

pub fn jit_compile(actions: &Vec<OptAction>, opts: CompilerOptions) -> fn() -> () {
    let mut flags = settings::builder();

    flags.set("use_colocated_libcalls", "false").unwrap();
    flags.set("is_pic", "false").unwrap();

    let isa = cranelift_native::builder().unwrap();
    let isa = isa.finish(Flags::new(flags)).unwrap();
    let builder = JITBuilder::with_isa(isa, default_libcall_names());
    let module = JITModule::new(builder);
    let mut compiler = Compiler::new(module, false, opts);
    let id = compiler.compile(actions);

    compiler.module.finalize_definitions().unwrap();

    let code = compiler.module.get_finalized_function(id);
    let func = unsafe { mem::transmute(code) };

    func
}

pub fn aot_compile(actions: &Vec<OptAction>, target: &Triple, opts: CompilerOptions) -> Vec<u8> {
    let mut flags = settings::builder();

    flags.set("use_colocated_libcalls", "false").unwrap();
    flags.set("is_pic", "true").unwrap();
    flags.set("opt_level", "speed").unwrap();
    flags.set("regalloc_checker", "false").unwrap();
    flags.set("enable_alias_analysis", "true").unwrap();
    flags.set("enable_verifier", "true").unwrap();
    flags.set("enable_probestack", "false").unwrap();
    flags.set("unwind_info", "false").unwrap();

    let isa = cranelift::codegen::isa::lookup(target.clone()).unwrap();
    let isa = isa.finish(Flags::new(flags)).unwrap();
    let builder = ObjectBuilder::new(isa, "code.o", default_libcall_names()).unwrap();
    let module = ObjectModule::new(builder);
    let mut compiler = Compiler::new(module, true, opts);
    let _id = compiler.compile(actions);
    let obj = compiler.module.finish();
    let obj = obj.emit().unwrap();

    obj
}

pub struct Compiler<M: Module> {
    fcx: FunctionBuilderContext,
    cx: Context,
    module: M,
    needs_exit: bool,
    opts: CompilerOptions,
}

impl<M: Module> Compiler<M> {
    pub fn new(module: M, needs_exit: bool, opts: CompilerOptions) -> Self {
        Self {
            cx: module.make_context(),
            fcx: FunctionBuilderContext::new(),
            module,
            needs_exit,
            opts,
        }
    }

    pub fn write_debug_output(&mut self) {
        if let Some(path) = &self.opts.output_clif {
            let mut buf = String::new();

            buf.push_str(&format!("target {}\n\n", self.module.isa().triple()));

            for flag in self.module.isa().flags().iter() {
                buf.push_str(&format!("set {flag}\n"));
            }

            buf.push('\n');
            write_function(&mut buf, &self.cx.func).unwrap();
            buf.push('\n');

            fs::write(path, buf).unwrap();
        }

        if let Some(path) = &self.opts.output_asm {
            let code = self
                .cx
                .compile(self.module.isa(), &mut ControlPlane::default())
                .unwrap();

            let isa = self.module.isa();
            let cap = isa.to_capstone().unwrap();
            let disas = cap.disasm_all(code.code_buffer(), 0).unwrap().to_string();

            fs::write(path, disas).unwrap();
        }
    }

    pub fn compile(&mut self, actions: &Vec<OptAction>) -> FuncId {
        let mut fb = FunctionBuilder::new(&mut self.cx.func, &mut self.fcx);
        let entry = fb.create_block();

        fb.append_block_params_for_function_params(entry);
        fb.switch_to_block(entry);
        fb.seal_block(entry);

        let mut cg = CodeGenerator::new(fb, &mut self.module, self.needs_exit, &self.opts);

        cg.compile(actions);
        cg.fb.ins().return_(&[]);
        cg.fb.finalize();

        let id = self
            .module
            .declare_function("_start", Linkage::Export, &self.cx.func.signature)
            .unwrap();

        self.module.define_function(id, &mut self.cx).unwrap();
        self.write_debug_output();
        self.module.clear_context(&mut self.cx);

        id
    }
}

pub struct CodeGenerator<'a, M: Module> {
    byte: Type,
    ptr: Type,
    fb: FunctionBuilder<'a>,
    module: &'a mut M,
    needs_exit: bool,
    opts: &'a CompilerOptions,

    // brainfuck stuff
    tape: StackSlot,
    tape_ptr: Variable,

    // functions
    putchar: FuncRef,
    getchar: FuncRef,
    exit: FuncRef,
}

impl<'a, M: Module> CodeGenerator<'a, M> {
    pub fn new(
        mut fb: FunctionBuilder<'a>,
        module: &'a mut M,
        needs_exit: bool,
        opts: &'a CompilerOptions,
    ) -> Self {
        let ptr = module.target_config().pointer_type();
        let byte = types::I8;

        let tape_data = StackSlotData::new(StackSlotKind::ExplicitSlot, TAPE_SIZE as u32, 0);
        let tape = fb.create_sized_stack_slot(tape_data);
        let tape_addr = fb.ins().stack_addr(types::I64, tape, 0);
        let zero = fb.ins().iconst(byte, 0);
        let size = fb.ins().iconst(types::I64, TAPE_SIZE as i64);

        fb.call_memset(module.target_config(), tape_addr, zero, size);

        let tape_ptr = if opts.unsafe_mode {
            let tape_ptr = fb.declare_var(ptr);

            fb.def_var(tape_ptr, tape_addr);
            tape_ptr
        } else {
            let zero = fb.ins().iconst(ptr, 0);
            let tape_ptr = fb.declare_var(ptr);

            fb.def_var(tape_ptr, zero);
            tape_ptr
        };

        let mut sig = module.make_signature();

        sig.params.push(AbiParam::new(types::I32));
        sig.returns.push(AbiParam::new(types::I32));

        let putchar = module
            .declare_function("putchar", Linkage::Import, &sig)
            .unwrap();

        let putchar = module.declare_func_in_func(putchar, fb.func);

        let mut sig = module.make_signature();

        sig.returns.push(AbiParam::new(types::I32));

        let getchar = module
            .declare_function("getchar", Linkage::Import, &sig)
            .unwrap();

        let getchar = module.declare_func_in_func(getchar, fb.func);

        let mut sig = module.make_signature();

        sig.params.push(AbiParam::new(types::I32));

        let exit = module
            .declare_function("exit", Linkage::Import, &sig)
            .unwrap();

        let exit = module.declare_func_in_func(exit, fb.func);

        Self {
            byte,
            ptr,
            fb,
            module,
            tape,
            tape_ptr,
            putchar,
            getchar,
            exit,
            needs_exit,
            opts,
        }
    }

    pub fn compile(&mut self, actions: &Vec<OptAction>) {
        for insn in actions {
            self.translate(insn);
        }

        if self.needs_exit {
            let zero = self.fb.ins().iconst(types::I32, 0);

            self.fb.ins().call(self.exit, &[zero]);
        }
    }

    pub fn translate(&mut self, insn: &OptAction) {
        match insn {
            OptAction::Noop => (),
            OptAction::Right => self.move_right(1),
            OptAction::Left => self.move_left(1),
            OptAction::Inc => self.add_slot(1),
            OptAction::Dec => self.add_slot(-1),
            OptAction::Output => self.print_slot(),
            OptAction::Input => self.input_slot(),
            OptAction::Loop(actions) => self.translate_loop(actions),
            OptAction::AddValue(v) => self.add_slot(*v as i64),
            OptAction::SubValue(v) => self.add_slot(-(*v as i64)),
            OptAction::SetValue(v) => self.set_slot(*v as i64),
            OptAction::MoveRight(v) => self.move_right(*v),
            OptAction::MoveLeft(v) => self.move_left(*v),
            OptAction::ZeroRight(v) => self.zero_right(*v as i64),
        }
    }

    pub fn translate_loop(&mut self, actions: &Vec<OptAction>) {
        let header = self.fb.create_block();
        let body = self.fb.create_block();
        let exit = self.fb.create_block();

        self.fb.ins().jump(header, &[]);
        self.fb.switch_to_block(header);

        let value = self.read_from_arr();
        let cond = self.fb.ins().icmp_imm(IntCC::NotEqual, value, 0);

        self.fb.ins().brif(cond, body, &[], exit, &[]);

        self.fb.switch_to_block(body);
        self.fb.seal_block(body);

        for action in actions {
            self.translate(action);
        }

        self.fb.ins().jump(header, &[]);
        self.fb.switch_to_block(exit);
        self.fb.seal_block(header);
        self.fb.seal_block(exit);
    }

    // "Unsafe" methods use the tape_ptr as a literal pointer instead of an index into the tape array

    pub fn print_slot(&mut self) {
        let value = self.read_from_arr();
        let value = self.fb.ins().uextend(types::I32, value);

        self.fb.ins().call(self.putchar, &[value]);
    }

    pub fn input_slot(&mut self) {
        let call = self.fb.ins().call(self.getchar, &[]);
        let value = self.fb.inst_results(call)[0];
        let value = self.fb.ins().ireduce(self.byte, value);

        self.write_to_arr(value);
    }

    pub fn write_to_arr(&mut self, value: Value) {
        if self.opts.unsafe_mode {
            return self.unsafe_write_to_arr(value);
        }

        let offset = self.fb.use_var(self.tape_ptr);
        let base_addr = self.fb.ins().stack_addr(self.ptr, self.tape, 0);
        let final_addr = self.fb.ins().iadd(base_addr, offset);

        self.fb.ins().store(MemFlags::new(), value, final_addr, 0);
    }

    pub fn unsafe_write_to_arr(&mut self, value: Value) {
        let base_addr = self.fb.use_var(self.tape_ptr);

        self.fb.ins().store(MemFlags::new(), value, base_addr, 0);
    }

    pub fn add_slot(&mut self, amount: i64) {
        if self.opts.unsafe_mode {
            return self.unsafe_add_slot(amount);
        }

        let offset = self.fb.use_var(self.tape_ptr);
        let base_addr = self.fb.ins().stack_addr(self.ptr, self.tape, 0);
        let final_addr = self.fb.ins().iadd(base_addr, offset);

        let value = self
            .fb
            .ins()
            .load(self.byte, MemFlags::new(), final_addr, 0);

        let value = self.fb.ins().iadd_imm(value, amount);

        self.fb.ins().store(MemFlags::new(), value, final_addr, 0);
    }

    pub fn unsafe_add_slot(&mut self, amount: i64) {
        let base_addr = self.fb.use_var(self.tape_ptr);
        let value = self.fb.ins().load(self.byte, MemFlags::new(), base_addr, 0);
        let value = self.fb.ins().iadd_imm(value, amount);

        self.fb.ins().store(MemFlags::new(), value, base_addr, 0);
    }

    pub fn set_slot(&mut self, value: i64) {
        if self.opts.unsafe_mode {
            return self.unsafe_set_slot(value);
        }

        let offset = self.fb.use_var(self.tape_ptr);
        let base_addr = self.fb.ins().stack_addr(self.ptr, self.tape, 0);
        let final_addr = self.fb.ins().iadd(base_addr, offset);
        let value = self.fb.ins().iconst(types::I64, value);
        let value = self.fb.ins().ireduce(self.byte, value);

        self.fb.ins().store(MemFlags::new(), value, final_addr, 0);
    }

    pub fn unsafe_set_slot(&mut self, value: i64) {
        let base_addr = self.fb.use_var(self.tape_ptr);
        let value = self.fb.ins().iconst(types::I64, value);
        let value = self.fb.ins().ireduce(self.byte, value);

        self.fb.ins().store(MemFlags::new(), value, base_addr, 0);
    }

    pub fn zero_right(&mut self, length: i64) {
        if self.opts.unsafe_mode {
            return self.unsafe_zero_right(length);
        }

        let offset = self.fb.use_var(self.tape_ptr);
        let base_addr = self.fb.ins().stack_addr(self.ptr, self.tape, 0);
        let final_addr = self.fb.ins().iadd(base_addr, offset);
        let zero = self.fb.ins().iconst(self.byte, 0);
        let size = self.fb.ins().iconst(types::I64, length);

        self.fb
            .call_memset(self.module.target_config(), final_addr, zero, size);
    }

    pub fn unsafe_zero_right(&mut self, length: i64) {
        let base_addr = self.fb.use_var(self.tape_ptr);
        let zero = self.fb.ins().iconst(self.byte, 0);
        let size = self.fb.ins().iconst(types::I64, length);

        self.fb
            .call_memset(self.module.target_config(), base_addr, zero, size);
    }

    pub fn read_from_arr(&mut self) -> Value {
        if self.opts.unsafe_mode {
            return self.unsafe_read_from_arr();
        }

        let offset = self.fb.use_var(self.tape_ptr);
        let base_addr = self.fb.ins().stack_addr(self.ptr, self.tape, 0);
        let final_addr = self.fb.ins().iadd(base_addr, offset);

        self.fb
            .ins()
            .load(self.byte, MemFlags::new(), final_addr, 0)
    }

    pub fn unsafe_read_from_arr(&mut self) -> Value {
        let base_addr = self.fb.use_var(self.tape_ptr);

        self.fb.ins().load(self.byte, MemFlags::new(), base_addr, 0)
    }

    pub fn move_right(&mut self, amount: i64) {
        if self.opts.unsafe_mode {
            return self.unsafe_move_right(amount);
        }

        let value = self.fb.use_var(self.tape_ptr);
        let value = self.fb.ins().iadd_imm(value, amount as i64);

        let did_hit = self.fb.ins().icmp_imm(
            IntCC::UnsignedGreaterThanOrEqual,
            value,
            (TAPE_SIZE - 1) as i64,
        );

        let zero = self.fb.ins().iconst(self.ptr, 0);
        let wrapped = self.fb.ins().select(did_hit, zero, value);

        self.fb.def_var(self.tape_ptr, wrapped);
    }

    pub fn unsafe_move_right(&mut self, amount: i64) {
        let base_addr = self.fb.use_var(self.tape_ptr);
        let new_addr = self.fb.ins().iadd_imm(base_addr, amount as i64);

        self.fb.def_var(self.tape_ptr, new_addr);
    }

    pub fn move_left(&mut self, amount: i64) {
        if self.opts.unsafe_mode {
            return self.unsafe_move_left(amount);
        }

        let value = self.fb.use_var(self.tape_ptr);

        let did_hit = self
            .fb
            .ins()
            .icmp_imm(IntCC::UnsignedLessThanOrEqual, value, 0);

        let value = self.fb.ins().iadd_imm(value, -(amount as i64));
        let max = self.fb.ins().iconst(self.ptr, (TAPE_SIZE - 1) as i64);
        let wrapped = self.fb.ins().select(did_hit, max, value);

        self.fb.def_var(self.tape_ptr, wrapped);
    }

    pub fn unsafe_move_left(&mut self, amount: i64) {
        let base_addr = self.fb.use_var(self.tape_ptr);
        let new_addr = self.fb.ins().iadd_imm(base_addr, -(amount as i64));

        self.fb.def_var(self.tape_ptr, new_addr);
    }
}
