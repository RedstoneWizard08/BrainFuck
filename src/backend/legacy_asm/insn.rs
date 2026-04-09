#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TargetArch {
    X86_64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Reg {
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rsi,
    Rdi,
    Rbp,
    Rsp,

    Eax,
    Ebx,
    Ecx,
    Edx,
    Esi,
    Edi,
    Ebp,
    Esp,

    Ax,
    Bx,
    Cx,
    Dx,
    Si,
    Di,
    Bp,
    Sp,

    Al,
    Bl,
    Cl,
    Dl,
    Sil,
    Dil,
    Bpl,
    Spl,
}

impl Reg {
    pub fn name(&self, arch: TargetArch) -> &'static str {
        match arch {
            TargetArch::X86_64 => match self {
                Reg::Rax => "rax",
                Reg::Rcx => "rcx",
                Reg::Rdx => "rdx",
                Reg::Rbx => "rbx",
                Reg::Rsi => "rsi",
                Reg::Rdi => "rdi",
                Reg::Eax => "eax",
                Reg::Ecx => "ecx",
                Reg::Edx => "edx",
                Reg::Ebx => "ebx",
                Reg::Esi => "esi",
                Reg::Edi => "edi",
                Reg::Rbp => "rbp",
                Reg::Rsp => "rsp",
                Reg::Ebp => "ebp",
                Reg::Esp => "esp",
                Reg::Ax => "ax",
                Reg::Bx => "bx",
                Reg::Cx => "cx",
                Reg::Dx => "dx",
                Reg::Si => "si",
                Reg::Di => "di",
                Reg::Bp => "bp",
                Reg::Sp => "sp",
                Reg::Al => "al",
                Reg::Bl => "bl",
                Reg::Cl => "cl",
                Reg::Dl => "dl",
                Reg::Sil => "sil",
                Reg::Dil => "dil",
                Reg::Bpl => "bpl",
                Reg::Spl => "spl",
            },
        }
    }

    pub fn ptr(&self) -> Data {
        Data::RegPtr(*self)
    }

    pub fn ptr_offs(&self, offs: i64) -> Data {
        Data::RegPtrOffset(*self, offs)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Data {
    Reg(Reg),
    Const(i64),
    Label(&'static str),

    RegPtr(Reg),
    RegPtr2(Reg, Reg),
    RegPtrOffset(Reg, i64),

    RelLabel(String),
}

impl Data {
    pub fn stringify(&self, arch: TargetArch) -> String {
        match self {
            Data::Reg(reg) => format!("{}", reg.name(arch)),
            Data::Const(it) => format!("{it}"),
            Data::Label(it) => format!("[{it}]"),

            Data::RegPtr(reg) => format!("[{}]", reg.name(arch)),
            Data::RegPtr2(r1, r2) => format!("[{} + {}]", r1.name(arch), r2.name(arch)),

            Data::RegPtrOffset(reg, offs) => {
                if *offs == 0 {
                    format!("[{}]", reg.name(arch))
                } else {
                    format!("[{} + {offs}]", reg.name(arch))
                }
            }

            Data::RelLabel(it) => format!("[rip + {it}]"),
        }
    }
}

macro_rules! insns {
    {$(
        $name: ident$((
            $($param: ident: $ty: ty),*
        ))?
    ),* $(,)?} => {
        #[allow(unused)]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Insn {
            $($name$(($($ty),*))?),*
        }

        #[allow(unused)]
        pub trait AsmBuilder {
            fn insns(&mut self) -> &mut Vec<Insn>;

            $(paste::paste! {
                fn [<$name: lower>](&mut self $(, $($param: impl Into<$ty>),*)?) {
                    self.insns().push(Insn::$name $(($($param.into()),*))?);
                }
            })*
        }
    };
}

const PRE: &str = "    ";
const GNU_ASM: bool = true;

fn prefixes(a: &Data, b: &Data) -> (&'static str, &'static str) {
    match (a, b) {
        (Data::RegPtr(_) | Data::RegPtr2(_, _) | Data::RegPtrOffset(_, _), Data::Const(_)) => {
            ("byte ptr ", "")
        }

        (
            Data::RegPtr(_) | Data::RegPtr2(_, _) | Data::RegPtrOffset(_, _),
            Data::RegPtr(_) | Data::RegPtr2(_, _) | Data::RegPtrOffset(_, _),
        ) => ("byte ", ""),

        (Data::Reg(_), Data::RegPtr(_) | Data::RegPtr2(_, _) | Data::RegPtrOffset(_, _)) => {
            ("", "byte ptr ")
        }

        _ => ("", ""),
    }
}

fn prefix(a: &Data) -> &'static str {
    match a {
        Data::RegPtr(_) | Data::RegPtr2(_, _) | Data::RegPtrOffset(_, _) => "byte ptr ",
        _ => "",
    }
}

insns! {
    Mov(target: Data, src: Data),
    Lea(target: Data, src: Data),
    Add(target: Data, src: Data),
    Imul(target: Data, src: Data, mul: Data),
    Cmp(a: Data, b: Data),
    Xor(a: Data, b: Data),
    Inc(data: Data),
    Je(label: String),
    Jne(label: String),
    Jmp(label: String),
    Label(label: String),
    Section(name: &'static str),
    Resb(name: &'static str, size: i64),
    Global(label: &'static str),
    ScanByte,
    Syscall,
}

impl Insn {
    pub fn stringify(&self, arch: TargetArch) -> String {
        match self {
            Insn::Lea(a, b) => format!("{PRE}lea {}, {}", a.stringify(arch), b.stringify(arch)),

            Insn::Mov(a, b) => {
                let op = match (a, b) {
                    (
                        Data::Reg(_),
                        Data::RegPtr(_) | Data::RegPtr2(_, _) | Data::RegPtrOffset(_, _),
                    ) => "movzx",
                    _ => "mov",
                };

                let (p0, p1) = prefixes(a, &b);

                format!(
                    "{PRE}{op} {p0}{}, {p1}{}",
                    a.stringify(arch),
                    b.stringify(arch)
                )
            }

            Insn::Add(a, b) => {
                let mut b = b.clone();

                let op = match &mut b {
                    Data::Const(it) => {
                        if *it < 0 {
                            *it = -*it;
                            "sub"
                        } else {
                            "add"
                        }
                    }

                    _ => "add",
                };

                let (p0, p1) = prefixes(a, &b);

                format!(
                    "{PRE}{op} {p0}{}, {p1}{}",
                    a.stringify(arch),
                    b.stringify(arch)
                )
            }

            Insn::Imul(a, b, c) => format!(
                "{PRE}imul {}, {}, {}",
                a.stringify(arch),
                b.stringify(arch),
                c.stringify(arch)
            ),

            Insn::Inc(data) => format!("{PRE}inc {}{}", prefix(data), data.stringify(arch)),

            Insn::Cmp(a, b) => {
                let (p0, p1) = prefixes(a, b);

                format!(
                    "{PRE}cmp {p0}{}, {p1}{}",
                    a.stringify(arch),
                    b.stringify(arch)
                )
            }

            Insn::Xor(a, b) => {
                let (p0, p1) = prefixes(a, b);

                format!(
                    "{PRE}xor {p0}{}, {p1}{}",
                    a.stringify(arch),
                    b.stringify(arch)
                )
            }

            Insn::Je(it) => format!("{PRE}je {it}"),
            Insn::Jne(it) => format!("{PRE}jne {it}"),
            Insn::Jmp(it) => format!("{PRE}jmp {it}"),
            Insn::Label(it) => format!("{it}:"),

            Insn::Section(it) => {
                if GNU_ASM {
                    format!(".{it}")
                } else {
                    format!("section .{it}")
                }
            }

            Insn::Resb(name, size) => {
                if GNU_ASM {
                    format!("{PRE}.lcomm {name}, {size}")
                } else {
                    format!("{PRE}{name}: resb {size}")
                }
            }

            Insn::Global(it) => {
                if GNU_ASM {
                    format!(".global {it}")
                } else {
                    format!("global {it}")
                }
            }

            Insn::Syscall => format!("{PRE}syscall"),
            Insn::ScanByte => format!("{PRE}repne scasb"),
        }
    }
}

impl Into<Data> for i64 {
    fn into(self) -> Data {
        Data::Const(self)
    }
}

impl Into<Data> for i32 {
    fn into(self) -> Data {
        Data::Const(self as i64)
    }
}

impl Into<Data> for i16 {
    fn into(self) -> Data {
        Data::Const(self as i64)
    }
}

impl Into<Data> for i8 {
    fn into(self) -> Data {
        Data::Const(self as i64)
    }
}

impl Into<Data> for Reg {
    fn into(self) -> Data {
        Data::Reg(self)
    }
}
