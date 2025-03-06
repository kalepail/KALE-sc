#![no_std]

use soroban_sdk::{contract, Address, BytesN, Env};

mod contract_farm;
mod contract_homestead;
mod errors;
mod storage;
mod tests;
mod types;

// Higher BLOCK_INTERVAL means there's more time to mine a block which means we can have more participation without risking problematic congestion
// Too low and the network could be congested
// Too high and there's too much time for fast miners to dominate
pub const BLOCK_INTERVAL: u64 = 60 * 5; // 5 minutes, in seconds
pub const BLOCK_REWARD: i128 = 501_0000000 * BLOCK_INTERVAL as i128 / 60; // base_per_minute * second_interval / seconds_per_minute
pub const BLOCKS_PER_MONTH: u32 = 24 * 60 / 5 * 30; // 24 hours * 60 minutes / 5 minutes * 30 days
pub const BLOCK_SCALE: i128 = 100_0000000000; // 100% // 10 additional zeros because our max block reward is BLOCK_REWARD (10 digits)
pub const DECAY_RATE: i128 = 5_0000000000; // 5%
pub const INVERSE_DECAY_RATE: i128 = BLOCK_SCALE - DECAY_RATE; // 95%
pub const WEEK_OF_LEDGERS: u32 = 60 * 60 * 24 / 5 * 7; // assumes 5 second ledger close times
pub const V2_GENESIS_BLOCK: u32 = 30_558; // sequence 56031551

// TODO add more comments

#[contract]
pub struct Contract;

pub trait HomesteadTrait {
    fn __constructor(env: Env, farmer: Address, asset: Address);

    fn upgrade(env: Env, hash: BytesN<32>);

    fn pause(env: Env);

    fn unpause(env: Env);

    fn remove_block(env: Env, index: u32);
}

pub trait FarmTrait {
    fn plant(env: Env, farmer: Address, amount: i128);
    // TODO plant_v2 separate farmer and a signer source (might be a little tricky)

    fn work(env: Env, farmer: Address, hash: BytesN<32>, nonce: u64) -> u32;
    // TODO work_v2 don't require a hash, just a nonce

    fn harvest(env: Env, farmer: Address, index: u32) -> i128;
    // TODO maybe instead of a plant_v2 we just make a harvest_v2 and allow folks to harvest into another account
}
