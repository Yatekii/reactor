use crate::base::*;

pub struct Reactor<S, E> {
    state: S,
    _marker: core::marker::PhantomData<E>
}

impl<S: State<E>, E: Clone> Reactor<S, E> {
    /// Creates a new Reactor with the initial state.
    /// TDDO: Use attributes to determine the initial state!
    pub fn new() -> Self {
        let reactor = Self {
            state: S::INITIAL_STATE,
            _marker: core::marker::PhantomData
        };
        reactor.state.super_enter();
        reactor
    }

    /// Let the Reactor handle and event.
    /// This logic is flawed atm, because it will always exit to the top of the state tree and enter down to the new state,
    /// instead of just exiting to the first common denominator state.
    /// TODO: Fix this behavior.
    pub fn react(&mut self, event: E) {
        match self.state.super_handle(event) {
            EventResult::Transition(new_state) => {
                self.state.super_exit();
                new_state.super_enter();
            }
            _ => {},
        }
    }
}