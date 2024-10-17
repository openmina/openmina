use node::core::{channels::mpsc, thread};

use super::NodeService;

pub struct ArchiveService {
    archive_sender: mpsc::UnboundedSender<()>,
}

impl ArchiveService {
    fn new(archive_sender: mpsc::UnboundedSender<()>) -> Self {
        Self { archive_sender }
    }

    fn run(mut archive_receiver: mpsc::UnboundedReceiver<()>) {
        while let Some(()) = archive_receiver.blocking_recv() {
            println!("Sending data to archive");
        }
    }

    pub fn start() -> Self {
        let (archive_sender, archive_receiver) = mpsc::unbounded_channel::<()>();

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
    fn send_to_archive(&mut self, data: ()) {
        if let Some(archive) = self.archive.as_mut() {
            let _ = archive.archive_sender.send(data);
        }
    }
}
