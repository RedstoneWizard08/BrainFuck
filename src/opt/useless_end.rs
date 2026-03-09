use crate::opt::{OptAction, Optimizer, ValueAction};

impl OptAction {
    fn does_output(&self) -> bool {
        match self {
            Self::Loop(it) => it.iter().any(|it| it.does_output()),
            Self::Value(ValueAction::BulkPrint(_)) | Self::Value(ValueAction::Output) => true,
            _ => false,
        }
    }
}

impl<'a> Optimizer<'a> {
    pub(super) fn useless_end(&mut self) {
        let mut actions = Vec::new();

        std::mem::swap(&mut actions, &mut self.actions);

        actions.reverse();

        let mut any = false;

        for action in actions {
            if action.does_output() {
                any = true;
            }

            if any {
                self.actions.push(action);
            }
        }

        self.actions.reverse();
    }
}
