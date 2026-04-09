mod chain;
mod combos;
mod copy_loop;
mod dead_code;
mod loop_unroll;
mod loops;
mod offsets;
mod scan;
mod set_move;
mod simplify;
mod useless_end;
mod useless_ops;

use crate::{
    Action,
    backend::{CompilerOptions, Optimization},
    opt::action::{OptAction, ValueAction},
};
use anyhow::Result;
use log::{debug, warn};
use ron::{Options, Serializer, ser::PrettyConfig};
use serde::Serialize;
use std::{fs, time::Instant};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ChainType {
    Add(i64),
    Set(i64),
    Move(i64),
    Print(i64),
}

impl ChainType {
    #[inline(always)]
    pub fn action(&self) -> OptAction {
        match self {
            Self::Add(value) => OptAction::Value(ValueAction::AddValue(*value)),
            Self::Set(value) => OptAction::Value(ValueAction::SetValue(*value)),
            Self::Move(value) => OptAction::MovePtr(*value),

            Self::Print(value) => {
                if *value == 1 {
                    OptAction::Value(ValueAction::Output)
                } else {
                    OptAction::Value(ValueAction::BulkPrint(*value))
                }
            }
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

            ChainType::Set(me) => {
                if let ChainType::Set(it) = other {
                    *me = *it;
                    true
                } else if let ChainType::Add(it) = other {
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
            Self::Value(ValueAction::AddValue(v)) => Some(ChainType::Add(*v)),
            Self::MovePtr(v) => Some(ChainType::Move(*v)),
            Self::Value(ValueAction::Output) => Some(ChainType::Print(1)),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn is_math(&self) -> bool {
        match self {
            Self::Value(ValueAction::AddValue(_)) => true,
            _ => false,
        }
    }

    #[inline(always)]
    pub fn count(&self) -> usize {
        match self {
            Self::Loop(it) => it.iter().map(|it| it.count()).sum::<usize>() + 1,
            _ => 1,
        }
    }
}

pub fn convert(actions: Vec<Action>) -> Vec<OptAction> {
    actions
        .into_iter()
        .map(|it| match it {
            Action::Right => OptAction::MovePtr(1),
            Action::Left => OptAction::MovePtr(-1),
            Action::Inc => OptAction::Value(ValueAction::AddValue(1)),
            Action::Dec => OptAction::Value(ValueAction::AddValue(-1)),
            Action::Output => OptAction::Value(ValueAction::Output),
            Action::Input => OptAction::Value(ValueAction::Input),
            Action::Loop(actions) => OptAction::Loop(convert(actions)),
        })
        .collect()
}

pub struct Optimizer<'a> {
    actions: Vec<OptAction>,
    opts: &'a CompilerOptions,
    depth: usize,
}

impl<'a> Optimizer<'a> {
    pub fn new(opts: &'a CompilerOptions, actions: Vec<Action>) -> Self {
        Self {
            actions: convert(actions),
            opts,
            depth: 0,
        }
    }

    fn sub(&self, actions: Vec<OptAction>) -> Optimizer<'a> {
        Optimizer {
            actions,
            opts: self.opts,
            depth: self.depth + 1,
        }
    }

    fn run(&mut self, opt: Optimization) {
        if self.opts.no_optimize.contains(&opt) {
            return;
        }

        let pre = "    ".repeat(self.depth);
        let pre_count = self.actions.len();

        debug!("{pre}-----------------------------------------------");
        debug!("{pre}Starting optimization pass: {opt}");
        debug!("{pre}Instruction count (depth=1): {}", self.actions.len());
        debug!("{pre}-----------------------------------------------");

        let now = Instant::now();

        match opt {
            Optimization::Chain => self.chains(),
            Optimization::Loop => self.loops(),
            Optimization::UselessOps => self.useless_ops(),
            Optimization::DeadCode => self.dead_code(),
            Optimization::SetMove => self.set_move(),
            Optimization::Simplify => self.simplify(),
            Optimization::SimplifyStart => self.simplify_start(),
            Optimization::CopyLoop => self.copy_loop(),
            Optimization::UselessEnd => self.useless_end(),
            Optimization::Offsets => self.offsets(),
            Optimization::Scanners => self.scanners(),
            Optimization::SetAdd => self.set_add(),
            Optimization::LoopUnroll => self.loop_unroll(),

            Optimization::SortOffsetOps => {
                warn!("sort_offset_ops optimization is unsupported with the V1 optimizer")
            }
        };

        let time = now.elapsed().as_micros();
        let count = self.actions.len();
        let change = count as isize - pre_count as isize;

        debug!("{pre}-----------------------------------------------");
        debug!("{pre}Completed optimization pass: {opt}");
        debug!("{pre}Took {time} μs");
        debug!("{pre}Instruction count (depth=1): {count}");
        debug!("{pre}Count change: {pre_count} -> {count}: {change}");
        debug!("{pre}-----------------------------------------------");
    }

    fn optimize_loops(&mut self, id: Optimization) {
        let mut actions = Vec::new();

        std::mem::swap(&mut self.actions, &mut actions);

        for action in actions {
            if let OptAction::Loop(it) = action {
                let mut opt = self.sub(it);

                opt.run(id);

                self.actions.push(OptAction::Loop(opt.finish()));
            } else {
                self.actions.push(action);
            }
        }
    }

    fn run_pass(&mut self) {
        self.run(Optimization::Chain);
        self.run(Optimization::Loop);
        self.run(Optimization::UselessOps);

        if self.depth == 0 {
            self.run(Optimization::DeadCode);
        }

        self.run(Optimization::CopyLoop);
        self.run(Optimization::Offsets);
        self.run(Optimization::SetMove);
        self.run(Optimization::Scanners);
        self.run(Optimization::Simplify);
        self.run(Optimization::SimplifyStart);

        if self.depth == 0 {
            self.run(Optimization::UselessEnd);
        }

        self.run(Optimization::SetAdd);
        self.run(Optimization::LoopUnroll);
    }

    pub fn run_all(mut self) -> Self {
        for _ in 0..self.opts.opt_level {
            self.run_pass();
        }

        let insns = self.actions.iter().map(|it| it.count()).sum::<usize>();

        debug!("Instruction count: {insns}");

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
