///#![feature(trivial_bounds)]
pub mod channels;
pub mod connection;
pub mod disconnection;
pub mod discovery;
pub mod identify;
pub mod identity;
pub mod peer;
use std::{
    cmp::{max, min},
    env,
    fs::File,
    io::Read,
    sync::Mutex,
};

pub use identity::PeerId;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use serde::Deserialize;

pub mod webrtc;

pub mod service_impl;

pub mod network;
pub use self::network::*;

mod p2p_actions;
pub use p2p_actions::*;

mod p2p_config;
pub use p2p_config::*;

mod p2p_event;
pub use p2p_event::*;

mod p2p_state;
pub use p2p_state::*;

mod p2p_effects;
mod p2p_reducer;
pub use self::p2p_effects::*;

use redux::SubStore;
pub trait P2pStore<GlobalState>: SubStore<GlobalState, P2pState, SubAction = P2pAction> {}
impl<S, T: SubStore<S, P2pState, SubAction = P2pAction>> P2pStore<S> for T {}

pub use libp2p_identity;
pub use multiaddr;

use rand_distr::{Distribution, Exp};

#[derive(Debug, Clone, Deserialize)]
pub struct FuzzerConf {
    pub rng_seed: u64,
    pub max_extend_size: usize,
    // rates are in the 0-1000 range or none if the specific mutation is disabled
    pub pnet_mutation_rate: Option<u32>,
    pub noise_mutation_rate: Option<u32>,
    pub select_authentication_mutation_rate: Option<u32>,
    pub select_multiplexing_mutation_rate: Option<u32>,
    pub select_stream_mutation_rate: Option<u32>,
    pub yamux_frame_mutation_rate: Option<u32>,
    pub identify_msg_mutation_rate: Option<u32>,
    pub kad_data_mutation_rate: Option<u32>,
    pub rpc_data_mutation_rate: Option<u32>,
    pub pubsub_mutation_rate: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct FuzzerState {
    pub rng: SmallRng,
    pub conf: FuzzerConf,
}

#[derive(Debug)]
enum MutationStrategy {
    FlipBits = 0,
    ExtendRandom = 1,
    ExtendCopy = 2,
    Shrink = 3,
}

impl MutationStrategy {
    pub fn range() -> std::ops::RangeInclusive<u32> {
        MutationStrategy::FlipBits as u32..=MutationStrategy::Shrink as u32
    }
}

impl From<u32> for MutationStrategy {
    fn from(item: u32) -> Self {
        match item {
            0 => MutationStrategy::FlipBits,
            1 => MutationStrategy::ExtendRandom,
            2 => MutationStrategy::ExtendCopy,
            3 => MutationStrategy::Shrink,
            _ => unreachable!(),
        }
    }
}

impl FuzzerState {
    pub fn gen_ratio(&mut self, numerator: Option<u32>) -> bool {
        if let Some(numerator) = numerator {
            self.rng.gen_ratio(numerator, 1000)
        } else {
            false
        }
    }

    pub fn gen_flips(&mut self, max_size: usize) -> usize {
        // Favor small number of flips
        let exp = Exp::new(2.0).unwrap();
        let v = exp.sample(&mut self.rng);

        min(max(1, (v * 10.0) as usize), max_size)
    }

    pub fn flip_bits(&mut self, data: &mut u8) {
        let num_bit_flips = self.gen_flips(8);

        for _ in 0..num_bit_flips {
            *data ^= 1u8 << self.rng.gen_range(0..8);
        }
    }

    pub fn flip_bytes(&mut self, data: &mut [u8]) {
        let max_size = data.len();
        let num_byte_flips = self.gen_flips(max_size);

        for _ in 0..num_byte_flips {
            let rnd_index = self.rng.gen_range(0..max_size);

            self.flip_bits(&mut data[rnd_index]);
        }
    }

    pub fn extend_random(&mut self, data: &[u8]) -> Vec<u8> {
        let extend_size = self.rng.gen_range(1..self.conf.max_extend_size);
        let random_pos = self.rng.gen_range(0..data.len());

        let mut random_bytes = vec![0u8; extend_size];
        self.rng.fill(&mut random_bytes[..]);

        let mut result = Vec::with_capacity(data.len() + extend_size);

        result.extend(&data[0..random_pos]);
        result.extend(random_bytes);
        result.extend(&data[random_pos..]);
        result
    }

    pub fn extend_copy(&mut self, data: &[u8]) -> Vec<u8> {
        let random_pos = self.rng.gen_range(0..data.len());
        let copy_pos = self.rng.gen_range(0..data.len() - 1);
        let copy_size = self.rng.gen_range(1..data.len() - copy_pos);
        let mut extend_size = self.rng.gen_range(copy_size..self.conf.max_extend_size);
        let mut result = Vec::with_capacity(data.len() + extend_size);

        result.extend(&data[0..random_pos]);

        while extend_size > copy_size {
            result.extend(&data[copy_pos..copy_pos + copy_size]);
            //self.flip_bytes(result.as_mut_slice());
            extend_size -= copy_size;
        }

        result.extend(&data[random_pos..]);
        result
    }

    pub fn shrink(&mut self, data: &[u8]) -> Vec<u8> {
        let pos = self.rng.gen_range(0..data.len() - 1);
        let size = self.rng.gen_range(1..data.len() - pos);
        let mut result = data[..pos].to_vec();

        result.extend_from_slice(&data[pos + size..]);
        result
    }

    pub fn mutate_pnet(&mut self, data: &mut Vec<u8>) {
        if self.gen_ratio(self.conf.pnet_mutation_rate) {
            let mutation_strategy: MutationStrategy =
                self.rng.gen_range(MutationStrategy::range()).into();

            println!(
                "[i] Mutating PNET data of len {} with strategy {:?}",
                data.len(),
                mutation_strategy
            );

            match mutation_strategy {
                MutationStrategy::FlipBits => self.flip_bytes(data.as_mut_slice()),
                MutationStrategy::ExtendRandom => {
                    *data = self.extend_random(data.as_slice());
                }
                MutationStrategy::ExtendCopy => {
                    *data = self.extend_copy(data.as_slice());
                }
                MutationStrategy::Shrink => {
                    *data = self.shrink(data.as_slice());
                }
            }
        }
    }

    pub fn mutate_noise(&mut self, data: &mut Data) {
        if self.gen_ratio(self.conf.noise_mutation_rate) {
            let mutation_strategy: MutationStrategy =
                self.rng.gen_range(MutationStrategy::range()).into();

            println!(
                "[i] Mutating Noise data of len {} with strategy {:?}",
                data.len(),
                mutation_strategy
            );

            match mutation_strategy {
                MutationStrategy::FlipBits => self.flip_bytes(data.0.as_mut()),
                MutationStrategy::ExtendRandom => {
                    data.0 = self.extend_random(data.0.as_ref()).as_slice().into();
                }
                MutationStrategy::ExtendCopy => {
                    data.0 = self.extend_copy(data.0.as_ref()).as_slice().into();
                }
                MutationStrategy::Shrink => {
                    data.0 = self.shrink(data.0.as_ref()).as_slice().into();
                }
            }
        }
    }

    pub fn mutate_select_authentication(&mut self, data: &mut Data) {
        if self.gen_ratio(self.conf.select_authentication_mutation_rate) {
            let mutation_strategy: MutationStrategy =
                self.rng.gen_range(MutationStrategy::range()).into();

            println!(
                "[i] Mutating Select authentication data of len {} with strategy {:?}",
                data.len(),
                mutation_strategy
            );

            match mutation_strategy {
                MutationStrategy::FlipBits => self.flip_bytes(data.0.as_mut()),
                MutationStrategy::ExtendRandom => {
                    data.0 = self.extend_random(data.0.as_ref()).as_slice().into();
                }
                MutationStrategy::ExtendCopy => {
                    data.0 = self.extend_copy(data.0.as_ref()).as_slice().into();
                }
                MutationStrategy::Shrink => {
                    data.0 = self.shrink(data.0.as_ref()).as_slice().into();
                }
            }
        }
    }

    pub fn mutate_select_multiplexing(&mut self, data: &mut Data) {
        if self.gen_ratio(self.conf.select_multiplexing_mutation_rate) {
            let mutation_strategy: MutationStrategy =
                self.rng.gen_range(MutationStrategy::range()).into();

            println!(
                "[i] Mutating Select multiplexing data of len {} with strategy {:?}",
                data.len(),
                mutation_strategy
            );

            match mutation_strategy {
                MutationStrategy::FlipBits => self.flip_bytes(data.0.as_mut()),
                MutationStrategy::ExtendRandom => {
                    data.0 = self.extend_random(data.0.as_ref()).as_slice().into();
                }
                MutationStrategy::ExtendCopy => {
                    data.0 = self.extend_copy(data.0.as_ref()).as_slice().into();
                }
                MutationStrategy::Shrink => {
                    data.0 = self.shrink(data.0.as_ref()).as_slice().into();
                }
            }
        }
    }

    pub fn mutate_select_stream(&mut self, data: &mut Data) {
        if self.gen_ratio(self.conf.select_stream_mutation_rate) {
            let mutation_strategy: MutationStrategy =
                self.rng.gen_range(MutationStrategy::range()).into();

            println!(
                "[i] Mutating Select stream data of len {} with strategy {:?}",
                data.len(),
                mutation_strategy
            );

            match mutation_strategy {
                MutationStrategy::FlipBits => self.flip_bytes(data.0.as_mut()),
                MutationStrategy::ExtendRandom => {
                    data.0 = self.extend_random(data.0.as_ref()).as_slice().into();
                }
                MutationStrategy::ExtendCopy => {
                    data.0 = self.extend_copy(data.0.as_ref()).as_slice().into();
                }
                MutationStrategy::Shrink => {
                    data.0 = self.shrink(data.0.as_ref()).as_slice().into();
                }
            }
        }
    }

    pub fn mutate_yamux_frame(&mut self, data: &mut Data) {
        if self.gen_ratio(self.conf.yamux_frame_mutation_rate) {
            let mutation_strategy: MutationStrategy =
                self.rng.gen_range(MutationStrategy::range()).into();

            println!(
                "[i] Mutating Yamux frame data of len {} with strategy {:?}",
                data.len(),
                mutation_strategy
            );

            match mutation_strategy {
                MutationStrategy::FlipBits => self.flip_bytes(data.0.as_mut()),
                MutationStrategy::ExtendRandom => {
                    data.0 = self.extend_random(data.0.as_ref()).as_slice().into();
                }
                MutationStrategy::ExtendCopy => {
                    data.0 = self.extend_copy(data.0.as_ref()).as_slice().into();
                }
                MutationStrategy::Shrink => {
                    data.0 = self.shrink(data.0.as_ref()).as_slice().into();
                }
            }
        }
    }

    pub fn mutate_identify_msg(&mut self, data: &mut Data) {
        if self.gen_ratio(self.conf.identify_msg_mutation_rate) {
            let mutation_strategy: MutationStrategy =
                self.rng.gen_range(MutationStrategy::range()).into();

            println!(
                "[i] Mutating Identify data of len {} with strategy {:?}",
                data.len(),
                mutation_strategy
            );

            match mutation_strategy {
                MutationStrategy::FlipBits => self.flip_bytes(data.0.as_mut()),
                MutationStrategy::ExtendRandom => {
                    data.0 = self.extend_random(data.0.as_ref()).as_slice().into();
                }
                MutationStrategy::ExtendCopy => {
                    data.0 = self.extend_copy(data.0.as_ref()).as_slice().into();
                }
                MutationStrategy::Shrink => {
                    data.0 = self.shrink(data.0.as_ref()).as_slice().into();
                }
            }
        }
    }

    pub fn mutate_kad_data(&mut self, data: &mut Data) {
        if self.gen_ratio(self.conf.kad_data_mutation_rate) {
            let mutation_strategy: MutationStrategy =
                self.rng.gen_range(MutationStrategy::range()).into();

            println!(
                "[i] Mutating Kad data of len {} with strategy {:?}",
                data.len(),
                mutation_strategy
            );

            match mutation_strategy {
                MutationStrategy::FlipBits => self.flip_bytes(data.0.as_mut()),
                MutationStrategy::ExtendRandom => {
                    data.0 = self.extend_random(data.0.as_ref()).as_slice().into();
                }
                MutationStrategy::ExtendCopy => {
                    data.0 = self.extend_copy(data.0.as_ref()).as_slice().into();
                }
                MutationStrategy::Shrink => {
                    data.0 = self.shrink(data.0.as_ref()).as_slice().into();
                }
            }
        }
    }

    pub fn mutate_rpc_data(&mut self, data: &mut Data) {
        if self.gen_ratio(self.conf.rpc_data_mutation_rate) {
            let mutation_strategy: MutationStrategy =
                self.rng.gen_range(MutationStrategy::range()).into();

            println!(
                "[i] Mutating RPC data of len {} with strategy {:?}",
                data.len(),
                mutation_strategy
            );

            match mutation_strategy {
                MutationStrategy::FlipBits => self.flip_bytes(data.0.as_mut()),
                MutationStrategy::ExtendRandom => {
                    data.0 = self.extend_random(data.0.as_ref()).as_slice().into();
                }
                MutationStrategy::ExtendCopy => {
                    data.0 = self.extend_copy(data.0.as_ref()).as_slice().into();
                }
                MutationStrategy::Shrink => {
                    data.0 = self.shrink(data.0.as_ref()).as_slice().into();
                }
            }
        }
    }

    pub fn mutate_pubsub(&mut self, data: &mut Data) {
        if self.gen_ratio(self.conf.pubsub_mutation_rate) {
            let mutation_strategy: MutationStrategy =
                self.rng.gen_range(MutationStrategy::range()).into();

            println!(
                "[i] Mutating PubSub data of len {} with strategy {:?}",
                data.len(),
                mutation_strategy
            );

            match mutation_strategy {
                MutationStrategy::FlipBits => self.flip_bytes(data.0.as_mut()),
                MutationStrategy::ExtendRandom => {
                    data.0 = self.extend_random(data.0.as_ref()).as_slice().into();
                }
                MutationStrategy::ExtendCopy => {
                    data.0 = self.extend_copy(data.0.as_ref()).as_slice().into();
                }
                MutationStrategy::Shrink => {
                    data.0 = self.shrink(data.0.as_ref()).as_slice().into();
                }
            }
        }
    }
}

use ctor::ctor;
use lazy_static::lazy_static;

lazy_static! {
    static ref FUZZ: Mutex<Option<FuzzerState>> = Mutex::new(None);
}

#[ctor]
fn init_fuzzer() {
    let mut fuzzer = FUZZ.lock().unwrap();

    let fuzzer_state = if let Ok(conf) = env::var("FUZZER_CONF") {
        let mut file = File::open(conf).expect("Failed to open fuzzer conf file");
        let mut contents = String::new();

        file.read_to_string(&mut contents)
            .expect("Failed to read fuzzer conf file");

        let conf: FuzzerConf =
            serde_json::from_str(&contents).expect("Failed to parse fuzzer configuration");

        println!("[+] FUZZER conf: {:?}", conf);

        Some(FuzzerState {
            rng: SmallRng::seed_from_u64(conf.rng_seed),
            conf,
        })
    } else {
        println!("[-] FUZZER disabled");
        None
    };

    *fuzzer = fuzzer_state;
}

/// Returns true if duration value is configured, and, given the time is `now`,
/// that duration is passed since `then`.
fn is_time_passed(
    now: redux::Timestamp,
    then: redux::Timestamp,
    duration: Option<std::time::Duration>,
) -> bool {
    duration.map_or(false, |d| now.checked_sub(then) >= Some(d))
}
