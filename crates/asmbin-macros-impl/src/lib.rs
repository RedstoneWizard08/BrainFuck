//! Implementation of procedural macros for x86-64 register declaration.
//!
//! This crate implements the parsing and code generation for the `registers!` macro,
//! which allows declarative specification of CPU registers with flexible configuration.
//!
//! # Examples
//!
//! The macro generates a `Reg` enum with methods for querying register properties:
//!
//! ```no_run
//! # // This is pseudo-code showing what the macro generates
//! #[repr(u8)]
//! pub enum Reg {
//!     Rax,
//!     Rbx,
//!     Rcx,
//!     // ... all defined registers
//! }
//!
//! impl Reg {
//!     pub const fn string_name(&self, prefix: Prefix) -> &'static str { }
//!     pub const fn bit_width(&self) -> usize { }
//!     pub const fn group(&self) -> RegGroup { }
//!     pub const fn id_bits(&self) -> u8 { }
//!     pub const fn prefix(&self) -> Prefix { }
//!     pub const fn is_ext(&self) -> bool { }
//! }
//! ```

use std::collections::HashSet;

use convert_case::{Case, Casing};
use itertools::Itertools;
use quote::quote;
use unsynn::{
    And, BraceGroupContaining, BracketGroupContaining, Colon, Comma, DotDot, Ident, Parse,
    Question, Semicolon, SemicolonDelimitedVec, Span, ToTokenIter, TokenStream, TrailingDelimiter,
    unsynn,
};

unsynn! {
    pub keyword Seg16 = "seg16";
    pub keyword St = "st";
    pub keyword R = "r";
    pub keyword Rex = "rex";
    pub keyword Gp8 = "gp8";
    pub keyword Gp = "gp";
    pub keyword Mmx = "mmx";

    pub operator Hash = "#";
    pub operator Equal = "=";

    pub struct ExtraSpec {
        _hash: Hash,
        _r: R,
    }

    pub enum RegFlag {
        Seg16 {
            _seg16_kw: Seg16,
            _eq: Equal,
            seg16: Ident,
        },

        RegSt {
            _st: St,
        },

        Mmx {
            _mmx: Mmx,
            _eq: Equal,
            mmx: usize,
        }
    }

    pub type RegFlags = BracketGroupContaining<SemicolonDelimitedVec<RegFlag, TrailingDelimiter::Optional>>;

    pub struct NameForRex {
        _rex: Rex,
        _q: Question,
        rex_name: Ident,
        _colon: Colon,
        default_name: Ident,
    }

    pub struct GpRest {
        gp16: Ident,
        _c1: Comma,
        gp32: Ident,
        _c2: Comma,
        gp64: Ident,
    }

    pub struct Seg16Def {
        _seg16: Seg16,
        _eq: Equal,
        name: Ident,
        _semi: Semicolon,
    }

    pub struct ComplexRegDecl {
        _gp8: Gp8,
        _gp8_eq: Equal,
        gp8: NameForRex,
        _gp8_semi: Semicolon,

        _gp_rest: Gp,
        _gp_rest_dotdot: DotDot,
        _gp_rest_eq: Equal,
        gp_rest: BracketGroupContaining<GpRest>,
        _gp_rest_semi: Semicolon,

        _st: St,
        _st_eq: Equal,
        st: bool,
        _st_semi: Semicolon,

        seg16: Option<Seg16Def>,
    }

    pub struct BasicFlags {
        _and: And,
        flags: RegFlags,
    }

    pub enum RegisterDeclValue {
        Basic {
            letter: Ident,
            flags: Option<BasicFlags>,
        },

        Complex(BraceGroupContaining<ComplexRegDecl>),

        Extra {
            _spec: ExtraSpec,
            flags: Option<BasicFlags>,
        }
    }

    pub struct RegisterDecl {
        index: usize,
        _eq: Equal,
        value: RegisterDeclValue,
        _semi: Semicolon,
    }

    pub type RegistersInput = Vec<RegisterDecl>;
}

// registers! {
//     0 = a & [seg16 = es; st];
//     1 = c & [seg16 = cs; st];
//     2 = d & [seg16 = ss; st];
//     3 = b & [seg16 = ds; st];

//     4 = {
//         gp8 = rex ? spl : ah;
//         gp.. = [sp, esp, rsp];
//         st = true;
//         seg16 = fs;
//     };

//     5 = {
//         gp8 = rex ? bpl : ch;
//         gp.. = [bp, ebp, rbp];
//         st = true;
//         seg16 = gs;
//     };

//     6 = {
//         gp8 = rex ? sil : dh;
//         gp.. = [si, esi, rsi];
//         st = true;
//     };

//     7 = {
//         gp8 = rex ? dil : bh;
//         gp.. = [di, edi, rdi];
//         st = true;
//     };

//     8 = #r & [seg16 = es];
//     9 = #r & [seg16 = cs];
//     10 = #r & [seg16 = ss];
//     11 = #r & [seg16 = ds];
//     12 = #r & [seg16 = fs];
//     13 = #r & [seg16 = gs];
//     14 = #r;
//     15 = #r;
// }

/// Represents instruction prefix types used in x86-64 encoding.
///
/// - `None`: No prefix
/// - `Rex`: REX prefix for extended registers (registers 8-15)
/// - `Evex`: EVEX prefix for SIMD instructions (512-bit registers)
///
/// # Examples
///
/// Checking register prefixes:
///
/// ```no_run
/// let prefix_type = Prefix::Rex;
/// match prefix_type {
///     Prefix::None => println!("No prefix needed"),
///     Prefix::Rex => println!("REX prefix for extended registers"),
///     Prefix::Evex => println!("EVEX prefix for SIMD"),
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Prefix {
    None,
    Rex,
    Evex,
}

/// Represents the classification group of a CPU register.
///
/// Groups include:
/// - `General8`: 8-bit general purpose registers
/// - `General16`: 16-bit general purpose registers
/// - `General32`: 32-bit general purpose registers
/// - `General64`: 64-bit general purpose registers
/// - `X87`: 80-bit x87 floating point stack registers
/// - `Mmx64`: 64-bit MMX registers
/// - `Xmm128`: 128-bit XMM SIMD registers
/// - `Ymm256`: 256-bit YMM SIMD registers
/// - `Zmm512`: 512-bit ZMM SIMD registers
/// - `Segment16`: 16-bit segment registers
/// - `Control32`: 32-bit control registers
/// - `Debug32`: 32-bit debug registers
///
/// # Examples
///
/// Matching on register groups:
///
/// ```no_run
/// match reg_info.group {
///     RegGroup::General64 => println!("64-bit general purpose register"),
///     RegGroup::Xmm128 => println!("128-bit SIMD register"),
///     _ => println!("Other register type"),
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RegGroup {
    General8,
    General16,
    General32,
    General64,
    X87,
    Mmx64,
    Xmm128,
    Ymm256,
    Zmm512,
    Segment16,
    Control32,
    Debug32,
}

/// Complete information about an x86-64 register.
///
/// Tracks the register's name, classification, bit width, encoding prefix,
/// alternative naming based on instruction prefix, and whether it's an extended register.
///
/// # Examples
///
/// Creating register information:
///
/// ```no_run
/// let reg = RegInfo {
///     name: "rax".to_string(),
///     group: RegGroup::General64,
///     bit_width: 64,
///     prefix: Prefix::None,
///     rex_name: None,
///     id_bits: 0,
///     is_ext: false,
/// };
///
/// println!("Register {} is {} bits", reg.name, reg.bit_width);
/// ```
#[derive(Debug, Clone, Eq, Ord)]
pub struct RegInfo {
    /// The register name (e.g., "rax", "xmm0")
    pub name: String,
    /// The functional group this register belongs to
    pub group: RegGroup,
    /// The bit width of the register
    pub bit_width: usize,
    /// The instruction prefix type needed for this register
    pub prefix: Prefix,
    /// Alternative name when using REX prefix (e.g., "spl" instead of "ah" for register 4, 8-bit)
    pub rex_name: Option<String>,
    /// The register ID in the encoding (0-15 for most registers)
    pub id_bits: u8,
    /// Whether this is an extended register (8-15)
    pub is_ext: bool,
}

impl PartialOrd for RegInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.name.partial_cmp(&other.name) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }

        match self.group.partial_cmp(&other.group) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }

        match self.bit_width.partial_cmp(&other.bit_width) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }

        match self.prefix.partial_cmp(&other.prefix) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }

        self.rex_name.partial_cmp(&other.rex_name)
    }
}

impl PartialEq for RegInfo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.group == other.group
            && self.bit_width == other.bit_width
            && self.prefix == other.prefix
            && self.rex_name == other.rex_name
    }
}

impl std::hash::Hash for RegInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.group.hash(state);
        self.bit_width.hash(state);
        self.prefix.hash(state);
        self.rex_name.hash(state);
    }
}

pub fn codegen_decl(decl: RegisterDecl, regs: &mut HashSet<RegInfo>) {
    let n = decl.index;

    let mut mmx = format!("mmx{n}");
    let xmm = format!("xmm{n}"); // rex = 8..15
    let ymm = format!("ymm{n}"); // rex = 8..15
    let zmm = format!("zmm{n}"); // evex

    let cr32 = format!("cr{n}"); // rex = 8..15
    let dr32 = format!("dr{n}"); // rex = 8..15

    let gp8_info;
    let gp16_info;
    let gp32_info;
    let gp64_info;
    let mmx_info;
    let mut st_info = None;
    let mut seg16_info = None;

    let xmm_info = RegInfo {
        name: xmm,
        group: RegGroup::Xmm128,
        bit_width: 128,
        prefix: if n > 7 { Prefix::Rex } else { Prefix::None },
        rex_name: None,
        id_bits: n as u8,
        is_ext: false,
    };

    let ymm_info = RegInfo {
        name: ymm,
        group: RegGroup::Ymm256,
        bit_width: 256,
        prefix: if n > 7 { Prefix::Rex } else { Prefix::None },
        rex_name: None,
        id_bits: n as u8,
        is_ext: false,
    };

    let zmm_info = RegInfo {
        name: zmm,
        group: RegGroup::Zmm512,
        bit_width: 512,
        prefix: Prefix::Evex,
        rex_name: None,
        id_bits: n as u8,
        is_ext: false,
    };

    let cr32_info = RegInfo {
        name: cr32,
        group: RegGroup::Control32,
        bit_width: 32,
        prefix: if n > 7 { Prefix::Rex } else { Prefix::None },
        rex_name: None,
        id_bits: n as u8,
        is_ext: false,
    };

    let dr32_info = RegInfo {
        name: dr32,
        group: RegGroup::Debug32,
        bit_width: 32,
        prefix: if n > 7 { Prefix::Rex } else { Prefix::None },
        rex_name: None,
        id_bits: n as u8,
        is_ext: false,
    };

    match decl.value {
        RegisterDeclValue::Basic {
            letter: x, flags, ..
        } => {
            let mut st = None;
            let mut seg16 = None;

            if let Some(BasicFlags {
                flags: BracketGroupContaining { content: flags },
                ..
            }) = flags
            {
                for flag in flags {
                    match flag.value {
                        RegFlag::Seg16 { seg16: id, .. } => {
                            seg16 = Some(format!("{id}"));
                        }

                        RegFlag::RegSt { .. } => {
                            st = Some(format!("st{n}"));
                        }

                        RegFlag::Mmx { mmx: n, .. } => {
                            mmx = format!("mmx{n}");
                        }
                    }
                }
            }

            gp8_info = RegInfo {
                name: format!("{x}l"),
                group: RegGroup::General8,
                bit_width: 8,
                prefix: Prefix::None,
                rex_name: None,
                id_bits: n as u8,
                is_ext: false,
            };

            gp16_info = RegInfo {
                name: format!("{x}x"),
                group: RegGroup::General16,
                bit_width: 16,
                prefix: Prefix::None,
                rex_name: None,
                id_bits: n as u8,
                is_ext: false,
            };

            gp32_info = RegInfo {
                name: format!("e{x}x"),
                group: RegGroup::General32,
                bit_width: 32,
                prefix: Prefix::None,
                rex_name: None,
                id_bits: n as u8,
                is_ext: false,
            };

            gp64_info = RegInfo {
                name: format!("r{x}x"),
                group: RegGroup::General64,
                bit_width: 64,
                prefix: Prefix::None,
                rex_name: None,
                id_bits: n as u8,
                is_ext: false,
            };

            if let Some(st) = st {
                st_info = Some(RegInfo {
                    name: st,
                    group: RegGroup::X87,
                    bit_width: 80,
                    prefix: Prefix::None,
                    rex_name: None,
                    id_bits: n as u8,
                    is_ext: false,
                });
            }

            if let Some(seg16) = seg16 {
                seg16_info = Some(RegInfo {
                    name: seg16,
                    group: RegGroup::Segment16,
                    bit_width: 16,
                    prefix: Prefix::None,
                    rex_name: None,
                    id_bits: n as u8,
                    is_ext: false,
                });
            }

            mmx_info = RegInfo {
                name: mmx,
                group: RegGroup::Mmx64,
                bit_width: 64,
                prefix: Prefix::None,
                rex_name: None,
                id_bits: n as u8,
                is_ext: false,
            };
        }

        RegisterDeclValue::Complex(BraceGroupContaining {
            content:
                ComplexRegDecl {
                    gp8:
                        NameForRex {
                            default_name: gp8,
                            rex_name: gp8_rex,
                            ..
                        },
                    gp_rest:
                        BracketGroupContaining {
                            content:
                                GpRest {
                                    gp16, gp32, gp64, ..
                                },
                        },
                    st: has_st,
                    seg16,
                    ..
                },
        }) => {
            gp8_info = RegInfo {
                name: format!("{gp8}"),
                group: RegGroup::General8,
                bit_width: 8,
                prefix: Prefix::None,
                rex_name: Some(format!("{gp8_rex}")),
                id_bits: n as u8,
                is_ext: false,
            };

            gp16_info = RegInfo {
                name: format!("{gp16}"),
                group: RegGroup::General16,
                bit_width: 16,
                prefix: Prefix::None,
                rex_name: None,
                id_bits: n as u8,
                is_ext: false,
            };

            gp32_info = RegInfo {
                name: format!("{gp32}"),
                group: RegGroup::General32,
                bit_width: 32,
                prefix: Prefix::None,
                rex_name: None,
                id_bits: n as u8,
                is_ext: false,
            };

            gp64_info = RegInfo {
                name: format!("{gp64}"),
                group: RegGroup::General64,
                bit_width: 64,
                prefix: Prefix::None,
                rex_name: None,
                id_bits: n as u8,
                is_ext: false,
            };

            if has_st {
                st_info = Some(RegInfo {
                    name: format!("st{n}"),
                    group: RegGroup::X87,
                    bit_width: 80,
                    prefix: Prefix::None,
                    rex_name: None,
                    id_bits: n as u8,
                    is_ext: false,
                });
            }

            if let Some(def) = seg16 {
                seg16_info = Some(RegInfo {
                    name: format!("{}", def.name),
                    group: RegGroup::Segment16,
                    bit_width: 16,
                    prefix: Prefix::None,
                    rex_name: None,
                    id_bits: n as u8,
                    is_ext: false,
                });
            }

            mmx_info = RegInfo {
                name: mmx,
                group: RegGroup::Mmx64,
                bit_width: 64,
                prefix: Prefix::None,
                rex_name: None,
                id_bits: n as u8,
                is_ext: false,
            };
        }

        RegisterDeclValue::Extra { flags, .. } => {
            let mut st = None;
            let mut seg16 = None;

            if let Some(BasicFlags {
                flags: BracketGroupContaining { content: flags },
                ..
            }) = flags
            {
                for flag in flags {
                    match flag.value {
                        RegFlag::Seg16 { seg16: id, .. } => {
                            seg16 = Some(format!("{id}"));
                        }

                        RegFlag::RegSt { .. } => {
                            st = Some(format!("st{n}"));
                        }

                        RegFlag::Mmx { mmx: n, .. } => {
                            mmx = format!("mmx{n}");
                        }
                    }
                }
            }

            gp8_info = RegInfo {
                name: format!("r{n}l"),
                group: RegGroup::General8,
                bit_width: 8,
                prefix: Prefix::None,
                rex_name: None,
                id_bits: n as u8,
                is_ext: true,
            };

            gp16_info = RegInfo {
                name: format!("r{n}w"),
                group: RegGroup::General16,
                bit_width: 16,
                prefix: Prefix::None,
                rex_name: None,
                id_bits: n as u8,
                is_ext: true,
            };

            gp32_info = RegInfo {
                name: format!("r{n}d"),
                group: RegGroup::General32,
                bit_width: 32,
                prefix: Prefix::None,
                rex_name: None,
                id_bits: n as u8,
                is_ext: true,
            };

            gp64_info = RegInfo {
                name: format!("r{n}"),
                group: RegGroup::General64,
                bit_width: 64,
                prefix: Prefix::None,
                rex_name: None,
                id_bits: n as u8,
                is_ext: true,
            };

            if let Some(st) = st {
                st_info = Some(RegInfo {
                    name: st,
                    group: RegGroup::X87,
                    bit_width: 80,
                    prefix: Prefix::None,
                    rex_name: None,
                    id_bits: n as u8,
                    is_ext: false,
                });
            }

            if let Some(seg16) = seg16 {
                seg16_info = Some(RegInfo {
                    name: seg16,
                    group: RegGroup::Segment16,
                    bit_width: 16,
                    prefix: Prefix::None,
                    rex_name: None,
                    id_bits: n as u8,
                    is_ext: false,
                });
            }

            mmx_info = RegInfo {
                name: mmx,
                group: RegGroup::Mmx64,
                bit_width: 64,
                prefix: Prefix::None,
                rex_name: None,
                id_bits: n as u8,
                is_ext: false,
            };
        }
    };

    regs.extend([
        gp8_info, gp16_info, gp32_info, gp64_info, mmx_info, xmm_info, ymm_info, zmm_info,
        cr32_info, dr32_info,
    ]);

    if let Some(st) = st_info {
        regs.insert(st);
    }

    if let Some(seg16) = seg16_info {
        regs.insert(seg16);
    }
}

/// Trait for converting types to Rust identifiers.
///
/// # Examples
///
/// Converting a string to an identifier:
///
/// ```no_run
/// let name = "my_register".to_string();
/// let ident = name.id();
/// // Can be used in quote!() macros for code generation
/// ```
pub trait ToIdent {
    /// Converts self to an identifier suitable for code generation.
    fn id(&self) -> Ident;
}

impl ToIdent for String {
    fn id(&self) -> Ident {
        Ident::new(self, Span::mixed_site())
    }
}

/// Generates Rust code for the Register enum and its methods.
///
/// Creates a complete Rust enum `Reg` with methods for querying register properties:
/// - `string_name`: Get the string name of a register
/// - `bit_width`: Get the bit width of a register
/// - `group`: Get the functional group of a register
/// - `id_bits`: Get the register ID used in encoding
/// - `prefix`: Get the instruction prefix required
/// - `is_ext`: Check if this is an extended register
///
/// # Arguments
///
/// * `regs` - The complete set of register information to codegen
pub fn codegen_regs(regs: HashSet<RegInfo>) -> TokenStream {
    let regs = regs
        .into_iter()
        .sorted_unstable_by_key(|it| it.name.clone())
        .collect_vec();

    let pre = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Prefix {
            None,
            Rex,
            Evex,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum RegGroup {
            General8,
            General16,
            General32,
            General64,
            X87,
            Mmx64,
            Xmm128,
            Ymm256,
            Zmm512,
            Segment16,
            Control32,
            Debug32,
        }
    };

    let names = regs.iter().map(|it| it.name.to_case(Case::Pascal).id());

    let str_conv = regs.iter().map(|it| {
        let name = it.name.to_case(Case::Pascal).id();
        let value = &it.name;

        if let Some(rex) = &it.rex_name {
            quote! {
                Self::#name => match _prefix {
                    Prefix::Rex => #rex,
                    _ => #value,
                }
            }
        } else {
            quote! {
                Self::#name => #value
            }
        }
    });

    let bit_width = regs.iter().map(|it| {
        let name = it.name.to_case(Case::Pascal).id();
        let value = it.bit_width;

        quote! { Self::#name => #value }
    });

    let group = regs.iter().map(|it| {
        let name = it.name.to_case(Case::Pascal).id();

        let group = match it.group {
            RegGroup::General8 => "General8".to_string().id(),
            RegGroup::General16 => "General16".to_string().id(),
            RegGroup::General32 => "General32".to_string().id(),
            RegGroup::General64 => "General64".to_string().id(),
            RegGroup::X87 => "X87".to_string().id(),
            RegGroup::Mmx64 => "Mmx64".to_string().id(),
            RegGroup::Xmm128 => "Xmm128".to_string().id(),
            RegGroup::Ymm256 => "Ymm256".to_string().id(),
            RegGroup::Zmm512 => "Zmm512".to_string().id(),
            RegGroup::Segment16 => "Segment16".to_string().id(),
            RegGroup::Control32 => "Control32".to_string().id(),
            RegGroup::Debug32 => "Debug32".to_string().id(),
        };

        quote! { Self::#name => RegGroup::#group }
    });

    let prefix = regs.iter().map(|it| {
        let name = it.name.to_case(Case::Pascal).id();

        let prefix = match it.prefix {
            Prefix::None => "None".to_string().id(),
            Prefix::Rex => "Rex".to_string().id(),
            Prefix::Evex => "Evex".to_string().id(),
        };

        quote! { Self::#name => Prefix::#prefix }
    });

    let id_bits = regs.iter().map(|it| {
        let name = it.name.to_case(Case::Pascal).id();
        let id = it.id_bits;

        quote! { Self::#name => #id }
    });

    let ext = regs.iter().map(|it| {
        let name = it.name.to_case(Case::Pascal).id();
        let ext = it.is_ext;

        quote! { Self::#name => #ext }
    });

    let enum_ = quote! {
        #[repr(u8)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Reg {
            #(#names),*
        }

        impl Reg {
            #[inline(always)]
            pub const fn string_name(&self, _prefix: Prefix) -> &'static str {
                match self {
                    #(#str_conv),*
                }
            }

            #[inline(always)]
            pub const fn bit_width(&self) -> usize {
                match self {
                    #(#bit_width),*
                }
            }

            #[inline(always)]
            pub const fn group(&self) -> RegGroup {
                match self {
                    #(#group),*
                }
            }

            #[inline(always)]
            pub const fn id_bits(&self) -> u8 {
                match self {
                    #(#id_bits),*
                }
            }

            #[inline(always)]
            pub const fn prefix(&self) -> Prefix {
                match self {
                    #(#prefix),*
                }
            }

            #[inline(always)]
            pub const fn is_ext(&self) -> bool {
                match self {
                    #(#ext),*
                }
            }
        }
    };

    quote! {
        #pre
        #enum_
    }
}

/// Parses register declarations and generates code.
///
/// This is the main macro implementation function. It takes the token stream
/// from the `registers!` macro invocation, parses the register declarations,
/// generates register info structures, and returns the code for the Reg enum.
///
/// # Arguments
///
/// * `input` - The token stream from the macro invocation
///
/// # Returns
///
/// A TokenStream containing the generated Rust code, or an error if parsing fails
pub fn registers(input: TokenStream) -> unsynn::Result<TokenStream> {
    let mut input = input.to_token_iter();
    let data = RegistersInput::parse(&mut input)?;
    let mut regs = HashSet::new();

    for decl in data {
        codegen_decl(decl, &mut regs);
    }

    Ok(codegen_regs(regs))
}
