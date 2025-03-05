use soroban_sdk::{panic_with_error, Address, Env};

use crate::{
    errors::Errors,
    types::{Block, Pail, Storage},
    WEEK_OF_LEDGERS,
};

pub fn extend_instance_ttl(env: &Env) {
    let max_ttl = env.storage().max_ttl();

    env.storage()
        .instance()
        .extend_ttl(max_ttl - WEEK_OF_LEDGERS, max_ttl);
}

pub fn has_farm_homesteader(env: &Env) -> bool {
    env.storage()
        .instance()
        .has::<Storage>(&Storage::Homesteader)
}
pub fn get_farm_homesteader(env: &Env) -> Address {
    env.storage()
        .instance()
        .get::<Storage, Address>(&Storage::Homesteader)
        .unwrap_or_else(|| panic_with_error!(&env, &Errors::HomesteadMissing))
}
pub fn set_farm_homesteader(env: &Env, homesteader: &Address) {
    env.storage()
        .instance()
        .set::<Storage, Address>(&Storage::Homesteader, homesteader);
}

pub fn get_farm_asset(env: &Env) -> Address {
    env.storage()
        .instance()
        .get::<Storage, Address>(&Storage::HomesteadAsset)
        .unwrap_or_else(|| panic_with_error!(&env, &Errors::HomesteadMissing))
}
pub fn set_farm_asset(env: &Env, asset: &Address) {
    env.storage()
        .instance()
        .set::<Storage, Address>(&Storage::HomesteadAsset, asset);
}

pub fn get_farm_index(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get::<Storage, u32>(&Storage::FarmIndex)
        .unwrap_or(0)
}
pub fn bump_farm_index(env: &Env, current_farm_index: &mut u32) {
    *current_farm_index += 1;

    env.storage()
        .instance()
        .set::<Storage, u32>(&Storage::FarmIndex, &current_farm_index);
}

pub fn get_farm_block(env: &Env) -> Option<Block> {
    env.storage()
        .instance()
        .get::<Storage, Block>(&Storage::FarmBlock)
}
pub fn set_farm_block(env: &Env, block: &Block) {
    env.storage()
        .instance()
        .set::<Storage, Block>(&Storage::FarmBlock, block);
}

pub fn get_farm_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get::<Storage, bool>(&Storage::FarmPaused)
        .unwrap_or(false)
}
pub fn set_farm_paused(env: &Env, paused: bool) {
    env.storage()
        .instance()
        .set::<Storage, bool>(&Storage::FarmPaused, &paused);
}

pub fn get_block(env: &Env, index: u32) -> Option<Block> {
    env.storage()
        .temporary()
        .get::<Storage, Block>(&Storage::Block(index))
}
pub fn set_block(env: &Env, index: u32, block: &Block) {
    env.storage()
        .temporary()
        .set::<Storage, Block>(&Storage::Block(index), block);
}

pub fn has_pail(env: &Env, farmer: Address, index: u32) -> bool {
    let pail_key = Storage::Pail(farmer, index);

    env.storage().temporary().has::<Storage>(&pail_key)
}
pub fn get_pail(env: &Env, farmer: Address, index: u32) -> Option<Pail> {
    let pail_key = Storage::Pail(farmer, index);

    env.storage().temporary().get::<Storage, Pail>(&pail_key)
}
pub fn set_pail(env: &Env, farmer: Address, index: u32, pail: Pail) {
    let pail_key = Storage::Pail(farmer, index);

    env.storage()
        .temporary()
        .set::<Storage, Pail>(&pail_key, &pail);
}
pub fn remove_pail(env: &Env, farmer: Address, index: u32) {
    let pail_key = Storage::Pail(farmer, index);

    env.storage().temporary().remove::<Storage>(&pail_key);
}
