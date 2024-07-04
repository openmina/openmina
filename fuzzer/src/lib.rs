use rand::{rngs::SmallRng, Rng, SeedableRng};
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
    pub yamux_flags_mutation_rate: Option<u32>,
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
pub enum MutationStrategy {
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
}

use lazy_static::lazy_static;
use serde::Deserialize;
use std::{
    cmp::{max, min},
    env,
    fs::File,
    sync::Mutex,
};

lazy_static! {
    pub static ref FUZZER: Option<Mutex<FuzzerState>> = {
        if let Ok(conf) = env::var("FUZZER_CONF") {
            let file = File::open(conf).expect("Failed to open fuzzer conf file");
            let conf: FuzzerConf =
                serde_json::from_reader(file).expect("Failed to parse fuzzer configuration");

            println!("[+] FUZZER conf: {:?}", conf);

            Some(Mutex::new(FuzzerState {
                rng: SmallRng::seed_from_u64(conf.rng_seed),
                conf,
            }))
        } else {
            println!("[-] no FUZZER conf specified");
            None
        }
    };
}

#[macro_export]
macro_rules! fuzz {
    ($expr:expr, $mutator:expr) => {
        if let Some(fuzzer) = $crate::FUZZER.as_ref() {
            let mut fuzzer = fuzzer.lock().expect("can't lock FUZZER");
            $mutator(&mut fuzzer, $expr);
        }
    };
}

#[macro_export]
macro_rules! fuzzed {
    ($expr:expr, $mutator:expr) => {
        if let Some(fuzzer) = $crate::FUZZER.as_ref() {
            let mut fuzzer = fuzzer.lock().expect("can't lock FUZZER");
            let mut fuzzed = $expr;
            $mutator(&mut fuzzer, &mut fuzzed);
            fuzzed
        } else {
            $expr
        }
    };
}
