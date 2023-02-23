use std::sync::Arc;
use waitgroup::Worker;

pub struct EventInCommand {
    pub event: StateEvent,
    pub worker: Arc<Worker>,
}

pub struct EventOutCommand {}

pub enum StateEvent {
    L2Transfer(L2TransferEvent),
    L2Withdraw(L2WithdrawEvent),
    Order(OrderSubmit),
}

pub struct L2TransferEvent {}

pub struct L2WithdrawEvent {}

pub struct OrderSubmit {}