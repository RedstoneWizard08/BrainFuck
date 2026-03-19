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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Data {
    Reg(Reg),
    Const(i64),
    Label(&'static str),

    RegPtr(Reg),
    RegPtr2(Reg, Reg),
    RegPtrOffset(Reg, i64),
}

impl Data {
    pub fn stringify(&self, arch: TargetArch) -> String {
        match self {
            Data::Reg(reg) => format!("{}", reg.name(arch)),
            Data::Const(it) => format!("{it}"),
            Data::Label(it) => format!("[{it}]"),

            Data::RegPtr(reg) => format!("[{}]", reg.name(arch)),
            Data::RegPtr2(r1, r2) => format!("[{} + {}]", r1.name(arch), r2.name(arch)),
            Data::RegPtrOffset(reg, offs) => format!("[{} + {offs}]", reg.name(arch)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Insn {
    Mov(Data, Data),
    Lea(Data, Data),
    Add(Data, Data),
    Imul(Data, Data, Data),
    Cmp(Data, Data),
    Je(String),
    Jmp(String),
    Label(String),
    Section(&'static str),
    Resb(&'static str, i64),
    Global(&'static str),
    Syscall,
}

pub trait AsmBuilder {
    fn insns(&mut self) -> &mut Vec<Insn>;

    fn mov(&mut self, target: impl Into<Data>, src: impl Into<Data>) {
        self.insns().push(Insn::Mov(target.into(), src.into()));
    }

    fn lea(&mut self, target: impl Into<Data>, src: impl Into<Data>) {
        self.insns().push(Insn::Lea(target.into(), src.into()));
    }

    fn add(&mut self, target: impl Into<Data>, src: impl Into<Data>) {
        self.insns().push(Insn::Add(target.into(), src.into()));
    }

    fn imul(&mut self, target: impl Into<Data>, src: impl Into<Data>, mul: impl Into<Data>) {
        self.insns()
            .push(Insn::Imul(target.into(), src.into(), mul.into()));
    }

    fn cmp(&mut self, a: impl Into<Data>, b: impl Into<Data>) {
        self.insns().push(Insn::Cmp(a.into(), b.into()));
    }

    fn je(&mut self, label: impl AsRef<str>) {
        self.insns().push(Insn::Je(label.as_ref().into()));
    }

    fn jmp(&mut self, label: impl AsRef<str>) {
        self.insns().push(Insn::Jmp(label.as_ref().into()));
    }

    fn label(&mut self, label: impl AsRef<str>) {
        self.insns().push(Insn::Label(label.as_ref().into()));
    }

    fn sect(&mut self, name: &'static str) {
        self.insns().push(Insn::Section(name));
    }

    fn resb(&mut self, name: &'static str, size: i64) {
        self.insns().push(Insn::Resb(name, size));
    }

    fn global(&mut self, label: &'static str) {
        self.insns().push(Insn::Global(label));
    }

    fn syscall(&mut self) {
        self.insns().push(Insn::Syscall);
    }
}

const PRE: &str = "    ";

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

const GNU_ASM: bool = true;

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
                let mut b = *b;

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

            Insn::Cmp(a, b) => format!("{PRE}cmp {}, {}", a.stringify(arch), b.stringify(arch)),
            Insn::Je(it) => format!("{PRE}je {it}"),
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
                    format!("{PRE}.lcomm {name} {size}")
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
        }
    }
}

impl Into<Data> for i64 {
    fn into(self) -> Data {
        Data::Const(self)
    }
}

impl Into<Data> for Reg {
    fn into(self) -> Data {
        Data::Reg(self)
    }
}
