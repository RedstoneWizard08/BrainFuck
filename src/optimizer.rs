use std::collections::BTreeMap;

use serde::Serialize;

use crate::Action;

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
    Loop(Vec<OptAction>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ChainType {
    Add(i64),
    Move(i64),
}

impl ChainType {
    #[inline(always)]
    pub fn action(&self) -> OptAction {
        match self {
            Self::Add(value) => OptAction::AddValue(*value),
            Self::Move(value) => OptAction::MovePtr(*value),
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
        }
    }
}

impl OptAction {
    #[inline(always)]
    pub fn as_chain(&self) -> Option<ChainType> {
        match self {
            Self::AddValue(v) => Some(ChainType::Add(*v)),
            Self::MovePtr(v) => Some(ChainType::Move(*v)),
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

pub struct Optimizer {
    actions: Vec<OptAction>,
}

impl Optimizer {
    pub fn new(actions: Vec<Action>) -> Self {
        Self {
            actions: convert(actions),
        }
    }

    fn chains(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        let mut chain: Option<ChainType> = None;

        for action in actions {
            if let Some(cur) = action.as_chain() {
                if let Some(chain) = &mut chain {
                    if !chain.merge(&cur) {
                        self.actions.push(chain.action());
                        *chain = cur;
                    }
                } else {
                    chain = Some(cur);
                }
            } else {
                if let Some(cur) = chain {
                    self.actions.push(cur.action());
                    chain = None;
                }

                if let OptAction::Loop(it) = action {
                    let mut opt = Optimizer { actions: it };

                    opt.chains();

                    self.actions.push(OptAction::Loop(opt.finish()));
                } else {
                    self.actions.push(action);
                }
            }
        }

        if let Some(cur) = chain {
            self.actions.push(cur.action());
        }
    }

    fn simplify(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        for action in actions {
            match action {
                OptAction::AddValue(0) | OptAction::MovePtr(0) | OptAction::Noop => (),

                OptAction::Loop(it) => {
                    let mut opt = Optimizer { actions: it };

                    opt.simplify();

                    self.actions.push(OptAction::Loop(opt.finish()));
                }

                other => self.actions.push(other),
            }
        }
    }

    fn loops(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        let mut was_loop = false;

        for action in actions {
            match action {
                OptAction::Loop(it) => {
                    if was_loop {
                        continue;
                    }

                    was_loop = true;

                    if it.len() == 0 {
                        continue;
                    } else if it.len() == 1 && matches!(it[0], OptAction::AddValue(_)) {
                        self.actions.push(OptAction::SetValue(0));
                    } else {
                        let mut opt = Optimizer { actions: it };

                        opt.loops();

                        self.actions.push(OptAction::Loop(opt.finish()));
                    }
                }

                other => {
                    was_loop = false;
                    self.actions.push(other);
                }
            }
        }
    }

    fn useless_ops(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        for action in actions {
            if action == OptAction::SetValue(0) {
                let mut cur = Vec::new();

                self.actions.reverse();

                std::mem::swap(&mut cur, &mut self.actions);

                let mut hit = false;

                for insn in cur {
                    if !insn.is_math() {
                        hit = true;
                    }

                    if hit {
                        self.actions.push(insn);
                    }
                }

                self.actions.reverse();
                self.actions.push(action);
            } else if let OptAction::Loop(it) = action {
                let mut opt = Optimizer { actions: it };

                opt.useless_ops();

                self.actions.push(OptAction::Loop(opt.finish()));
            } else {
                self.actions.push(action);
            }
        }
    }

    fn dead_code(&mut self, start: bool) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        let mut hit = !start;

        for action in actions {
            if !hit {
                if !matches!(action, OptAction::Loop(_)) {
                    hit = true;
                }
            }

            if hit {
                if let OptAction::CopyLoop(v) = action {
                    if v.len() != 0 {
                        self.actions.push(OptAction::CopyLoop(v));
                    } else {
                        self.actions.push(OptAction::SetValue(0));
                    }
                } else if let OptAction::Loop(v) = action {
                    let mut opt = Optimizer { actions: v };

                    opt.dead_code(false);

                    self.actions.push(OptAction::Loop(opt.finish()));
                } else {
                    self.actions.push(action);
                }
            }
        }
    }

    fn set_move(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        let mut prev: Option<OptAction> = None;

        for action in actions {
            if let OptAction::MovePtr(o) = action {
                if let Some(a) = prev {
                    if let OptAction::SetValue(v) = a {
                        self.actions.push(OptAction::SetAndMove(v, o));
                        prev = None;
                    } else if let OptAction::AddValue(v) = a {
                        self.actions.push(OptAction::AddAndMove(v, o));
                        prev = None;
                    } else if let OptAction::Loop(it) = a {
                        let mut opt = Optimizer { actions: it };

                        opt.set_move();

                        self.actions.push(OptAction::Loop(opt.finish()));
                        prev = Some(OptAction::MovePtr(o));
                    } else {
                        self.actions.push(a);
                        prev = Some(OptAction::MovePtr(o));
                    }
                } else {
                    prev = Some(OptAction::MovePtr(o));
                }
            } else if let Some(a) = prev {
                if let OptAction::Loop(it) = a {
                    let mut opt = Optimizer { actions: it };

                    opt.set_move();

                    self.actions.push(OptAction::Loop(opt.finish()));
                } else {
                    self.actions.push(a);
                }

                prev = Some(action);
            } else {
                prev = Some(action);
            }
        }

        if let Some(a) = prev {
            if let OptAction::Loop(it) = a {
                let mut opt = Optimizer { actions: it };

                opt.set_move();

                self.actions.push(OptAction::Loop(opt.finish()));
            } else {
                self.actions.push(a);
            }
        }
    }

    // THIS IS AN UNSAFE OPTIMIZATION
    // IT REQUIRES UNSAFE MODE TO BE ON
    // IT RELIES ON DIRECT POINTER ACCESS
    fn copy_loop(&mut self) -> bool {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        let mut no = false;
        let mut pos = 0;
        let mut changes = BTreeMap::new();

        for insn in &actions {
            match insn {
                OptAction::AddAndMove(v, o) => {
                    *changes.entry(pos).or_insert(0) += *v;
                    pos += *o;
                }

                OptAction::AddValue(v) => {
                    *changes.entry(pos).or_insert(0) += *v;
                }

                OptAction::MovePtr(o) => {
                    pos += *o;
                }

                _ => {
                    no = true;
                    break;
                }
            }
        }

        let c = changes.remove(&0).unwrap_or(0);

        if no || pos != 0 || c != -1 {
            for insn in actions {
                if let OptAction::Loop(it) = insn {
                    let mut opt = Optimizer { actions: it };

                    if opt.copy_loop() {
                        let mut insns = opt.finish();

                        if insns.len() != 1 {
                            panic!("Invalid instruction list for CopyLoop optimization: {insns:?}");
                        }

                        let insn = insns.remove(0);

                        self.actions.push(insn);
                    } else {
                        self.actions.push(OptAction::Loop(opt.finish()));
                    }
                } else {
                    self.actions.push(insn);
                }
            }

            return false;
        }

        self.actions.push(OptAction::CopyLoop(changes));

        true
    }

    pub fn run_all(mut self, passes: u8, unsafe_mode: bool) -> Self {
        for _ in 0..passes {
            self.chains();
            self.loops();
            self.useless_ops();
            self.dead_code(true);
            self.set_move();

            if unsafe_mode {
                self.copy_loop();
            }

            self.simplify();
        }

        self
    }

    pub fn finish(self) -> Vec<OptAction> {
        self.actions
    }
}
