mod simd;

use crate::{
    TAPE_SIZE,
    compiler::{CompilerOptions, TestingIo},
    interp::wrapping_conv,
    opt::OptAction,
};
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
use std::{collections::BTreeMap, fs, mem};
use target_lexicon::Triple;

pub fn jit_compile_run(
    actions: &Vec<OptAction>,
    opts: CompilerOptions,
    _testing_io: Option<Box<&dyn TestingIo>>,
) {
    jit_compile(actions, opts, _testing_io)();
}

pub fn jit_compile(
    actions: &Vec<OptAction>,
    opts: CompilerOptions,
    _testing_io: Option<Box<&dyn TestingIo>>,
) -> fn() -> () {
    let mut flags = settings::builder();

    flags.set("use_colocated_libcalls", "false").unwrap();
    flags.set("is_pic", "false").unwrap();

    let isa = cranelift_native::builder().unwrap();
    let isa = isa.finish(Flags::new(flags)).unwrap();

    #[allow(unused_mut)]
    let mut builder = JITBuilder::with_isa(isa, default_libcall_names());

    #[cfg(feature = "testing")]
    if let Some(io) = _testing_io {
        builder.symbol("putchar", io.putchar());
        builder.symbol("getchar", io.getchar());
    }

    let module = JITModule::new(builder);
    let mut compiler = Compiler::new(module, false, opts);
    let id = compiler.compile(actions);

    compiler.module.finalize_definitions().unwrap();

    let code = compiler.module.get_finalized_function(id);
    let func: fn() -> () = unsafe { mem::transmute(code) };

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
        if let Some(path) = &self.opts.output_ir {
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
        cg.b.ins().return_(&[]);
        cg.b.finalize();

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

#[allow(unused)]
pub struct CodeGenerator<'a, M: Module> {
    byte: Type,
    ptr: Type,
    b: FunctionBuilder<'a>,
    needs_exit: bool,
    module: &'a mut M,
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

        let tape_ptr = fb.declare_var(ptr);

        fb.def_var(tape_ptr, tape_addr);

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
            b: fb,
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
            let zero = self.b.ins().iconst(types::I32, 0);

            self.b.ins().call(self.exit, &[zero]);
        }
    }

    fn translate(&mut self, insn: &OptAction) {
        match insn {
            OptAction::Noop => (),
            OptAction::Output => self.print_slot(),
            OptAction::Input => self.input_slot(),
            OptAction::Loop(actions) => self.translate_loop(actions),
            OptAction::AddValue(v) => self.add_slot(*v as i64),
            OptAction::SetValue(v) => self.set_slot(*v as i64),
            OptAction::MovePtr(v) => self.move_ptr(*v),
            OptAction::SetAndMove(v, o) => self.set_move(*v, *o),
            OptAction::AddAndMove(v, o) => self.add_move(*v, *o),
            OptAction::SimdAddMove(a, o) => self.unsafe_simd_add_arr_move(a, *o),
            OptAction::BulkPrint(n) => self.bulk_print(*n),
            OptAction::CopyLoop(v) => self.copy_loop(&v),
        }
    }

    fn translate_loop(&mut self, actions: &Vec<OptAction>) {
        let header = self.b.create_block();
        let body = self.b.create_block();
        let exit = self.b.create_block();

        self.b.ins().jump(header, &[]);
        self.b.switch_to_block(header);

        let value = self.read_from_arr();
        let cond = self.b.ins().icmp_imm(IntCC::NotEqual, value, 0);

        self.b.ins().brif(cond, body, &[], exit, &[]);

        self.b.switch_to_block(body);
        self.b.seal_block(body);

        for action in actions {
            self.translate(action);
        }

        self.b.ins().jump(header, &[]);
        self.b.switch_to_block(exit);
        self.b.seal_block(header);
        self.b.seal_block(exit);
    }

    // "Unsafe" methods use the tape_ptr as a literal pointer instead of an index into the tape array

    fn print_slot(&mut self) {
        let value = self.read_from_arr();
        let value = self.b.ins().uextend(types::I32, value);

        self.b.ins().call(self.putchar, &[value]);
    }

    fn bulk_print(&mut self, n: i64) {
        let value = self.read_from_arr();
        let value = self.b.ins().uextend(types::I32, value);

        for _ in 0..n {
            self.b.ins().call(self.putchar, &[value]);
        }
    }

    fn input_slot(&mut self) {
        let call = self.b.ins().call(self.getchar, &[]);
        let value = self.b.inst_results(call)[0];
        let value = self.b.ins().ireduce(self.byte, value);

        self.write_to_arr(value);
    }

    fn write_to_arr(&mut self, value: Value) {
        let base_addr = self.b.use_var(self.tape_ptr);

        self.b.ins().store(MemFlags::new(), value, base_addr, 0);
    }

    fn add_slot(&mut self, amount: i64) {
        let base_addr = self.b.use_var(self.tape_ptr);
        let value = self.b.ins().load(self.byte, MemFlags::new(), base_addr, 0);
        let value = self.b.ins().iadd_imm(value, amount);

        self.b.ins().store(MemFlags::new(), value, base_addr, 0);
    }

    fn set_slot(&mut self, value: i64) {
        let base_addr = self.b.use_var(self.tape_ptr);
        let value = self.b.ins().iconst(self.byte, wrapping_conv(value) as i64);

        self.b.ins().store(MemFlags::new(), value, base_addr, 0);
    }

    fn set_move(&mut self, value: i64, offset: i64) {
        let base_addr = self.b.use_var(self.tape_ptr);
        let post = self.b.ins().iadd_imm(base_addr, offset);
        let value = self.b.ins().iconst(self.byte, wrapping_conv(value) as i64);

        self.b.ins().store(MemFlags::new(), value, base_addr, 0);
        self.b.def_var(self.tape_ptr, post);
    }

    fn add_move(&mut self, amount: i64, offset: i64) {
        let base_addr = self.b.use_var(self.tape_ptr);
        let post = self.b.ins().iadd_imm(base_addr, offset);
        let value = self.b.ins().load(self.byte, MemFlags::new(), base_addr, 0);
        let value = self.b.ins().iadd_imm(value, amount);

        self.b.ins().store(MemFlags::new(), value, base_addr, 0);
        self.b.def_var(self.tape_ptr, post);
    }

    fn read_from_arr(&mut self) -> Value {
        let base_addr = self.b.use_var(self.tape_ptr);

        self.b.ins().load(self.byte, MemFlags::new(), base_addr, 0)
    }

    fn move_ptr(&mut self, amount: i64) {
        let base_addr = self.b.use_var(self.tape_ptr);
        let new_addr = self.b.ins().iadd_imm(base_addr, amount);

        self.b.def_var(self.tape_ptr, new_addr);
    }

    fn copy_loop(&mut self, values: &BTreeMap<i64, i64>) {
        let base_addr = self.b.use_var(self.tape_ptr);
        let value = self.b.ins().load(self.byte, MemFlags::new(), base_addr, 0);

        for (offset, mul) in values {
            let addr = self.b.ins().iadd_imm(base_addr, *offset);
            let cur = self.b.ins().load(self.byte, MemFlags::new(), addr, 0);
            let additional = self.b.ins().imul_imm(value, *mul);
            let result = self.b.ins().iadd(cur, additional);

            self.b.ins().store(MemFlags::new(), result, addr, 0);
        }

        let zero = self.b.ins().iconst(self.byte, 0);

        self.b.ins().store(MemFlags::new(), zero, base_addr, 0);
    }
}
