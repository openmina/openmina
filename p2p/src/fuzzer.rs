use openmina_fuzzer::{FuzzerState, MutationStrategy};
use rand::Rng;

use crate::{Data, YamuxFlags};

pub fn mutate_pnet(fuzzer: &mut FuzzerState, data: &mut Vec<u8>) {
    if fuzzer.gen_ratio(fuzzer.conf.pnet_mutation_rate) {
        let mutation_strategy: MutationStrategy = if !data.is_empty() {
            fuzzer.rng.gen_range(MutationStrategy::range()).into()
        } else {
            MutationStrategy::ExtendRandom
        };

        println!(
            "[i] Mutating PNET data of len {} with strategy {:?}",
            data.len(),
            mutation_strategy
        );

        match mutation_strategy {
            MutationStrategy::FlipBits => fuzzer.flip_bytes(data.as_mut_slice()),
            MutationStrategy::ExtendRandom => {
                *data = fuzzer.extend_random(data.as_slice());
            }
            MutationStrategy::ExtendCopy => {
                *data = fuzzer.extend_copy(data.as_slice());
            }
            MutationStrategy::Shrink => {
                *data = fuzzer.shrink(data.as_slice());
            }
        }
    }
}

pub fn mutate_noise(fuzzer: &mut FuzzerState, data: &mut Data) {
    if fuzzer.gen_ratio(fuzzer.conf.noise_mutation_rate) {
        let mutation_strategy: MutationStrategy = if !data.is_empty() {
            fuzzer.rng.gen_range(MutationStrategy::range()).into()
        } else {
            MutationStrategy::ExtendRandom
        };

        println!(
            "[i] Mutating Noise data of len {} with strategy {:?}",
            data.len(),
            mutation_strategy
        );

        match mutation_strategy {
            MutationStrategy::FlipBits => fuzzer.flip_bytes(data.0.as_mut()),
            MutationStrategy::ExtendRandom => {
                data.0 = fuzzer.extend_random(data.0.as_ref()).as_slice().into();
            }
            MutationStrategy::ExtendCopy => {
                data.0 = fuzzer.extend_copy(data.0.as_ref()).as_slice().into();
            }
            MutationStrategy::Shrink => {
                data.0 = fuzzer.shrink(data.0.as_ref()).as_slice().into();
            }
        }
    }
}

pub fn mutate_select_authentication(fuzzer: &mut FuzzerState, data: &mut Data) {
    if fuzzer.gen_ratio(fuzzer.conf.select_authentication_mutation_rate) {
        let mutation_strategy: MutationStrategy = if !data.is_empty() {
            fuzzer.rng.gen_range(MutationStrategy::range()).into()
        } else {
            MutationStrategy::ExtendRandom
        };

        println!(
            "[i] Mutating Select authentication data of len {} with strategy {:?}",
            data.len(),
            mutation_strategy
        );

        match mutation_strategy {
            MutationStrategy::FlipBits => fuzzer.flip_bytes(data.0.as_mut()),
            MutationStrategy::ExtendRandom => {
                data.0 = fuzzer.extend_random(data.0.as_ref()).as_slice().into();
            }
            MutationStrategy::ExtendCopy => {
                data.0 = fuzzer.extend_copy(data.0.as_ref()).as_slice().into();
            }
            MutationStrategy::Shrink => {
                data.0 = fuzzer.shrink(data.0.as_ref()).as_slice().into();
            }
        }
    }
}

pub fn mutate_select_multiplexing(fuzzer: &mut FuzzerState, data: &mut Data) {
    if fuzzer.gen_ratio(fuzzer.conf.select_multiplexing_mutation_rate) {
        let mutation_strategy: MutationStrategy = if !data.is_empty() {
            fuzzer.rng.gen_range(MutationStrategy::range()).into()
        } else {
            MutationStrategy::ExtendRandom
        };

        println!(
            "[i] Mutating Select multiplexing data of len {} with strategy {:?}",
            data.len(),
            mutation_strategy
        );

        match mutation_strategy {
            MutationStrategy::FlipBits => fuzzer.flip_bytes(data.0.as_mut()),
            MutationStrategy::ExtendRandom => {
                data.0 = fuzzer.extend_random(data.0.as_ref()).as_slice().into();
            }
            MutationStrategy::ExtendCopy => {
                data.0 = fuzzer.extend_copy(data.0.as_ref()).as_slice().into();
            }
            MutationStrategy::Shrink => {
                data.0 = fuzzer.shrink(data.0.as_ref()).as_slice().into();
            }
        }
    }
}

pub fn mutate_select_stream(fuzzer: &mut FuzzerState, data: &mut Data) {
    if fuzzer.gen_ratio(fuzzer.conf.select_stream_mutation_rate) {
        let mutation_strategy: MutationStrategy = if !data.is_empty() {
            fuzzer.rng.gen_range(MutationStrategy::range()).into()
        } else {
            MutationStrategy::ExtendRandom
        };

        println!(
            "[i] Mutating Select stream data of len {} with strategy {:?}",
            data.len(),
            mutation_strategy
        );

        match mutation_strategy {
            MutationStrategy::FlipBits => fuzzer.flip_bytes(data.0.as_mut()),
            MutationStrategy::ExtendRandom => {
                data.0 = fuzzer.extend_random(data.0.as_ref()).as_slice().into();
            }
            MutationStrategy::ExtendCopy => {
                data.0 = fuzzer.extend_copy(data.0.as_ref()).as_slice().into();
            }
            MutationStrategy::Shrink => {
                data.0 = fuzzer.shrink(data.0.as_ref()).as_slice().into();
            }
        }
    }
}

pub fn mutate_yamux_frame(fuzzer: &mut FuzzerState, data: &mut Data) {
    if fuzzer.gen_ratio(fuzzer.conf.yamux_frame_mutation_rate) {
        let mutation_strategy: MutationStrategy = if !data.is_empty() {
            fuzzer.rng.gen_range(MutationStrategy::range()).into()
        } else {
            MutationStrategy::ExtendRandom
        };

        println!(
            "[i] Mutating Yamux frame data of len {} with strategy {:?}",
            data.len(),
            mutation_strategy
        );

        match mutation_strategy {
            MutationStrategy::FlipBits => fuzzer.flip_bytes(data.0.as_mut()),
            MutationStrategy::ExtendRandom => {
                data.0 = fuzzer.extend_random(data.0.as_ref()).as_slice().into();
            }
            MutationStrategy::ExtendCopy => {
                data.0 = fuzzer.extend_copy(data.0.as_ref()).as_slice().into();
            }
            MutationStrategy::Shrink => {
                data.0 = fuzzer.shrink(data.0.as_ref()).as_slice().into();
            }
        }
    }
}

pub fn mutate_identify_msg(fuzzer: &mut FuzzerState, data: &mut Data) {
    if fuzzer.gen_ratio(fuzzer.conf.identify_msg_mutation_rate) {
        let mutation_strategy: MutationStrategy = if !data.is_empty() {
            fuzzer.rng.gen_range(MutationStrategy::range()).into()
        } else {
            MutationStrategy::ExtendRandom
        };

        println!(
            "[i] Mutating Identify data of len {} with strategy {:?}",
            data.len(),
            mutation_strategy
        );

        match mutation_strategy {
            MutationStrategy::FlipBits => fuzzer.flip_bytes(data.0.as_mut()),
            MutationStrategy::ExtendRandom => {
                data.0 = fuzzer.extend_random(data.0.as_ref()).as_slice().into();
            }
            MutationStrategy::ExtendCopy => {
                data.0 = fuzzer.extend_copy(data.0.as_ref()).as_slice().into();
            }
            MutationStrategy::Shrink => {
                data.0 = fuzzer.shrink(data.0.as_ref()).as_slice().into();
            }
        }
    }
}

pub fn mutate_yamux_flags(fuzzer: &mut FuzzerState, flags: &mut YamuxFlags) {
    if fuzzer.gen_ratio(fuzzer.conf.yamux_flags_mutation_rate) {
        let new_flags = YamuxFlags::from_bits_truncate(fuzzer.rng.gen());
        println!("[i] Mutating flags {:?} -> {:?}", flags, new_flags);

        *flags = new_flags;
    }
}

pub fn mutate_kad_data(fuzzer: &mut FuzzerState, data: &mut Data) {
    if fuzzer.gen_ratio(fuzzer.conf.kad_data_mutation_rate) {
        let mutation_strategy: MutationStrategy = if !data.is_empty() {
            fuzzer.rng.gen_range(MutationStrategy::range()).into()
        } else {
            MutationStrategy::ExtendRandom
        };

        println!(
            "[i] Mutating Kad data of len {} with strategy {:?}",
            data.len(),
            mutation_strategy
        );

        match mutation_strategy {
            MutationStrategy::FlipBits => fuzzer.flip_bytes(data.0.as_mut()),
            MutationStrategy::ExtendRandom => {
                data.0 = fuzzer.extend_random(data.0.as_ref()).as_slice().into();
            }
            MutationStrategy::ExtendCopy => {
                data.0 = fuzzer.extend_copy(data.0.as_ref()).as_slice().into();
            }
            MutationStrategy::Shrink => {
                data.0 = fuzzer.shrink(data.0.as_ref()).as_slice().into();
            }
        }
    }
}

pub fn mutate_rpc_data(fuzzer: &mut FuzzerState, data: &mut Data) {
    if fuzzer.gen_ratio(fuzzer.conf.rpc_data_mutation_rate) {
        let mutation_strategy: MutationStrategy = if !data.is_empty() {
            fuzzer.rng.gen_range(MutationStrategy::range()).into()
        } else {
            MutationStrategy::ExtendRandom
        };

        println!(
            "[i] Mutating RPC data of len {} with strategy {:?}",
            data.len(),
            mutation_strategy
        );

        match mutation_strategy {
            MutationStrategy::FlipBits => fuzzer.flip_bytes(data.0.as_mut()),
            MutationStrategy::ExtendRandom => {
                data.0 = fuzzer.extend_random(data.0.as_ref()).as_slice().into();
            }
            MutationStrategy::ExtendCopy => {
                data.0 = fuzzer.extend_copy(data.0.as_ref()).as_slice().into();
            }
            MutationStrategy::Shrink => {
                data.0 = fuzzer.shrink(data.0.as_ref()).as_slice().into();
            }
        }
    }
}

pub fn mutate_pubsub(fuzzer: &mut FuzzerState, data: &mut Data) {
    if fuzzer.gen_ratio(fuzzer.conf.pubsub_mutation_rate) {
        let mutation_strategy: MutationStrategy = if !data.is_empty() {
            fuzzer.rng.gen_range(MutationStrategy::range()).into()
        } else {
            MutationStrategy::ExtendRandom
        };

        println!(
            "[i] Mutating PubSub data of len {} with strategy {:?}",
            data.len(),
            mutation_strategy
        );

        match mutation_strategy {
            MutationStrategy::FlipBits => fuzzer.flip_bytes(data.0.as_mut()),
            MutationStrategy::ExtendRandom => {
                data.0 = fuzzer.extend_random(data.0.as_ref()).as_slice().into();
            }
            MutationStrategy::ExtendCopy => {
                data.0 = fuzzer.extend_copy(data.0.as_ref()).as_slice().into();
            }
            MutationStrategy::Shrink => {
                data.0 = fuzzer.shrink(data.0.as_ref()).as_slice().into();
            }
        }
    }
}
