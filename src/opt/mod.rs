mod chain;
mod copy_loop;
mod dead_code;
mod loops;
mod set_move;
mod simd;
mod simplify;
mod useless_ops;

use crate::{
    Action,
    compiler::{CompilerOptions, Optimization},
};
use anyhow::Result;
use ron::{Options, Serializer, ser::PrettyConfig};
use serde::Serialize;
use std::{collections::BTreeMap, fs};

#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum OptAction {
    Noop,
    Output,
    Input,
    AddValue(i64),
    SetValue(i64),
    MovePtr(i64),
    SetAndMove(i64, i64),
    AddAndMove(i64, i64),
    CopyLoop(BTreeMap<i64, i64>),
    SimdAddMove(Vec<i8>, i64),
    Loop(Vec<OptAction>),
    BulkPrint(i64),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ChainType {
    Add(i64),
    Move(i64),
    Print(i64),
}

impl ChainType {
    #[inline(always)]
    pub fn action(&self) -> OptAction {
        match self {
            Self::Add(value) => OptAction::AddValue(*value),
            Self::Move(value) => OptAction::MovePtr(*value),
            Self::Print(value) => OptAction::BulkPrint(*value),
        }
    }

    #[inline(always)]
    pub fn merge(&mut self, other: &ChainType) -> bool {
        match self {
            ChainType::Add(me) => {
                if let ChainType::Add(it) = other {
                    *me = *me + *it;
                    true
                } else {
                    false
                }
            }

            ChainType::Move(me) => {
                if let ChainType::Move(it) = other {
                    *me = *me + *it;
                    true
                } else {
                    false
                }
            }

            ChainType::Print(me) => {
                if let ChainType::Print(it) = other {
                    *me = *me + *it;
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl OptAction {
    #[inline(always)]
    pub fn as_chain(&self) -> Option<ChainType> {
        match self {
            Self::AddValue(v) => Some(ChainType::Add(*v)),
            Self::MovePtr(v) => Some(ChainType::Move(*v)),
            Self::Output => Some(ChainType::Print(1)),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn is_math(&self) -> bool {
        match self {
            Self::AddValue(_) => true,
            _ => false,
        }
    }
}

pub fn convert(actions: Vec<Action>) -> Vec<OptAction> {
    actions
        .into_iter()
        .map(|it| match it {
            Action::Right => OptAction::MovePtr(1),
            Action::Left => OptAction::MovePtr(-1),
            Action::Inc => OptAction::AddValue(1),
            Action::Dec => OptAction::AddValue(-1),
            Action::Output => OptAction::Output,
            Action::Input => OptAction::Input,
            Action::Loop(actions) => OptAction::Loop(convert(actions)),
        })
        .collect()
}

pub struct Optimizer<'a> {
    actions: Vec<OptAction>,
    opts: &'a CompilerOptions,
}

impl<'a> Optimizer<'a> {
    pub fn new(opts: &'a CompilerOptions, actions: Vec<Action>) -> Self {
        Self {
            actions: convert(actions),
            opts,
        }
    }

    fn sub(&self, actions: Vec<OptAction>) -> Optimizer<'a> {
        Optimizer {
            actions,
            opts: self.opts,
        }
    }

    fn run(&mut self, opt: Optimization) {
        if self.opts.no_optimize.contains(&opt) {
            return;
        }

        match opt {
            Optimization::Chain => self.chains(),
            Optimization::Loop => self.loops(),
            Optimization::UselessOps => self.useless_ops(),
            Optimization::DeadCode => self.dead_code(),
            Optimization::SetMove => self.set_move(),
            Optimization::Simplify => self.simplify(),
            Optimization::CopyLoop => self.copy_loop(),

            #[cfg(feature = "llvm")]
            Optimization::Simd => {
                if opts.backend == crate::compiler::Backend::LLVM {
                    log::warn!("Vectorization is currently not supported on the LLVM backend!");
                } else {
                    self.simd_add();
                }
            }

            #[cfg(not(feature = "llvm"))]
            Optimization::Simd => self.simd_add(),
        }
    }

    pub fn run_all(mut self) -> Self {
        for _ in 0..self.opts.opt_level {
            self.run(Optimization::Chain);
            self.run(Optimization::Loop);
            self.run(Optimization::UselessOps);
            self.run(Optimization::DeadCode);
            self.run(Optimization::SetMove);

            if self.opts.unsafe_mode {
                self.run(Optimization::CopyLoop);
                self.run(Optimization::Simd);
            }

            self.run(Optimization::Simplify);
        }

        self
    }

    pub fn finish(self) -> Vec<OptAction> {
        self.actions
    }

    pub fn finish_with_write(self) -> Result<Vec<OptAction>> {
        let actions = self.actions;

        if let Some(path) = &self.opts.output_tokens {
            log::debug!("Serializing tokens...");

            let mut out = String::new();

            let mut ser = Serializer::with_options(
                &mut out,
                Some(PrettyConfig::new()),
                &Options::default().without_recursion_limit(),
            )?;

            let ser = serde_stacker::Serializer::new(&mut ser);

            actions.serialize(ser)?;

            fs::write(path, out)?;

            log::debug!("Token dump written!");
        }

        Ok(actions)
    }
}
