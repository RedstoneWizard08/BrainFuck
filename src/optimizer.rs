use serde::Serialize;

use crate::Action;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum OptAction {
    Noop,
    Right,
    Left,
    Inc,
    Dec,
    Output,
    Input,
    AddValue(usize),
    SubValue(usize),
    SetValue(usize),
    MoveRight(usize),
    MoveLeft(usize),
    /// (length_to_zero)
    /// Make sure to move right the same amount!!
    ZeroRight(usize),
    Loop(Vec<OptAction>),
}

impl OptAction {
    pub fn can_chain(&self) -> bool {
        match self {
            Self::Right | Self::Left | Self::Inc | Self::Dec => true,
            _ => false,
        }
    }
}

pub fn convert(actions: Vec<Action>) -> Vec<OptAction> {
    actions
        .into_iter()
        .map(|it| match it {
            Action::Right => OptAction::Right,
            Action::Left => OptAction::Left,
            Action::Inc => OptAction::Inc,
            Action::Dec => OptAction::Dec,
            Action::Output => OptAction::Output,
            Action::Input => OptAction::Input,
            Action::Loop(actions) => OptAction::Loop(convert(actions)),
        })
        .collect()
}

pub struct Optimizer {
    actions: Vec<OptAction>,

    counter: usize,
    last_kind: OptAction,
}

impl Optimizer {
    pub fn new(actions: Vec<Action>) -> Self {
        Self {
            actions: convert(actions),
            counter: 0,
            last_kind: OptAction::Noop,
        }
    }

    pub fn reset_chain(&mut self) {
        self.counter = 0;
        self.last_kind = OptAction::Noop;
    }

    pub fn try_chain(&mut self, insn: OptAction) {
        if self.last_kind != insn {
            let mut kind = OptAction::Noop;

            std::mem::swap(&mut kind, &mut self.last_kind);

            match kind {
                OptAction::Right => self.actions.push(OptAction::MoveRight(self.counter)),
                OptAction::Left => self.actions.push(OptAction::MoveLeft(self.counter)),
                OptAction::Inc => self.actions.push(OptAction::AddValue(self.counter)),
                OptAction::Dec => self.actions.push(OptAction::SubValue(self.counter)),

                it => self.actions.push(it),
            };

            self.reset_chain();
        }

        if !insn.can_chain() {
            if let OptAction::Loop(it) = insn {
                let mut opt = Optimizer {
                    actions: it,
                    counter: 0,
                    last_kind: OptAction::Noop,
                };

                opt.optimize_straight_runs();

                self.actions.push(OptAction::Loop(opt.finish()));
            } else {
                self.actions.push(insn);
            }
        } else {
            self.last_kind = insn;
            self.counter += 1;
        }
    }

    pub fn optimize_straight_runs(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        for action in actions {
            self.try_chain(action);
        }

        self.try_chain(OptAction::Noop); // Catch any stragglers
    }

    pub fn simplify(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        for action in actions {
            match action {
                OptAction::AddValue(0)
                | OptAction::SubValue(0)
                | OptAction::MoveLeft(0)
                | OptAction::MoveRight(0)
                | OptAction::Noop => (),

                OptAction::AddValue(1) => self.actions.push(OptAction::Inc),
                OptAction::SubValue(1) => self.actions.push(OptAction::Dec),
                OptAction::MoveLeft(1) => self.actions.push(OptAction::Left),
                OptAction::MoveRight(1) => self.actions.push(OptAction::Right),

                OptAction::Loop(it) => {
                    let mut opt = Optimizer {
                        actions: it,
                        counter: 0,
                        last_kind: OptAction::Noop,
                    };

                    opt.simplify();

                    self.actions.push(OptAction::Loop(opt.finish()));
                }

                other => self.actions.push(other),
            }
        }
    }

    pub fn loops(&mut self) {
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
                    } else if it.len() == 1 && (it == [OptAction::Dec] || it == [OptAction::Inc]) {
                        self.actions.push(OptAction::SetValue(0));
                    } else {
                        let mut opt = Optimizer {
                            actions: it,
                            counter: 0,
                            last_kind: OptAction::Noop,
                        };

                        opt.loops();

                        self.actions.push(OptAction::Loop(opt.finish()));
                    }
                }

                other => {
                    was_loop = false;
                    self.actions.push(other);
                },
            }
        }
    }

    pub fn zeroizer(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        let mut actions = actions.into_iter().peekable();

        while let Some(action) = actions.next() {
            match action {
                OptAction::Right => {
                    if actions.peek() == Some(&OptAction::SetValue(0)) {
                        let mut counter = 1;
                        actions.next();

                        loop {
                            if actions.peek() != Some(&OptAction::Right) {
                                break;
                            }

                            actions.next();

                            if actions.peek() != Some(&OptAction::SetValue(0)) {
                                break;
                            }

                            counter += 1;
                            actions.next();
                        }

                        self.actions.push(OptAction::ZeroRight(counter));
                    } else {
                        self.actions.push(OptAction::Right);
                    }
                }

                OptAction::Loop(it) => {
                    let mut opt = Optimizer {
                        actions: it,
                        counter: 0,
                        last_kind: OptAction::Noop,
                    };

                    opt.zeroizer();

                    self.actions.push(OptAction::Loop(opt.finish()));
                }

                other => self.actions.push(other),
            }
        }
    }

    pub fn run_all(mut self) -> Self {
        self.optimize_straight_runs();
        self.simplify();
        self.loops();
        self.simplify();
        // self.zeroizer(); // THIS DOES NOT WORK RIGHT NOW!!
        self
    }

    pub fn finish(self) -> Vec<OptAction> {
        self.actions
    }
}
