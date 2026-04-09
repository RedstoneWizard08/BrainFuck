use crate::{backend::Optimization, opt::v1::{ChainType, Optimizer}};

impl<'a> Optimizer<'a> {
    pub(super) fn chains(&mut self) {
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

                self.actions.push(action);
            }
        }

        if let Some(cur) = chain {
            self.actions.push(cur.action());
        }

        self.optimize_loops(Optimization::Chain);
    }
}
