use crate::{TAPE_SIZE, optimizer::OptAction};
use cranelift::{
    codegen::{
        Context,
        ir::{FuncRef, StackSlot},
    },
    prelude::{
        AbiParam, Configurable, FunctionBuilder, FunctionBuilderContext, InstBuilder, IntCC,
        MemFlags, StackSlotData, StackSlotKind, Type, Value, Variable,
        settings::{self, Flags},
        types,
    },
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module, default_libcall_names};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::mem;
use target_lexicon::Triple;

pub fn jit_compile(actions: &Vec<OptAction>) -> fn() -> () {
    let mut flags = settings::builder();

    flags.set("use_colocated_libcalls", "false").unwrap();
    flags.set("is_pic", "false").unwrap();

    let isa = cranelift_native::builder().unwrap();
    let isa = isa.finish(Flags::new(flags)).unwrap();
    let builder = JITBuilder::with_isa(isa, default_libcall_names());
    let module = JITModule::new(builder);
    let compiler = Compiler::new(module, false);
    let (id, mut module) = compiler.compile(actions);

    module.finalize_definitions().unwrap();

    let code = module.get_finalized_function(id);

    unsafe { mem::transmute(code) }
}

pub fn aot_compile(actions: &Vec<OptAction>, target: &Triple) -> Vec<u8> {
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
    let compiler = Compiler::new(module, true);
    let (_id, module) = compiler.compile(actions);
    let obj = module.finish();
    let obj = obj.emit().unwrap();

    obj
}

pub struct Compiler<M: Module> {
    fcx: FunctionBuilderContext,
    cx: Context,
    module: M,
    needs_exit: bool,
}

impl<M: Module> Compiler<M> {
    pub fn new(module: M, needs_exit: bool) -> Self {
        Self {
            cx: module.make_context(),
            fcx: FunctionBuilderContext::new(),
            module,
            needs_exit,
        }
    }

    pub fn compile(mut self, actions: &Vec<OptAction>) -> (FuncId, M) {
        let mut fb = FunctionBuilder::new(&mut self.cx.func, &mut self.fcx);
        let entry = fb.create_block();

        fb.append_block_params_for_function_params(entry);
        fb.switch_to_block(entry);
        fb.seal_block(entry);

        let mut cg = CodeGenerator::new(fb, &mut self.module, self.needs_exit);

        cg.compile(actions);
        cg.fb.ins().return_(&[]);
        cg.fb.finalize();

        let id = self
            .module
            .declare_function("_start", Linkage::Export, &self.cx.func.signature)
            .unwrap();

        self.module.define_function(id, &mut self.cx).unwrap();
        self.module.clear_context(&mut self.cx);

        (id, self.module)
    }
}

pub struct CodeGenerator<'a, M: Module> {
    byte: Type,
    ptr: Type,
    fb: FunctionBuilder<'a>,
    module: &'a mut M,
    needs_exit: bool,

    // brainfuck stuff
    tape: StackSlot,
    tape_ptr: Variable,

    // functions
    putchar: FuncRef,
    getchar: FuncRef,
    exit: FuncRef,
}

impl<'a, M: Module> CodeGenerator<'a, M> {
    pub fn new(mut fb: FunctionBuilder<'a>, module: &'a mut M, needs_exit: bool) -> Self {
        let ptr = module.target_config().pointer_type();
        let byte = types::I8;
        let zero = fb.ins().iconst(ptr, 0);
        let tape_ptr = fb.declare_var(ptr);

        fb.def_var(tape_ptr, zero);

        let tape_data = StackSlotData::new(StackSlotKind::ExplicitSlot, TAPE_SIZE as u32, 0);
        let tape = fb.create_sized_stack_slot(tape_data);
        let tape_addr = fb.ins().stack_addr(types::I64, tape, 0);
        let zero = fb.ins().iconst(byte, 0);
        let size = fb.ins().iconst(types::I64, TAPE_SIZE as i64);

        fb.call_memset(module.target_config(), tape_addr, zero, size);

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
            OptAction::Right => self.move_right(1),
            OptAction::Left => self.move_left(1),
            OptAction::Inc => self.add_slot(1),
            OptAction::Dec => self.add_slot(-1),
            OptAction::Output => self.print_slot(),
            OptAction::Input => self.input_slot(),
            OptAction::Loop(actions) => self.translate_loop(actions),
            OptAction::Noop => (),
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
        let offset = self.fb.use_var(self.tape_ptr);
        let base_addr = self.fb.ins().stack_addr(self.ptr, self.tape, 0);
        let final_addr = self.fb.ins().iadd(base_addr, offset);

        self.fb.ins().store(MemFlags::new(), value, final_addr, 0);
    }

    pub fn add_slot(&mut self, amount: i64) {
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

    pub fn set_slot(&mut self, value: i64) {
        let offset = self.fb.use_var(self.tape_ptr);
        let base_addr = self.fb.ins().stack_addr(self.ptr, self.tape, 0);
        let final_addr = self.fb.ins().iadd(base_addr, offset);
        let value = self.fb.ins().iconst(types::I64, value);
        let value = self.fb.ins().ireduce(self.byte, value);

        self.fb.ins().store(MemFlags::new(), value, final_addr, 0);
    }

    pub fn zero_right(&mut self, length: i64) {
        let offset = self.fb.use_var(self.tape_ptr);
        let base_addr = self.fb.ins().stack_addr(self.ptr, self.tape, 0);
        let final_addr = self.fb.ins().iadd(base_addr, offset);
        let zero = self.fb.ins().iconst(self.byte, 0);
        let size = self.fb.ins().iconst(types::I64, length);

        self.fb
            .call_memset(self.module.target_config(), final_addr, zero, size);
    }

    pub fn read_from_arr(&mut self) -> Value {
        let offset = self.fb.use_var(self.tape_ptr);
        let base_addr = self.fb.ins().stack_addr(self.ptr, self.tape, 0);
        let final_addr = self.fb.ins().iadd(base_addr, offset);

        self.fb
            .ins()
            .load(self.byte, MemFlags::new(), final_addr, 0)
    }

    pub fn move_right(&mut self, amount: usize) {
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

    pub fn move_left(&mut self, amount: usize) {
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
}
