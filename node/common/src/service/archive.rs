use mina_p2p_messages::v2::ArchiveBreadcrumb;
use node::core::{channels::mpsc, thread};

use super::NodeService;

pub struct ArchiveService {
    archive_sender: mpsc::UnboundedSender<ArchiveBreadcrumb>,
}

impl ArchiveService {
    fn new(archive_sender: mpsc::UnboundedSender<ArchiveBreadcrumb>) -> Self {
        Self { archive_sender }
    }

    fn run(mut archive_receiver: mpsc::UnboundedReceiver<ArchiveBreadcrumb>) {
        while let Some(breadcrumb) = archive_receiver.blocking_recv() {
            println!("Sending data to archive");
        }
    }

    pub fn start() -> Self {
        let (archive_sender, archive_receiver) = mpsc::unbounded_channel::<ArchiveBreadcrumb>();

        thread::Builder::new()
            .name("openmina_archive".to_owned())
            .spawn(move || {
                Self::run(archive_receiver);
            })
            .unwrap();

        Self::new(archive_sender)
    }
}

impl node::transition_frontier::archive::archive_service::ArchiveService for NodeService {
    fn send_to_archive(&mut self, data: ArchiveBreadcrumb) {
        if let Some(archive) = self.archive.as_mut() {
            let _ = archive.archive_sender.send(data);
        }
    }
}
