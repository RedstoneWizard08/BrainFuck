mod copy;
mod io;
mod loops;
mod ptr;
mod value;

use inkwell::{
    AddressSpace, OptimizationLevel,
    attributes::{Attribute, AttributeLoc},
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    passes::PassBuilderOptions,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetTriple},
    types::{BasicType, PointerType},
    values::{BasicValue, FunctionValue, MetadataValue, PointerValue},
};
use itertools::Itertools;
use target_lexicon::{OperatingSystem, Triple};

use crate::{
    backend::CompilerOptions,
    opt::action::{OptAction, ValueAction},
};

const MAIN_NAME: &str = "_start";

type LlvmMain = unsafe extern "C" fn() -> ();

#[allow(unused)]
pub struct CodeGenerator<'a, 'c> {
    opts: CompilerOptions,
    cx: &'c Context,
    b: &'a Builder<'c>,
    module: &'a Module<'c>,
    tape: PointerValue<'c>,
    func: FunctionValue<'c>,
    ptr_ty: PointerType<'c>,
    putchar: FunctionValue<'c>,
    getchar: FunctionValue<'c>,
    exit: FunctionValue<'c>,
    tbaa_access: MetadataValue<'c>,
    tbaa_kind: u32,
}

impl<'a, 'c> CodeGenerator<'a, 'c> {
    fn compile(&mut self, actions: Vec<OptAction>) {
        for insn in actions {
            self.translate(&insn);
        }

        let code = self.cx.i32_type().const_int(0, false);

        self.b
            .build_call(self.exit, &[code.as_basic_value_enum().into()], "exit")
            .unwrap();

        self.b.build_unreachable().unwrap();
    }

    fn translate(&mut self, insn: &OptAction) {
        match insn {
            OptAction::Noop => (),

            OptAction::Value(it) => match it {
                ValueAction::Output => self.print_slot(),
                ValueAction::Input => self.input_slot(),
                ValueAction::AddValue(v) => self.add_slot(*v),
                ValueAction::BulkPrint(n) => self.bulk_print(*n),
                ValueAction::SetValue(v) => self.set_slot(*v),
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
            OptAction::CopyLoop(v) => self.copy_loop(&v),
            OptAction::Scan(s) => self.scan(*s),
        };
    }
}

pub fn compile(
    triple_opt: Triple,
    opts: CompilerOptions,
    actions: Vec<OptAction>,
    jit: bool,
) -> Vec<u8> {
    Target::initialize_all(&InitializationConfig::default());

    let triple = TargetTriple::create(&triple_opt.to_string());
    let target = Target::from_triple(&triple).unwrap();

    let machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            OptimizationLevel::Aggressive,
            RelocMode::PIC,
            CodeModel::Default,
        )
        .unwrap();

    let cx = Context::create();
    let b = cx.create_builder();
    let module = cx.create_module("bf");

    module.set_triple(&machine.get_triple());
    module.set_data_layout(&machine.get_target_data().get_data_layout());

    let noret = Attribute::get_named_enum_kind_id("noreturn");
    let noret = cx.create_enum_attribute(noret, 0);

    let nounw = Attribute::get_named_enum_kind_id("nounwind");
    let nounw = cx.create_enum_attribute(nounw, 0);

    let nocap = Attribute::get_named_enum_kind_id("nocapture");
    let nocap = cx.create_enum_attribute(nocap, 0);

    let int = cx.i32_type();
    let putchar_ty = int.fn_type(&[int.as_basic_type_enum().into()], false);
    let getchar_ty = int.fn_type(&[], false);

    let exit_ty = cx
        .void_type()
        .fn_type(&[int.as_basic_type_enum().into()], false);

    let putchar = if triple_opt.operating_system == OperatingSystem::Linux {
        module.add_function("putchar_unlocked", putchar_ty, None)
    } else {
        module.add_function("putchar", putchar_ty, None)
    };

    let getchar = module.add_function("getchar", getchar_ty, None);
    let exit = module.add_function("exit", exit_ty, None);

    putchar.add_attribute(AttributeLoc::Param(0), nocap);
    putchar.add_attribute(AttributeLoc::Function, nounw);
    getchar.add_attribute(AttributeLoc::Function, nounw);
    exit.add_attribute(AttributeLoc::Function, noret);

    let func_ty = cx.i32_type().fn_type(&[], false);
    let func = module.add_function(MAIN_NAME, func_ty, Some(Linkage::External));

    func.add_attribute(AttributeLoc::Function, nounw);

    let entry = cx.append_basic_block(func, "entry");

    b.position_at_end(entry);

    let ptr_ty = cx.ptr_type(AddressSpace::default());
    let arr_ty = cx.i8_type().array_type(65536);
    let tape_arr = b.build_alloca(arr_ty, "tape").unwrap();

    let memset = module.add_function(
        "llvm.memset.p0.i64",
        cx.void_type().fn_type(
            &[
                ptr_ty.into(),
                cx.i8_type().into(),
                cx.i64_type().into(),
                cx.bool_type().into(),
            ],
            false,
        ),
        None,
    );

    b.build_call(
        memset,
        &[
            tape_arr.into(),
            cx.i8_type().const_zero().into(),
            cx.i64_type().const_int(65536, false).into(),
            cx.bool_type().const_zero().into(),
        ],
        "",
    )
    .unwrap();

    let tape = b.build_alloca(ptr_ty, "tape_ptr").unwrap();

    b.build_store(tape, tape_arr).unwrap();

    let tbaa_root = cx.metadata_node(&[cx.metadata_string("BF TBAA").into()]);

    let tbaa_tape_type = cx.metadata_node(&[
        cx.metadata_string("tape").into(),
        tbaa_root.into(),
        cx.i64_type().const_int(0, false).into(),
    ]);

    let tbaa_access = cx.metadata_node(&[
        tbaa_tape_type.into(),
        tbaa_tape_type.into(),
        cx.i64_type().const_int(0, false).into(),
    ]);

    let tbaa_kind = cx.get_kind_id("tbaa");

    let mut cg = CodeGenerator {
        opts: opts.clone(),
        cx: &cx,
        b: &b,
        module: &module,
        tape,
        func,
        ptr_ty,
        putchar,
        getchar,
        exit,
        tbaa_access,
        tbaa_kind,
    };

    cg.compile(actions);

    if !func.verify(true) {
        unsafe {
            func.delete();
        }

        panic!("Failed to verify function!");
    }

    let mod_passes = &[
        "inferattrs",
        "function-attrs",
        "ipsccp",
        "globaldce",
        "globalopt",
    ];

    let passes = &[
        "sroa",
        "mem2reg",
        "early-cse<memssa>",
        "aggressive-instcombine",
        "instcombine<no-verify-fixpoint>",
        "simplifycfg",
        "reassociate",
        "nary-reassociate",
        "sccp",
        "correlated-propagation",
        "speculative-execution",
        "jump-threading",
        "memcpyopt",
        "loop-simplify",
        "loop-rotate",
        "loop-mssa(licm<allowspeculation>)",
        "loop-load-elim",
        "loop-idiom",
        "indvars",
        "loop-deletion",
        "loop-unroll",
        "bdce",
        "dse",
        "sink",
        "newgvn",
        "constraint-elimination",
        "adce",
        "tailcallelim",
        "instcombine<no-verify-fixpoint>",
        "simplifycfg",
    ];

    let mod_passes = std::iter::repeat_n(mod_passes.join(","), opts.opt_level as usize).join(",");
    let passes = std::iter::repeat_n(passes.join(","), opts.opt_level as usize).join(",");

    module
        .run_passes(&mod_passes, &machine, PassBuilderOptions::create())
        .unwrap();

    module
        .run_passes(&passes, &machine, PassBuilderOptions::create())
        .unwrap();

    if let Some(path) = opts.output_ir {
        std::fs::write(path, module.print_to_string().to_string()).unwrap();
    }

    if let Some(path) = opts.output_asm {
        let asm = machine
            .write_to_memory_buffer(&module, FileType::Assembly)
            .unwrap();

        let asm = asm.as_slice();

        std::fs::write(path, asm).unwrap();
    }

    if jit {
        // ========= JIT =========

        let jit = module
            .create_jit_execution_engine(OptimizationLevel::Aggressive)
            .unwrap();

        let func = unsafe { jit.get_function::<LlvmMain>(MAIN_NAME) }.unwrap();

        unsafe {
            func.call();
        }

        vec![]
    } else {
        // ========= AOT =========

        let buf = machine
            .write_to_memory_buffer(&module, FileType::Object)
            .unwrap();

        let file = buf.create_binary_file(Some(&cx)).unwrap();

        file.get_memory_buffer().as_slice().to_vec()
    }
}
