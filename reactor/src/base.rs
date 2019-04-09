pub enum EventResult<O> {
    Handled,
    Transition(O),
    NotHandled,
}

pub trait Actor<E: Clone> {
    type State;

    fn enter(&self) {}
    fn handle(&self, _event: E) -> EventResult<<Self as Actor<E>>::State> { EventResult::NotHandled }
    fn exit(&self) {}
}

pub trait State<E: Clone>: Actor<E> {
    type State: State<E>;
    const INITIAL_STATE: Self;

    fn super_enter(&self);
    fn super_handle(&self, event: E) -> EventResult<<Self as State<E>>::State>;
    fn super_exit(&self);
}