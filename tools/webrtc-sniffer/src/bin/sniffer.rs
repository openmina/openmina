use std::path::PathBuf;

use clap::Parser;
use pcap::{Capture, ConnectionStatus, Device, IfFlags};

use p2p::identity::SecretKey;

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

    /// Peer secret key
    #[arg(long, short = 's', env = "OPENMINA_P2P_SEC_KEY")]
    p2p_secret_key: Option<SecretKey>,

    // warning, this overrides `OPENMINA_P2P_SEC_KEY`
    /// Compatibility with OCaml Mina node
    #[arg(long)]
    libp2p_keypair: Option<String>,

    // warning, this overrides `OPENMINA_P2P_SEC_KEY`
    /// Compatibility with OCaml Mina node
    #[arg(env = "MINA_LIBP2P_PASS")]
    libp2p_password: Option<String>,
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
        p2p_secret_key,
        libp2p_keypair,
        libp2p_password,
    } = Cli::parse();

    let secret_key = if let Some(v) = p2p_secret_key {
        v
    } else {
        let (Some(libp2p_keypair), Some(libp2p_password)) = (libp2p_keypair, libp2p_password)
        else {
            log::error!("no secret key specified");
            return;
        };

        match SecretKey::from_encrypted_file(libp2p_keypair, &libp2p_password) {
            Ok(v) => v,
            Err(err) => {
                log::error!("cannot read secret key {err}");
                return;
            }
        }
    };

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

                webrtc_sniffer::run(capture, Some(savefile), secret_key)
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
            webrtc_sniffer::run(capture, None, secret_key)
        });
        if let Err(err) = res {
            log::error!("{err}");
        }
    }
}
