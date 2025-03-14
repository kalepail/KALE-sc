use crate::{ContractArgs, BLOCKS_PER_MONTH, BLOCK_SCALE, INVERSE_DECAY_RATE, V2_GENESIS_BLOCK};
use soroban_fixed_point_math::SorobanFixedPoint;
use soroban_sdk::{contractimpl, panic_with_error, token, xdr::ToXdr, Address, Bytes, BytesN, Env};

use crate::{
    errors::Errors,
    storage::{
        bump_farm_index, extend_instance_ttl, get_block, get_farm_asset, get_farm_block,
        get_farm_index, get_farm_paused, get_pail, has_pail, remove_pail, set_block,
        set_farm_block, set_pail,
    },
    types::{Block, Pail},
    Contract, ContractClient, FarmTrait, BLOCK_INTERVAL, BLOCK_REWARD,
};

#[contractimpl]
impl FarmTrait for Contract {
    fn plant(env: Env, farmer: Address, amount: i128) {
        farmer.require_auth();

        if amount < 0 {
            panic_with_error!(&env, &Errors::PlantAmountTooLow);
        }

        if get_farm_paused(&env) {
            panic_with_error!(&env, &Errors::FarmPaused);
        }

        let asset = get_farm_asset(&env);
        let mut index = get_farm_index(&env);
        let mut farm_block = get_farm_block(&env)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::FarmBlockMissing));

        // NOTE originally we were calling `new_block` as the branch logic which read Block+0
        // but then in race txns 99+ we needed to read Block+1 but didn't have the read_bytes for that
        // If we include Block+0 as the branch logic it's LedgerKey would be included in the RW footprint which will consume read_bytes
        // Then in the 99+ when we try to read an additional Block+1 it will fail because we don't have the read_bytes for it
        // TODO Originally I noticed a variance of 460 and 240 read_bytes in the 99+ which I don't understand. 460 makes sense but 240 doesn't
        // 240 is the length of the Pail LedgerEntry so actually what was happening wasn't the Block it was the Pail read somehow?
        // I _think_ the only way 240 would make any sense as a deficit would be in the case of duplicate `plant` calls for the same farmer

        // if the block is >= BLOCK_INTERVAL old, we need to create a new one
        let mut block = if env.ledger().timestamp() >= farm_block.timestamp + BLOCK_INTERVAL {
            let block = new_block(&env, &farm_block);

            // call `get_block` on the previous block so we've got the necessary read_bytes for the N+ transactions which would otherwise be a `Block` short
            // e.g. 100 tx simulate this branch but only 1 actually executes it and the rest go to the else branch
            // this will give us a bonus budget of 460 read_bytes which the 99+ transactions can use to read the block this simulation didn't need to read
            get_block(&env, index);

            // ensure we put this after the `new_block` above
            farm_block = new_farm_block(&env);
            bump_farm_index(&env, &mut index);

            block
        } else {
            match get_block(&env, index) {
                // genesis or evicted
                None => {
                    if index > 0 {
                        // Only when we're in an evicted scenario should the index be bumped
                        bump_farm_index(&env, &mut index);
                    }

                    new_block(&env, &farm_block)
                }
                Some(block) => block,
            }
        };

        // must come after block discovery as the index may have been bumped
        if has_pail(&env, farmer.clone(), index) {
            panic_with_error!(&env, &Errors::PailExists);
        }

        block.staked_total += amount;

        if amount > 0 {
            token::Client::new(&env, &asset).burn(&farmer, &amount);
        }

        if amount > farm_block.max_stake {
            farm_block.max_stake = amount;
        }

        if amount < farm_block.min_stake {
            farm_block.min_stake = amount;
        }

        let pail = Pail {
            sequence: env.ledger().sequence(),
            gap: None,
            stake: amount,
            zeros: None,
        };

        set_pail(&env, farmer, index, pail);
        set_block(&env, index, &block);
        set_farm_block(&env, &farm_block);

        extend_instance_ttl(&env);
    }

    fn work(env: Env, farmer: Address, hash: BytesN<32>, nonce: u64) -> u32 {
        // No auth_require here so others can call this function on the `farmer`'s behalf

        let index = get_farm_index(&env);
        let mut farm_block = get_farm_block(&env)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::FarmBlockMissing));
        let mut block = get_block(&env, index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::BlockMissing));
        let mut pail = get_pail(&env, farmer.clone(), index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::PailMissing));

        let generated_hash = generate_hash(&env, &index, &nonce, &block.entropy, &farmer);
        let sequence = env.ledger().sequence();
        let gap = sequence - pail.sequence;
        let mut zeros = 0;

        // Ensure there's at least 1 ledger gap between plant and work (sorry RowBear, you're a genius)
        if gap == 0 {
            panic_with_error!(env, &Errors::GapCountTooLow);
        }

        // TODO No real reason to check if the hash is valid. If it's not the zero count would just be low or nil which is fine
        if hash != generated_hash {
            panic_with_error!(&env, &Errors::HashInvalid);
        }

        for byte in hash {
            if byte == 0 {
                zeros += 2;
            } else {
                zeros += byte.leading_zeros() / 4;
                break;
            }
        }

        // TODO save per farmer normalizations to their Pail so we don't have to recalculate during harvest
        // Would allow us to upgrade the normalizations logic without needing to toss the active block

        let (normalized_gap, normalized_stake, normalized_zeros) =
            generate_normalizations(&env, &block, gap, pail.stake, zeros);

        block.normalized_total += normalized_gap + normalized_stake + normalized_zeros;

        match pail.zeros {
            Some(prev_zeros) => {
                if zeros <= prev_zeros {
                    panic_with_error!(&env, &Errors::ZeroCountTooLow);
                }

                let (prev_normalized_gap, prev_normalized_stake, prev_normalized_zeros) =
                    generate_normalizations(&env, &block, gap, pail.stake, prev_zeros);

                block.normalized_total -=
                    prev_normalized_gap + prev_normalized_stake + prev_normalized_zeros;
            }
            None => {
                // Reclaim the stake from the work step
                block.staked_total -= pail.stake;
            }
        }

        farm_block.entropy = generated_hash;

        if gap > farm_block.max_gap {
            farm_block.max_gap = gap;
        }

        if gap < farm_block.min_gap {
            farm_block.min_gap = gap;
        }

        if zeros > farm_block.max_zeros {
            farm_block.max_zeros = zeros;
        }

        if zeros < farm_block.min_zeros {
            farm_block.min_zeros = zeros;
        }

        pail.gap = Some(gap);
        pail.zeros = Some(zeros);

        set_pail(&env, farmer, index, pail);
        set_block(&env, index, &block);
        set_farm_block(&env, &farm_block);

        extend_instance_ttl(&env);

        gap
    }

    fn harvest(env: Env, farmer: Address, index: u32) -> i128 {
        let asset = get_farm_asset(&env);
        let farm_index = get_farm_index(&env);
        let block = get_block(&env, index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::BlockMissing));
        let Pail {
            gap, stake, zeros, ..
        } = get_pail(&env, farmer.clone(), index)
            .unwrap_or_else(|| panic_with_error!(&env, &Errors::PailMissing));

        if index >= farm_index {
            panic_with_error!(&env, &Errors::HarvestNotReady);
        }

        if gap.is_none() || zeros.is_none() {
            panic_with_error!(&env, &Errors::WorkMissing);
        }

        let gap = gap.unwrap();
        let zeros = zeros.unwrap();

        let (normalized_gap, normalized_stake, normalized_zeros) =
            generate_normalizations(&env, &block, gap, stake, zeros);

        // Calculate the decayed block reward
        // Calculated dynamically per harvest (vs in the instance) as each block may have its own reward depending on when a user harvests
        let block_reward = calculate_block_reward(&env, index);

        let reward = (normalized_gap + normalized_stake + normalized_zeros).fixed_mul_floor(
            &env,
            &(block_reward + block.staked_total),
            &block.normalized_total.max(1),
        );
        let reward_and_stake = reward + stake;

        if reward_and_stake > 0 {
            token::StellarAssetClient::new(&env, &asset).mint(&farmer, &reward_and_stake);
        }

        remove_pail(&env, farmer.clone(), index);

        extend_instance_ttl(&env);

        reward
    }
}

pub fn new_farm_block(env: &Env) -> Block {
    Block {
        timestamp: env.ledger().timestamp(),
        min_gap: u32::MAX,
        min_stake: i128::MAX,
        min_zeros: u32::MAX,
        max_gap: u32::MIN,
        max_stake: i128::MIN,
        max_zeros: u32::MIN,
        entropy: BytesN::from_array(env, &[0; 32]),
        staked_total: 0,
        normalized_total: 0,
    }
}

fn new_block(env: &Env, farm_block: &Block) -> Block {
    // Autofill any non-default values with any current farm_block values we've got
    Block {
        timestamp: env.ledger().timestamp(),
        min_gap: if farm_block.min_gap == u32::MAX {
            0
        } else {
            farm_block.min_gap
        },
        min_stake: if farm_block.min_stake == i128::MAX {
            0
        } else {
            farm_block.min_stake
        },
        min_zeros: if farm_block.min_zeros == u32::MAX {
            0
        } else {
            farm_block.min_zeros
        },
        max_gap: if farm_block.max_gap == u32::MIN {
            0
        } else {
            farm_block.max_gap
        },
        max_stake: if farm_block.max_stake == i128::MIN {
            0
        } else {
            farm_block.max_stake
        },
        max_zeros: if farm_block.max_zeros == u32::MIN {
            0
        } else {
            farm_block.max_zeros
        },
        entropy: farm_block.entropy.clone(),
        staked_total: 0,
        normalized_total: 0,
    }
}

fn generate_hash(
    env: &Env,
    index: &u32,
    nonce: &u64,
    entropy: &BytesN<32>,
    farmer: &Address,
) -> BytesN<32> {
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

    env.crypto()
        .keccak256(&Bytes::from_array(env, &hash_array))
        .to_bytes()
}

fn generate_normalizations(
    env: &Env,
    block: &Block,
    gap: u32,
    stake: i128,
    zeros: u32,
) -> (i128, i128, i128) {
    // Prevent division by zero by ensuring max >= min for each range.
    // TODO should be impossible to hit (consider dropping)
    if block.max_gap < block.min_gap
        || block.max_stake < block.min_stake
        || block.max_zeros < block.min_zeros
    {
        panic_with_error!(&env, &Errors::BlockInvalid);
    }

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

fn calculate_block_reward(env: &Env, index: u32) -> i128 {
    let elapsed_time = index.saturating_sub(V2_GENESIS_BLOCK);
    let periods = elapsed_time.saturating_div(BLOCKS_PER_MONTH);

    let mut result = BLOCK_SCALE;

    for _ in 0..periods {
        result = result.fixed_mul_floor(env, &INVERSE_DECAY_RATE, &BLOCK_SCALE);
    }

    BLOCK_REWARD.fixed_mul_floor(env, &result, &BLOCK_SCALE)
}
