pub trait React<E> {
    /// Creates a new Reactor with the initial state.
    /// TDDO: Use attributes to determine the initial state!
    fn new() -> Self;

    /// Let the Reactor handle and event.
    /// This logic is flawed atm, because it will always exit to the top of the state tree and enter down to the new state,
    /// instead of just exiting to the first common denominator state.
    /// TODO: Fix this behavior.
    fn react(&mut self, event: E) where E: Clone;
}