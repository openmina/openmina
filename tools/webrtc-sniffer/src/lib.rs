mod net;

use pcap::{Activated, Capture, Savefile};

pub fn run<T: Activated + ?Sized>(
    capture: Capture<T>,
    file: Option<Savefile>,
) -> Result<(), net::DissectError> {
    for item in net::UdpIter::new(capture, file) {
        let (src, dst, data) = item?;
        log::info!(
            "{src} -> {dst}: {} {}",
            data.len(),
            hex::encode(&data[..data.len().min(12)])
        );
    }

    Ok(())
}
