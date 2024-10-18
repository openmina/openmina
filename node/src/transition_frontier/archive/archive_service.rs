use mina_p2p_messages::v2::ArchiveBreadcrumb;

pub trait ArchiveService: redux::Service {
    fn send_to_archive(&mut self, data: ArchiveBreadcrumb);
}
