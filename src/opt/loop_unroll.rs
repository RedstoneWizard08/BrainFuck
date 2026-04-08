use crate::{
    backend::Optimization,
    opt::{OptAction, Optimizer, ValueAction},
};

impl<'a> Optimizer<'a> {
    pub(super) fn loop_unroll(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut self.actions, &mut actions);

        let mut cur_val = 0;

        for action in actions {
            let new_val = match &action {
                OptAction::AddAndMove(_, mov) => {
                    if *mov > 0 {
                        0
                    } else {
                        cur_val
                    }
                }

                OptAction::SetAndMove(_, mov) => {
                    if *mov > 0 {
                        0
                    } else {
                        cur_val
                    }
                }

                OptAction::MovePtr(mov) => {
                    if *mov > 0 {
                        0
                    } else {
                        cur_val
                    }
                }

                OptAction::Value(ValueAction::AddValue(add)) => cur_val + *add,
                OptAction::Value(ValueAction::SetValue(set)) => *set,

                OptAction::Loop(_) => 0,
                OptAction::Scan(_) => 0,
                OptAction::CopyLoop(_) => 0,

                _ => cur_val,
            };

            if let OptAction::Loop(it) = action {
                let mut pos = 0;
                let mut change = 0;

                for item in &it {
                    match item {
                        OptAction::AddAndMove(add, mov) => {
                            if pos == 0 {
                                change += *add;
                            }

                            pos += *mov;
                        }

                        OptAction::SetAndMove(set, mov) => {
                            if pos == 0 {
                                change = *set;
                            }

                            pos += *mov;
                        }

                        OptAction::MovePtr(mov) => {
                            pos += *mov;
                        }

                        OptAction::Value(ValueAction::AddValue(add)) => {
                            if pos == 0 {
                                change += *add;
                            }
                        }

                        OptAction::Value(ValueAction::SetValue(set)) => {
                            if pos == 0 {
                                change = *set;
                            }
                        }

                        OptAction::OffsetValue(ValueAction::AddValue(add), 0) => {
                            if pos == 0 {
                                change += *add;
                            }
                        }

                        OptAction::OffsetValue(ValueAction::SetValue(set), 0) => {
                            if pos == 0 {
                                change = *set;
                            }
                        }

                        _ => {}
                    };
                }

                let abs = if cur_val < 0 {
                    256 - (cur_val.abs() % 256)
                } else {
                    cur_val % 256
                };

                if change == 0 || cur_val == 0 {
                    self.actions.push(OptAction::Loop(it));
                } else {
                    let times = (abs as i64 / change).abs();

                    for _ in 0..times {
                        self.actions.extend(it.clone());
                    }
                }
            } else {
                self.actions.push(action);
            }

            cur_val = new_val;
        }

        self.optimize_loops(Optimization::LoopUnroll);
    }
}
