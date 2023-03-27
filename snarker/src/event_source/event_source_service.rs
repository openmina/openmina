use super::Event;

pub trait EventSourceService: redux::Service {
    fn next_event(&mut self) -> Option<Event>;
}
