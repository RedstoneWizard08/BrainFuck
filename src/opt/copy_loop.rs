use crate::opt::{OptAction, Optimizer, ValueAction};
use std::collections::BTreeMap;

impl<'a> Optimizer<'a> {
    pub(super) fn copy_loop(&mut self) {
        self.copy_loop_inner();
    }

    fn copy_loop_inner(&mut self) -> bool {
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

                OptAction::Value(ValueAction::AddValue(v)) => {
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
                    let mut opt = self.sub(it);

                    if opt.copy_loop_inner() {
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
}
