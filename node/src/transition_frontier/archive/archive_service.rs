use mina_p2p_messages::v2::ArchiveTransitionFronntierDiff;

pub trait ArchiveService: redux::Service {
    fn send_to_archive(&mut self, data: ArchiveTransitionFronntierDiff);
}
