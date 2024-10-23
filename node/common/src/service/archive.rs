use binprot::BinProtWrite;
use mina_p2p_messages::v2::ArchiveTransitionFronntierDiff;
use node::core::{channels::mpsc, thread};

use super::NodeService;

pub struct ArchiveService {
    archive_sender: mpsc::UnboundedSender<ArchiveTransitionFronntierDiff>,
}

impl ArchiveService {
    fn new(archive_sender: mpsc::UnboundedSender<ArchiveTransitionFronntierDiff>) -> Self {
        Self { archive_sender }
    }

    fn run(
        mut archive_receiver: mpsc::UnboundedReceiver<ArchiveTransitionFronntierDiff>,
        address: &str,
    ) {
        while let Some(breadcrumb) = archive_receiver.blocking_recv() {
            println!("Sending data to archive");
            // TODO(adonagy): Async?
            let mut data: Vec<u8> = Vec::new();

            if let Err(e) = breadcrumb.binprot_write(&mut data) {
                println!("Error writing to binprot: {:?}", e);
                continue;
            }

            println!("[archive] msg length: {}", data.len());

            // let url = "http://adonagy.hz.minaprotocol.network:3086/v2/archive";
            if let Err(e) = reqwest::blocking::Client::new()
                .post(address)
                .body(data)
                .send()
            {
                println!("Error sending to archive: {:?}", e);
            }
        }
    }

    pub fn start(address: &str) -> Self {
        let (archive_sender, archive_receiver) =
            mpsc::unbounded_channel::<ArchiveTransitionFronntierDiff>();

        let address = address.to_string();
        thread::Builder::new()
            .name("openmina_archive".to_owned())
            .spawn(move || {
                Self::run(archive_receiver, &address);
            })
            .unwrap();

        Self::new(archive_sender)
    }
}

impl node::transition_frontier::archive::archive_service::ArchiveService for NodeService {
    fn send_to_archive(&mut self, data: ArchiveTransitionFronntierDiff) {
        if let Some(archive) = self.archive.as_mut() {
            let _ = archive.archive_sender.send(data);
        }
    }
}
