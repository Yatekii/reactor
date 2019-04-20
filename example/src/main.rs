use reactor_derive::{
    StateMachine,
    state_machine,
};

use reactor::Reactor;
use reactor::base::EventResult;
use reactor::base::State;
use reactor::base::Actor;

state_machine!{
    enum Outer2 {
        enum Bla {
            A1, B1,
        },
        C1,
    }
}

impl Actor<Event> for Outer2 {
    type State = Outer2;
}

impl Actor<Event> for Bla {
    type State = Outer2;
}

impl Actor<Event> for C1 {
    type State = Outer2;
}

impl Actor<Event> for A1 {
    type State = Outer2;
}

impl Actor<Event> for B1 {
    type State = Outer2;
}

#[derive(StateMachine)]
#[event(Event)]
#[state(Outer)]
enum Outer {
    A(Option<A>),
    B(Option<B>),
    C,
}

#[derive(StateMachine)]
#[event(Event)]
#[state(Outer)]
enum A {
    IAA,
    IAB,
}

#[derive(StateMachine)]
#[event(Event)]
#[state(Outer)]
enum B {
    IBA,
    IBB,
}

#[derive(Copy, Clone, Debug)]
enum Event {
    U,
    V,
}

impl Outer {
    fn A_() -> Self {
        let a = Outer::A(None);
        a.enter();
        a
    }

    fn B_() -> Self {
        let b = Outer::B(None);
        b.enter();
        b
    }
}

impl Actor<Event> for Outer {
    type State = Outer;

    fn enter(&self) {
        println!("Enter Outer");
    }
    
    fn handle(&self, event: Event) -> EventResult<Self::State> {
        println!("Outer Event({})",
            match event {
                Event::U => "U",
                Event::V => "V",
            }
        );
        match event {
            Event::U => EventResult::Handled,
            Event::V => EventResult::Transition(Outer::A(Some(A::IAA))),
        }
    }

    fn exit(&self) {
        println!("Exit Outer");
    }
}

impl Actor<Event> for A {
    type State = Outer;

    fn enter(&self) {
        println!("Enter A");
    }
    
    fn handle(&self, event: Event) -> EventResult<Self::State> {
        println!("A Event({})",
            match event {
                Event::U => "U",
                Event::V => "V",
            }
        );
        EventResult::Handled
    }

    fn exit(&self) {
        println!("Exit A");
    }
}

impl Actor<Event> for B {
    type State = Outer;

    fn enter(&self) {
        println!("Enter B");
    }
    
    fn handle(&self, event: Event) -> EventResult<Self::State> {
        println!("B Event({})",
            match event {
                Event::U => "U",
                Event::V => "V",
            }
        );
        EventResult::Handled
    }

    fn exit(&self) {
        println!("Exit B");
    }
}

fn main() {
    let mut reactor: Reactor<Outer, Event> = Reactor::new();
    reactor.react(Event::U);
    reactor.react(Event::V);
}
