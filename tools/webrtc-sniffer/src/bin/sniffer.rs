use std::path::PathBuf;

use clap::Parser;
use pcap::{Capture, ConnectionStatus, Device, IfFlags};

// cargo run --release --bin sniffer -- --interface auto --path target/test.pcap

#[derive(Parser)]
struct Cli {
    #[arg(
        long,
        help = "name of the interface, use `auto` to determine automatically"
    )]
    interface: Option<String>,

    #[arg(
        long,
        help = "if `interface` is set, the packets will be written to the `pcap` file, \
                otherwise the file will be a source of packets"
    )]
    path: PathBuf,

    #[arg(long, help = "bpf filter, example: \"udp and not port 443\"")]
    filter: Option<String>,

    /// rng seed
    #[arg(long, short)]
    rng_seed: String,
}

fn init_logger_std() -> Box<dyn log::Log> {
    use env_logger::{Builder, Env};

    let env = Env::new().filter_or("RUST_LOG", "debug");
    let logger = Builder::default().parse_env(env).build();
    Box::new(logger) as Box<dyn log::Log>
}

fn main() {
    log::set_boxed_logger(init_logger_std()).unwrap_or_default();
    log::set_max_level(log::LevelFilter::max());

    let Cli {
        interface,
        path,
        filter,
        rng_seed,
    } = Cli::parse();

    let rng_seed = <[u8; 32]>::try_from(hex::decode(rng_seed).unwrap().as_slice()).unwrap();

    if let Some(name) = interface {
        sudo::escalate_if_needed().unwrap();

        log::info!("try to choose device");
        let mut selected = None;
        match Device::list() {
            Ok(list) => {
                for device in list {
                    if name != "auto" {
                        if device.name.eq(&name) {
                            selected = Some(device);
                            break;
                        }
                    } else {
                        log::debug!("candidate: {device:?}");
                        if !device.addresses.is_empty()
                            && device.flags.contains(IfFlags::UP | IfFlags::RUNNING)
                            && matches!(device.flags.connection_status, ConnectionStatus::Connected)
                        {
                            selected = Some(device);
                        }
                    }
                }
            }
            Err(err) => log::error!("{err}"),
        }

        if let Some(device) = selected {
            log::info!("will use: {device:?}");
            let res = Ok(()).and_then(|()| {
                let mut capture = Capture::from_device(device)?.immediate_mode(true).open()?;
                capture
                    .filter(&filter.unwrap_or_default(), true)
                    .expect("Failed to apply filter");
                let savefile = capture.savefile(&path)?;

                webrtc_sniffer::run(capture, Some(savefile), rng_seed)
            });
            if let Err(err) = res {
                log::error!("{err}");
            }
        } else {
            log::error!("cannot find a device: {name}");
        }
    } else {
        log::info!("use file");
        let res = Ok(()).and_then(|()| {
            let mut capture = Capture::from_file(&path)?;
            capture
                .filter(&filter.unwrap_or_default(), true)
                .expect("Failed to apply filter");
            webrtc_sniffer::run(capture, None, rng_seed)
        });
        if let Err(err) = res {
            log::error!("{err}");
        }
    }
}
