use std::println;
extern crate std;

use ed25519_dalek::Keypair;
use soroban_fixed_point_math::SorobanFixedPoint;
use soroban_sdk::{testutils::Address as _, xdr::ToXdr, Address, Bytes, BytesN, Env};
use stellar_strkey::{ed25519, Strkey};
use tiny_keccak::{Hasher, Keccak};

use crate::{
    types::{Block, Pail},
    BLOCKS_PER_MONTH, BLOCK_REWARD, BLOCK_SCALE, INVERSE_DECAY_RATE, V2_GENESIS_BLOCK,
};

#[test]
fn calc_emissions_rate() {
    let env = Env::default();

    fn calculate_block_reward(env: &Env, index: u32) -> i128 {
        let elapsed_time = index.saturating_sub(V2_GENESIS_BLOCK);
        let periods = elapsed_time.saturating_div(BLOCKS_PER_MONTH);

        let mut result = BLOCK_SCALE;

        for _ in 0..periods {
            result = result.fixed_mul_floor(env, &INVERSE_DECAY_RATE, &BLOCK_SCALE);
        }

        BLOCK_REWARD.fixed_mul_floor(env, &result, &BLOCK_SCALE)
    }

    // let target_date = 2556057600; // 2050-12-31
    // let target_date = 1767139200; // 2025-12-31
    let index = V2_GENESIS_BLOCK + (BLOCKS_PER_MONTH * 8);

    let emissions_amount = calculate_block_reward(&env, index);

    println!("{:?}", BLOCK_REWARD / 5);
    println!("{:?}", emissions_amount / 5); // 3317102156
}

#[test]
fn test_generate_reward() {
    let env = Env::default();

    let block = Block {
        entropy: BytesN::from_array(&env, &[0; 32]),
        max_gap: 50,
        max_stake: 1234747261419,
        max_zeros: 10,
        min_gap: 0,
        min_stake: 0,
        min_zeros: 5,
        normalized_total: 230227940395912,
        staked_total: 15000000,
        timestamp: 0,
    };

    let pail = Pail {
        sequence: 0,
        gap: Some(40),
        stake: 10000,
        zeros: Some(7),
    };

    let (normalized_gap, normalized_stake, normalized_zeros) = generate_normalizations_v2(
        &env,
        &block,
        pail.gap.unwrap(),
        pail.stake,
        pail.zeros.unwrap(),
    );

    let reward = (normalized_gap + normalized_stake + normalized_zeros).fixed_mul_floor(
        &env,
        &(BLOCK_REWARD + block.staked_total),
        &block.normalized_total.max(1),
    );

    println!("v2 {:?}", reward);
}

#[test]
fn test_zero_harvest() {
    let env = Env::default();

    println!("{:?}", BLOCK_REWARD);

    let pail = Pail {
        sequence: 0,
        gap: Some(0),
        stake: 0,
        zeros: Some(0),
    };

    let block = Block {
        entropy: BytesN::from_array(&env, &[0; 32]),
        max_gap: 46,
        max_stake: 1114640798,
        max_zeros: 9,
        min_gap: 0,
        min_stake: 0,
        min_zeros: 4,
        normalized_total: 1114640798 * 3 * 100,
        staked_total: 0,
        timestamp: 0,
    };

    let (normalized_gap, normalized_stake, normalized_zeros) = generate_normalizations_v1(
        &env,
        &block,
        pail.gap.unwrap(),
        pail.stake,
        pail.zeros.unwrap(),
    );

    let reward = (normalized_gap + normalized_stake + normalized_zeros).fixed_mul_floor(
        &env,
        &(BLOCK_REWARD + block.staked_total),
        &block.normalized_total.max(1),
    );

    println!("v1 {:?}", reward);

    let (normalized_gap, normalized_stake, normalized_zeros) = generate_normalizations_v2(
        &env,
        &block,
        pail.gap.unwrap(),
        pail.stake,
        pail.zeros.unwrap(),
    );

    let reward = (normalized_gap + normalized_stake + normalized_zeros).fixed_mul_floor(
        &env,
        &(BLOCK_REWARD + block.staked_total),
        &block.normalized_total.max(1),
    );

    println!("v2 {:?}", reward);
}

#[test]
fn test_misc() {
    let env = Env::default();

    let max_zero: i128 = 9;
    let min_zero: i128 = 5;
    let my_zero: i128 = 4;

    let range_zero = max_zero - min_zero;

    println!(
        "{:?}",
        100_0000000.fixed_mul_floor(&env, &(my_zero - min_zero), &range_zero)
    );

    let max_stake: i128 = 100_0000000;
    let min_stake: i128 = 1;
    let my_stake: i128 = 10_0000000;

    let range_stake = max_stake - min_stake;

    println!(
        "{:?}",
        100_0000000.fixed_mul_floor(&env, &(my_stake - min_stake), &range_stake)
    );

    let max_gap: i128 = 60;
    let min_gap: i128 = 1;
    let my_gap: i128 = 40;

    let range_gap = max_gap - min_gap;

    println!(
        "{:?}",
        100_0000000.fixed_mul_floor(&env, &(my_gap - min_gap), &range_gap)
    );
}

#[test]
fn test_address_lengths() {
    let env: Env = Env::default();

    let contract_address = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();

    let ed25519_keypair = Keypair::from_bytes(&[
        149, 154, 40, 132, 13, 234, 167, 87, 182, 44, 152, 45, 242, 179, 187, 17, 139, 106, 49, 85,
        249, 235, 17, 248, 24, 170, 19, 164, 23, 117, 145, 252, 172, 35, 170, 26, 69, 15, 75, 127,
        192, 170, 166, 54, 68, 127, 218, 29, 130, 173, 159, 1, 253, 192, 48, 242, 80, 12, 55, 152,
        223, 122, 198, 96,
    ])
    .unwrap();

    let ed25519_strkey =
        Strkey::PublicKeyEd25519(ed25519::PublicKey(ed25519_keypair.public.to_bytes()));
    let ed25519_address = Bytes::from_slice(&env, ed25519_strkey.to_string().as_bytes());
    let ed25519_address = Address::from_string_bytes(&ed25519_address);

    println!(
        "g-{:?} {:?}",
        ed25519_address.clone().to_xdr(&env).len(),
        ed25519_address.to_string()
    );
    println!(
        "c-{:?} {:?}",
        contract_address.clone().to_xdr(&env).len(),
        contract_address.to_string()
    );
}

#[test]
fn test_count_bytes() {
    let env = Env::default();

    let bytesn: BytesN<32> = BytesN::from_array(
        &env,
        &hex::decode("00000000075122504cde7b56f7a295ab5588a88eb74f87048d1c0ff9ec083bcb")
            .unwrap()
            .try_into()
            .unwrap(),
    );

    let mut count = 0;

    for byte in bytesn {
        if byte == 0 {
            count += 2;
        } else {
            // Use leading_zeros to count bits, convert to hex digits
            count += byte.leading_zeros() / 4;
            break;
        }
    }

    println!("{:?}", count);
}

#[test]
fn test_fixed_mul_floor() {
    let env = Env::default();

    let y: i128 = 500;
    let x: i128 = 100 * 10000000;
    let denominator: i128 = 1;

    let res = x.saturating_sub(x.fixed_div_floor(&env, &y, &denominator));

    println!("{:?}", res);
}

#[test]
fn test_integer_nth_root() {
    let y = 1_0000000;
    let n = 8;

    let res = integer_nth_root(y, n);

    println!("{:?}", res);
}

#[test]
fn test_unsafe_stuff() {
    let hash1: &[u8] =
        &hex::decode("00100000000f2ea462db5f7a04090430384433845c3367d55c06f20efd6656").unwrap();

    println!("{:?}", check_zeros(0, hash1));

    let hash2: &[u8] =
        &hex::decode("00000000000f2ea462db5f7a04090430384433845c3367d55c06f20efd6656").unwrap();

    println!("{:?}", check_zeros(2, hash2));

    // unsafe {
    //     let mut index = 0;
    //     let mut zeros = 0;

    //     loop {
    //         let more = hash.get_unchecked(index as usize).count_zeros();

    //         if more == 8 {
    //             zeros += 2;
    //         } else if more >= 4 {
    //             zeros += more / 4;
    //             break;
    //         } else {
    //             break;
    //         }

    //         index += 1;
    //     }

    //     println!("{:?} {:?}", index, zeros);
    // }
}

pub fn find_nonce_and_hash(
    env: &Env,
    index: &u32,
    entropy: &BytesN<32>,
    farmer: &Address,
    zeros: u32,
) -> (u64, BytesN<32>) {
    let mut nonce = 0;
    let mut hash_array = generate_hash(env, index, &nonce, entropy, farmer);

    // println!("{:?}", hash_array);

    loop {
        let hash = generate_keccak(&mut hash_array, &nonce);
        let mut leading_zeros = 0;

        for byte in hash {
            if byte == 0 {
                leading_zeros += 2;
            } else {
                // Use leading_zeros to count bits, convert to hex digits
                leading_zeros += byte.leading_zeros() / 4;
                break;
            }
        }

        if leading_zeros >= zeros {
            return (nonce, BytesN::from_array(env, &hash));
        }

        nonce += 1;
    }
}

fn generate_keccak(hash_b: &mut [u8; 76], nonce: &u64) -> [u8; 32] {
    let mut hash = [0u8; 32];

    hash_b[8..16].copy_from_slice(&nonce.to_be_bytes());

    let mut keccak = Keccak::v256();
    keccak.update(hash_b);
    keccak.finalize(&mut hash);

    hash
}

fn generate_hash(
    env: &Env,
    index: &u32,
    nonce: &u64,
    entropy: &BytesN<32>,
    farmer: &Address,
) -> [u8; 76] {
    let mut hash_array = [0u8; 76];

    let mut farmer_array = [0u8; 32];
    let farmer_bytes = farmer.to_xdr(env);
    farmer_bytes
        .slice(farmer_bytes.len() - 32..)
        .copy_into_slice(&mut farmer_array);

    hash_array[..4].copy_from_slice(&index.to_be_bytes());
    hash_array[4..12].copy_from_slice(&nonce.to_be_bytes());
    hash_array[12..44].copy_from_slice(&entropy.to_array());
    hash_array[44..].copy_from_slice(&farmer_array);

    hash_array
}

fn integer_nth_root(y: u64, n: u32) -> u64 {
    if y == 0 {
        return 0;
    }

    if y == 1 || n == 1 {
        return y;
    }

    let mut low = 1;
    let mut high = y;

    while low < high {
        let mid = (low + high) / 2;

        // Calculate mid^n using integer multiplication
        let mut power = 1u64;
        let mut overflow = false;

        for _ in 0..n {
            match power.checked_mul(mid) {
                Some(val) if val <= y => power = val,
                _ => {
                    overflow = true;
                    break;
                }
            }
        }

        if !overflow && power == y {
            return mid; // Exact match found
        } else if !overflow && power < y {
            low = mid + 1;
        } else {
            high = mid;
        }
    }

    low - 1
}

fn check_zeros(max_zeros: u32, hash: &[u8]) -> u32 {
    let mut zeros = max_zeros;
    let mut index = zeros / 2;

    loop {
        let more = unsafe { hash.get_unchecked(index as usize).leading_zeros() };

        if more == 8 {
            zeros += 2;
        } else {
            zeros += more / 4;
            break;
        }

        index += 1;
    }

    zeros
}

fn generate_normalizations_v1(
    env: &Env,
    block: &Block,
    gap: u32,
    stake: i128,
    zeros: u32,
) -> (i128, i128, i128) {
    // Calculate ranges
    let range_gap = (block.max_gap - block.min_gap).max(1) as i128;
    let range_stake = (block.max_stake - block.min_stake).max(1);
    let range_zeros = (block.max_zeros - block.min_zeros).max(1) as i128;

    // Find largest range for scaling
    let max_range = range_gap.max(range_stake).max(range_zeros);

    // Set minimum threshold (1% of max_range)
    let min_threshold = (max_range / 100).max(1);

    // Clamp inputs to valid ranges
    let clamped_gap = gap.max(block.min_gap).min(block.max_gap);
    let clamped_stake = stake.max(block.min_stake).min(block.max_stake);
    let clamped_zeros = zeros.max(block.min_zeros).min(block.max_zeros);

    // Scale each value relative to max_range
    let normalized_gap = ((clamped_gap - block.min_gap) as i128)
        .fixed_mul_floor(&env, &max_range, &range_gap)
        .max(min_threshold);
    let normalized_stake = ((clamped_stake - block.min_stake) as i128)
        .fixed_mul_floor(&env, &max_range, &range_stake)
        .max(min_threshold);
    let normalized_zeros = ((clamped_zeros - block.min_zeros) as i128)
        .fixed_mul_floor(&env, &max_range, &range_zeros)
        .max(min_threshold);

    println!(
        "{:?} {:?} {:?}",
        normalized_gap, normalized_stake, normalized_zeros
    );

    (normalized_gap, normalized_stake, normalized_zeros)
}

fn generate_normalizations_v2(
    env: &Env,
    block: &Block,
    gap: u32,
    stake: i128,
    zeros: u32,
) -> (i128, i128, i128) {
    // Calculate ranges
    let range_gap = (block.max_gap - block.min_gap).max(1) as i128;
    let range_stake = (block.max_stake - block.min_stake).max(1);
    let range_zeros = (block.max_zeros - block.min_zeros).max(1) as i128;

    // Find largest range for scaling
    let normalization_scale = range_gap.max(range_stake).max(range_zeros);

    // Clamp each value within its range.
    let clamped_gap = gap.max(block.min_gap).min(block.max_gap);
    let clamped_stake = stake.max(block.min_stake).min(block.max_stake);
    let clamped_zeros = zeros.max(block.min_zeros).min(block.max_zeros);

    // Normalize each value by subtracting the minimum and scaling relative to the range size.
    let normalized_gap = ((clamped_gap - block.min_gap) as i128)
        .fixed_mul_floor(&env, &normalization_scale, &range_gap)
        .max(1);
    let normalized_stake = (clamped_stake - block.min_stake)
        .fixed_mul_floor(&env, &normalization_scale, &range_stake)
        .max(1);
    let normalized_zeros = ((clamped_zeros - block.min_zeros) as i128)
        .fixed_mul_floor(&env, &normalization_scale, &range_zeros)
        .max(1);

    (normalized_gap, normalized_stake, normalized_zeros)
}
