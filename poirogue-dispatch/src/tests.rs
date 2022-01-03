
#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use dispatcher::Dispatcher;
    use event::{Envelope, Event};
    use listener::EventListener;

    // AN EVENT YOU CAN STOP

    struct StoppableTestEvent {
        pub vals: Vec<String>,
        blocked: bool,
    }

    impl Event for StoppableTestEvent {
        fn is_propagating(&self) -> bool { !self.blocked }
        fn stop_propagation(&mut self) { self.blocked = true; }
    }

    impl StoppableTestEvent {
        pub fn new() -> StoppableTestEvent {
            StoppableTestEvent { vals: Vec::new(), blocked: false }
        }
    }

    // AN EVENT YOU CAN'T STOP

    struct NoStopTestEvent {
        pub vals: Vec<String>,
    }

    impl NoStopTestEvent {
        pub fn new() -> NoStopTestEvent {
            NoStopTestEvent { vals: Vec::new() }
        }
    }

    impl Event for NoStopTestEvent {
        fn is_propagating(&self) -> bool { true }
        fn stop_propagation(&mut self) {}
    }

    // TESTS

    #[test]
    fn test_event_dispatch_no_listeners() {
        let dispatcher = Dispatcher::new();
        let mut event = NoStopTestEvent::new();
        dispatcher.dispatch(&mut event);
    }

    #[test]
    fn test_event_dispatch_to_listener() {
        let mut dispatcher = Dispatcher::new();
        let mut event = NoStopTestEvent::new();
        event.vals.push("hello".to_string());
        let listener = EventListener::new(
            |e: &mut NoStopTestEvent| { assert_eq!(*e.vals.get(0).unwrap(), "hello".to_string()); });

        dispatcher.add_listener(listener);
        assert!(dispatcher.has_listeners());

        dispatcher.dispatch(&mut event);
    }

    lazy_static! {
        static ref CHANNEL: Mutex<Vec<String>> = Mutex::new(Vec::new());
    }

    #[test]
    fn test_event_dispatch_with_priority() {
        let mut dispatcher = Dispatcher::new();
        let mut event = NoStopTestEvent::new();

        let slow = EventListener::new_with_priority(
            |e: &mut NoStopTestEvent| { e.vals.push("slow".to_string()); }, 5);
        let fast = EventListener::new_with_priority(
            |e: &mut NoStopTestEvent| { e.vals.push("fast".to_string()) }, 10);

        dispatcher.add_listener(slow);
        dispatcher.add_listener(fast);
        assert!(dispatcher.has_listeners());

        dispatcher.dispatch(&mut event);
        let v = event.vals;

        assert_eq!(v.iter().len(), 2);
        assert_eq!(*v.get(0).unwrap(), "fast".to_string());
        assert_eq!(*v.get(1).unwrap(), "slow".to_string());
    }

    #[test]
    fn test_event_dispatch_with_blocking() {
        { /* NON-BLOCKING FIRST */
            let mut event = StoppableTestEvent::new();

            let non_blocking = EventListener::new(
                |e: &mut StoppableTestEvent| { e.vals.push("non block".to_string()); });

            let blocking = EventListener::new(
                |e: &mut StoppableTestEvent| { e.vals.push("block".to_string()); e.stop_propagation(); });

            let mut dispatcher = Dispatcher::new();
            dispatcher.add_listener(non_blocking);
            dispatcher.add_listener(blocking);
            assert!(dispatcher.has_listeners());

            dispatcher.dispatch(&mut event);
            let v = event.vals;

            assert_eq!(v.iter().len(), 2);
            assert_eq!(*v.get(0).unwrap(), "non block".to_string());
            assert_eq!(*v.get(1).unwrap(), "block".to_string());
        }

        { /* BLOCKING FIRST */
            let mut event = StoppableTestEvent::new();

            let non_blocking = EventListener::new(
                |e: &mut StoppableTestEvent| { e.vals.push("non block".to_string()); });
            let blocking = EventListener::new(
                |e: &mut StoppableTestEvent| { e.vals.push("block".to_string()); e.stop_propagation(); });

            let mut dispatcher = Dispatcher::new();
            dispatcher.add_listener(blocking);
            dispatcher.add_listener(non_blocking);
            assert!(dispatcher.has_listeners());

            dispatcher.dispatch(&mut event);
            let v = event.vals;

            assert_eq!(v.iter().len(), 1);
            assert_eq!(*v.get(0).unwrap(), "block".to_string());
        }
    }

    #[test]
    fn test_event_dispatch_with_blocking_priority() {
        let mut dispatcher = Dispatcher::new();
        let mut event = StoppableTestEvent::new();

        let slow = EventListener::new_with_priority(
            |e: &mut StoppableTestEvent| { e.vals.push("slow".to_string()); }, 5);
        let fast = EventListener::new_with_priority(
            |e: &mut StoppableTestEvent| { e.vals.push("fast".to_string()); e.stop_propagation(); }, 10);

        dispatcher.add_listener(fast);
        dispatcher.add_listener(slow);
        assert!(dispatcher.has_listeners());

        dispatcher.dispatch(&mut event);
        let v = event.vals;

        assert_eq!(v.iter().len(), 1);
        assert_eq!(*v.get(0).unwrap(), "fast".to_string());
    }

    struct IntAndBool {
        pub int: i32,
        pub boo: bool,
    }

    #[test]
    fn test_event_dispatch_with_envelopes() {
        let mut dispatcher = Dispatcher::new();

        let listener = EventListener::new(
            |e: &mut Envelope<IntAndBool>| {
                assert_eq!(e.data.int, 3);
                assert_eq!(e.data.boo, true);
            });

        let mut env = Envelope::new(IntAndBool{ int: 3, boo: true });
        dispatcher.add_listener(listener);
        assert!(dispatcher.has_listeners());

        dispatcher.dispatch(&mut env);
    }
}