use crate::middleware::middleware::TreeMiddleware;
use crate::state::account::AccountState;
use crate::state::event::{EventInCommand};
use crate::state::order::{OrderState};


pub struct State<M: TreeMiddleware> {
    acc: AccountState<M>,
    order: OrderState<M>,
}


impl<M> State<M>
    where
        M: TreeMiddleware
{
    pub fn on_event(&mut self, e: EventInCommand) {}
}