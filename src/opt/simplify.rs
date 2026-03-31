use std::collections::HashSet;

use crate::opt::{OptAction, Optimizer, ValueAction};

impl<'a> Optimizer<'a> {
    pub(super) fn simplify(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        for action in actions {
            match action {
                OptAction::Value(ValueAction::AddValue(0))
                | OptAction::MovePtr(0)
                | OptAction::Noop => (),

                OptAction::OffsetValue(it, 0) => self.actions.push(OptAction::Value(it)),

                OptAction::Value(ValueAction::BulkPrint(0)) => {
                    self.actions.push(OptAction::Value(ValueAction::Output))
                }

                other => self.actions.push(other),
            }
        }
    }

    pub(super) fn simplify_start(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        let mut off = false;
        let mut written = HashSet::new();
        let mut written_0 = false;

        for action in actions {
            if !off {
                match action {
                    OptAction::OffsetValue(ValueAction::AddValue(add), off) => {
                        if !written.contains(&off) {
                            self.actions
                                .push(OptAction::OffsetValue(ValueAction::SetValue(add), off));
                            written.insert(off);
                        } else {
                            self.actions
                                .push(OptAction::OffsetValue(ValueAction::AddValue(add), off));
                        }
                    }

                    OptAction::Value(ValueAction::AddValue(add)) => {
                        if !written_0 {
                            self.actions
                                .push(OptAction::Value(ValueAction::SetValue(add)));
                            written_0 = true;
                        } else {
                            self.actions
                                .push(OptAction::Value(ValueAction::AddValue(add)));
                        }
                    }

                    _ => {
                        self.actions.push(action);
                        off = true;
                    }
                }
            } else {
                self.actions.push(action);
            }
        }
    }
}
