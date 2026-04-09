use crate::opt::{
    action::{OptAction, ValueAction},
    v1::Optimizer,
};

impl OptAction {
    fn does_output(&self) -> bool {
        match self {
            Self::Loop(it) => it.iter().any(|it| it.does_output()),
            Self::Value(ValueAction::BulkPrint(_)) | Self::Value(ValueAction::Output) => true,
            Self::OffsetValue(ValueAction::BulkPrint(_), _)
            | Self::OffsetValue(ValueAction::Output, _) => true,
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
            if any {
                self.actions.push(action);
            } else if action.does_output() {
                any = true;
                self.actions.push(action);
            }
        }

        self.actions.reverse();
    }
}
