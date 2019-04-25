pub enum EventResult<O: core::fmt::Debug> {
    Handled,
    Transition(O),
    NotHandled,
}

pub trait Actor<E: Clone> where <Self as Actor<E>>::State: core::fmt::Debug {
    type State;

    fn enter(&self) {}
    fn handle(&self, _event: E) -> EventResult<<Self as Actor<E>>::State> { EventResult::NotHandled }
    fn exit(&self) {}
}

pub trait State<E: Clone>: Actor<E> + core::fmt::Debug {
    type State: State<E> + core::fmt::Debug;
    const INITIAL_STATE: Self;

    fn get_levels(&self, levels: &mut [core::any::TypeId], ptr: usize);

    fn super_enter(&self, level: i32);
    fn super_handle(&self, event: E) -> EventResult<<Self as State<E>>::State>;
    fn super_exit(&self, level: i32);
}