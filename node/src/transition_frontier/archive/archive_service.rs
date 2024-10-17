pub trait ArchiveService: redux::Service {
    fn send_to_archive(&mut self, data: ());
}
