use crate::base::*;

pub struct Reactor<S: State<E, State = S>, E: Clone> {
    state: S,
    _marker: core::marker::PhantomData<E>
}

impl<S: Root<E> + State<E, State = S> + std::fmt::Debug, E: Clone> Reactor<S, E> {
    /// Creates a new Reactor with the initial state.
    /// TDDO: Use attributes to determine the initial state!
    pub fn new() -> Self {
        let reactor = Self {
            state: <S as State<E>>::State::INITIAL_STATE,
            _marker: core::marker::PhantomData
        };
        reactor.state.super_enter(0);
        reactor
    }

    /// Let the Reactor handle and event.
    /// This logic is flawed atm, because it will always exit to the top of the state tree and enter down to the new state,
    /// instead of just exiting to the first common denominator state.
    /// TODO: Fix this behavior.
    pub fn react(&mut self, event: E) {
        match self.state.super_handle(event) {
            EventResult::Transition(new_state) => {
                // TODO: Make this generic!
                let levels_new = &mut [core::any::TypeId::of::<bool>(); 12];
                let levels_old = &mut [core::any::TypeId::of::<bool>(); 12];
                new_state.get_levels(levels_new, 0);
                self.state.get_levels(levels_old, 0);

                let mut i = 0;
                while i < S::MAX_LEVELS {
                    if levels_new[i] != levels_old[i] {
                        break;
                    }
                    i += 1;
                }

                // let old_level = self.state.level();
                // let new_level = new_state.level();
                // let difference = old_level - new_level;

                println!("Moving {:?} -> {:?}", self.state, new_state);

                // println!("Old path: {:#?}", levels_old);
                // println!("Old path: {:#?}", levels_new);
                // println!("{} - {} = {}", old_level, new_level, difference);
                
                self.state.super_exit(i as i32);
                self.state = new_state;
                self.state.super_enter(i as i32);
            }
            _ => {},
        }
    }
}