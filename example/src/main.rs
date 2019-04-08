use reactor_derive::StateMachine;

use reactor::Reactor;
use reactor::base::EventResult;
use reactor::base::State;
use reactor::base::Actor;

#[derive(StateMachine)]
enum Outer {
    A(Option<A>),
    B(Option<B>),
    C,
}

#[derive(StateMachine)]
enum A {
    IAA,
    IAB,
}

#[derive(StateMachine)]
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
    fn enter(&self) {
        println!("Enter Outer");
    }
    
    fn handle<O>(&self, event: Event) -> EventResult<O> {
        println!("Outer Event({})",
            match event {
                Event::U => "U",
                Event::V => "V",
            }
        );
        EventResult::Handled
    }

    fn exit(&self) {
        println!("Exit Outer");
    }
}

impl Actor<Event> for A {
    fn enter(&self) {
        println!("Enter A");
    }
    
    fn handle<O>(&self, event: Event) -> EventResult<O> {
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
    fn enter(&self) {
        println!("Enter B");
    }
    
    fn handle<O>(&self, event: Event) -> EventResult<O> {
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
}
