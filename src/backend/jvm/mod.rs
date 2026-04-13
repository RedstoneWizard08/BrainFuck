//! JVM bytecode generation backend.
//!
//! This backend compiles Brainf*ck to JVM bytecode for execution
//! on the Java Virtual Machine.

/// Copy loop optimization module
mod copy;
/// I/O operation implementation module
mod io;
/// Loop construct implementation module
mod loops;
/// Pointer manipulation implementation module
mod ptr;
/// Value operation implementation module
mod value;

use std::{borrow::Cow, collections::HashMap, io::Cursor, ops::Div};

use log::warn;
use ristretto_classfile::{
    ClassAccessFlags, ClassFile, Constant, ConstantPool, JavaStr, Method, MethodAccessFlags,
    Version,
    attributes::{ArrayType, Attribute, Instruction},
};

use crate::{
    backend::CompilerOptions,
    opt::action::{OptAction, ValueAction},
};

pub const trait NumUtil {
    fn digits(&self) -> usize;
}

impl const NumUtil for i64 {
    fn digits(&self) -> usize {
        (self.ilog10() as usize) + 1
    }
}

impl const NumUtil for usize {
    fn digits(&self) -> usize {
        (self.ilog10() as usize) + 1
    }
}

const LOOP_START_PREFIX: &[u8] = b"loop_start_";
const LOOP_END_PREFIX: &[u8] = b"loop_end_";

const MAX_LOOPS: usize = 9999;
const LOOP_START_LEN: usize = LOOP_START_PREFIX.len() + MAX_LOOPS.digits();
const LOOP_END_LEN: usize = LOOP_END_PREFIX.len() + MAX_LOOPS.digits();

const fn create_loop_start_name(id: usize) -> [u8; LOOP_START_LEN] {
    assert!(id <= MAX_LOOPS, "ID out of range of max loop count!");

    let mut buf = [b'0'; LOOP_START_LEN];

    buf.copy_from_slice(LOOP_START_PREFIX);

    let start = LOOP_START_PREFIX.len();
    let digits = id.digits();
    let mut i = 0;

    while i < digits {
        let mask = 10_usize.pow(i as u32);
        let digit = id.div(mask);
        let digit_byte = b'0' + digit as u8;

        buf[start + i] = digit_byte;

        i += 1;
    }

    buf
}

const fn create_loop_end_name(id: usize) -> [u8; LOOP_END_LEN] {
    assert!(id <= MAX_LOOPS, "ID out of range of max loop count!");

    let mut buf = [b'0'; LOOP_END_LEN];

    buf.copy_from_slice(LOOP_END_PREFIX);

    let start = LOOP_END_PREFIX.len();
    let digits = id.digits();
    let mut i = 0;

    while i < digits {
        let mask = 10_usize.pow(i as u32);
        let digit = id.div(mask);
        let digit_byte = b'0' + digit as u8;

        buf[start + i] = digit_byte;

        i += 1;
    }

    buf
}

const unsafe fn jstr_from_bytes<const N: usize>(bytes: [u8; N]) -> &'static JavaStr {
    // SAFETY: caller guarantees bytes are valid MUTF-8
    unsafe { &*(&bytes as *const [u8] as *const JavaStr) }
}

const fn create_loop_start_name_str(id: usize) -> Cow<'static, JavaStr> {
    let bytes = create_loop_start_name(id);

    Cow::Borrowed(unsafe { jstr_from_bytes(bytes) })
}

const fn create_loop_end_name_str(id: usize) -> Cow<'static, JavaStr> {
    let bytes = create_loop_end_name(id);

    Cow::Borrowed(unsafe { jstr_from_bytes(bytes) })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConstData {
    I64(i64),
    LoopStart(usize),
    LoopEnd(usize),
}

impl Into<Constant<'static>> for ConstData {
    fn into(self) -> Constant<'static> {
        match self {
            ConstData::I64(it) => Constant::Integer(it as i32),
            ConstData::LoopStart(id) => Constant::Utf8(create_loop_start_name_str(id)),
            ConstData::LoopEnd(id) => Constant::Utf8(create_loop_end_name_str(id)),
        }
    }
}

#[allow(unused)]
pub struct CodeGenerator<'a> {
    pub(self) insns: Vec<Instruction>,
    pub(self) pos: usize,

    pool: ConstantPool<'static>,
    pool_map: HashMap<ConstData, u16>,
    opts: &'a CompilerOptions,
    block: usize,
    locals: usize,
    known_nonzero: bool,
    known_zero: bool,

    // Local IDs
    arr_id: usize,
    ptr_id: usize,

    id_system: u16,
    id_printstream: u16,
    id_inputstream: u16,

    id_system_out: u16,
    id_system_in: u16,

    id_printstream_append: u16,
    id_inputstream_read: u16,
}

impl Into<ConstData> for i64 {
    fn into(self) -> ConstData {
        ConstData::I64(self)
    }
}

impl<'a> CodeGenerator<'a> {
    pub fn run(opts: &'a CompilerOptions, actions: &Vec<OptAction>) -> Vec<u8> {
        warn!("The JVM backend is unstable and may not work! Do not use in production!");

        let mut pool = ConstantPool::new();

        let name = pool.add_utf8("main").unwrap();
        let sig = pool.add_utf8("([Ljava/lang/String;)V").unwrap();

        let id_system = pool.add_class("java/lang/System").unwrap();
        let id_printstream = pool.add_class("java/io/PrintStream").unwrap();
        let id_inputstream = pool.add_class("java/io/InputStream").unwrap();

        let id_system_out = pool
            .add_field_ref(id_system, "out", "Ljava/io/PrintStream;")
            .unwrap();

        let id_system_in = pool
            .add_field_ref(id_system, "in", "Ljava/io/InputStream;")
            .unwrap();

        let id_printstream_append = pool
            .add_method_ref(id_printstream, "append", "(C)Ljava/io/PrintStream;")
            .unwrap();

        let id_inputstream_read = pool.add_method_ref(id_inputstream, "read", "()I").unwrap();

        let code = pool.add_utf8("Code").unwrap();
        let sup = pool.add_class("java/lang/Object").unwrap();
        let cn = pool.add_class("Test").unwrap();

        let mut me = Self {
            opts,
            pos: 0,
            insns: Vec::new(),
            pool,
            pool_map: HashMap::new(),
            block: 0,
            locals: 3,
            known_nonzero: false,
            known_zero: false,
            arr_id: 1,
            ptr_id: 2,

            id_system,
            id_printstream,
            id_inputstream,

            id_system_out,
            id_system_in,

            id_printstream_append,
            id_inputstream_read,
        };

        me.compile(actions);

        let method = Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: name,
            descriptor_index: sig,
            attributes: [Attribute::Code {
                name_index: code,
                max_stack: 256,
                max_locals: me.locals as u16,
                code: me.insns,
                exception_table: vec![],
                attributes: vec![],
            }]
            .to_vec(),
        };

        let cf = ClassFile {
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER,
            code_source_url: None,
            attributes: vec![],
            constant_pool: me.pool,
            fields: vec![],
            interfaces: vec![],
            methods: vec![method],
            super_class: sup,
            this_class: cn,
            version: Version::Java21 { minor: 0 },
        };

        cf.verify().unwrap();

        let mut bytes = Vec::new();

        cf.to_bytes(&mut bytes).unwrap();

        bytes
    }

    pub(self) fn add(&mut self, insn: Instruction) {
        let mut buf = Cursor::new(Vec::new());

        insn.to_bytes(&mut buf).unwrap();

        self.pos += buf.into_inner().len();
        self.insns.push(insn);
    }

    pub(self) fn value<T: Into<ConstData>>(&mut self, it: T) -> u16 {
        let item = it.into();

        *self
            .pool_map
            .entry(item)
            .or_insert_with_key(|it| self.pool.add((*it).into()).unwrap())
    }

    pub(self) fn ldc<T: Into<ConstData>>(&mut self, it: T) {
        let id = self.value(it);

        if id <= u8::MAX as u16 {
            self.add(Instruction::Ldc(id as u8));
        } else {
            self.add(Instruction::Ldc_w(id));
        }
    }

    fn compile(&mut self, actions: &Vec<OptAction>) {
        // arr
        self.ldc(65536);
        self.add(Instruction::Newarray(ArrayType::Byte));
        self.add(Instruction::Astore_1);

        // ptr
        self.add(Instruction::Iconst_0);
        self.add(Instruction::Istore_2);

        for insn in actions {
            self.translate(insn);
        }

        self.add(Instruction::Return);
    }

    fn translate(&mut self, insn: &OptAction) {
        match insn {
            OptAction::Noop => (),

            OptAction::Value(it) => match it {
                ValueAction::Output => self.print_slot(),
                ValueAction::Input => self.input_slot(),
                ValueAction::AddValue(v) => self.add_slot(*v),
                ValueAction::BulkPrint(n) => self.bulk_print(*n),

                ValueAction::SetValue(v) => {
                    if *v == 0 {
                        self.known_zero = true;
                        self.known_nonzero = false;
                    } else {
                        self.known_zero = false;
                        self.known_nonzero = true;
                    }

                    self.set_slot(*v)
                }
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

        match insn {
            OptAction::Value(ValueAction::SetValue(_)) => {}

            _ => {
                self.known_zero = false;
                self.known_nonzero = false;
            }
        }
    }
}
