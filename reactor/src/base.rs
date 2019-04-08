pub enum EventResult<O> {
    Handled,
    Transition(O),
    NotHandled,
}

pub trait Actor<E: Clone> {
    fn enter(&self) {}
    fn handle<O>(&self, _event: E) -> EventResult<O> { EventResult::NotHandled }
    fn exit(&self) {}
}

pub trait State<E: Clone>: Actor<E> {
    const INITIAL_STATE: Self;

    fn super_enter(&self);
    fn super_handle<O>(&self, event: E) -> EventResult<O>;
    fn super_exit(&self);
}