use super::Event;

pub trait EventSourceService {
    fn next_event(&mut self) -> Option<Event>;
}
