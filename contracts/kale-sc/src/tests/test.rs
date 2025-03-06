use std::{print, println};
extern crate std;

use crate::{
    errors::Errors,
    tests::utils::find_nonce_and_hash,
    types::{Block, Storage},
    Contract, ContractClient, BLOCK_INTERVAL, BLOCK_REWARD,
};
use soroban_sdk::{
    testutils::{Address as _, BytesN as _, EnvTestConfig, Ledger},
    token, Address, BytesN, Env,
};

// TODO add more tests
// TODO write test utils

#[test]
fn test() {
    let mut env: Env = Env::default();

    env.set_config(EnvTestConfig {
        capture_snapshot_at_drop: false,
    });

    env.ledger().set_min_temp_entry_ttl(17280);
    env.ledger().set_min_persistent_entry_ttl(2073600);

    env.mock_all_auths();

    let sequence = env.ledger().sequence();
    let timestamp = env.ledger().timestamp();

    let homesteader: Address = Address::generate(&env);

    let asset_sac = env.register_stellar_asset_contract_v2(homesteader.clone());
    let asset_address = asset_sac.address();
    let asset_homesteader = token::StellarAssetClient::new(&env, &asset_address);
    let asset_client = token::Client::new(&env, &asset_address);

    let farm_address: Address = env.register(Contract, (&homesteader, &asset_address));
    let farm_client = ContractClient::new(&env, &farm_address);

    asset_homesteader.set_admin(&farm_address);

    let farmer_1: Address = Address::generate(&env);
    let farmer_2: Address = Address::generate(&env);
    let farmer_3: Address = Address::generate(&env);
    let farmer_4: Address = Address::generate(&env);

    let amount_1 = 1_0000000;
    let amount_2 = 0_0001000;
    let amount_3 = 0;
    let amount_4 = 0_1000000;

    asset_homesteader.mint(&farmer_1, &amount_1);
    asset_homesteader.mint(&farmer_2, &amount_2);
    asset_homesteader.mint(&farmer_3, &amount_3);
    asset_homesteader.mint(&farmer_4, &amount_4);

    farm_client.plant(&farmer_1, &amount_1);
    farm_client.plant(&farmer_2, &amount_2);
    farm_client.plant(&farmer_3, &amount_3);
    farm_client.plant(&farmer_4, &amount_4);

    let mut index: Option<u32> = None;
    let mut block: Option<Block> = None;

    env.as_contract(&farm_address, || {
        index = env
            .storage()
            .instance()
            .get::<Storage, u32>(&Storage::FarmIndex);
        block = env
            .storage()
            .temporary()
            .get::<Storage, Block>(&Storage::Block(index.unwrap_or(0)));
    });

    let index = index.unwrap_or(0);
    let block = block.unwrap();

    println!("{} {:?}", index, block);
    print!("\n");

    let (nonce_0, hash_0) = find_nonce_and_hash(&env, &index, &block.entropy, &farmer_1, 0);

    let (nonce_1, hash_1) = (
        26428569u64,
        BytesN::<32>::from_array(
            &env,
            &hex::decode("000000e9acb451f7830969e9aab5edb9c2f28c4e70096e723b75f303fbdcad11")
                .unwrap()
                .try_into()
                .unwrap(),
        ),
    );
    let (nonce_2, hash_2) = (
        36205151u64,
        BytesN::<32>::from_array(
            &env,
            &hex::decode("000000039a0708ea018d9158a784fba3bc338e07c84db4941d5b0e98daa7995b")
                .unwrap()
                .try_into()
                .unwrap(),
        ),
    );
    let (nonce_3, hash_3) = (
        1972700706u64,
        BytesN::<32>::from_array(
            &env,
            &hex::decode("0000000064de5fda472adf58a2c5f5946b5c4e66e57d8cbe3e9393afe4357a96")
                .unwrap()
                .try_into()
                .unwrap(),
        ),
    );
    let (nonce_4, hash_4) = (
        50600046851u64,
        BytesN::<32>::from_array(
            &env,
            &hex::decode("00000000013ccae976811077e2554ade4bebcbff7902acd99716a445e7ceb595")
                .unwrap()
                .try_into()
                .unwrap(),
        ),
    );

    env.ledger().set_sequence_number(sequence + 1);

    farm_client.work(&farmer_1, &hash_0, &nonce_0); // 0 zeros
    farm_client.work(&farmer_4, &hash_4, &nonce_4); // 9 zeros

    // Should not be able to update for a lower or equal zero count
    let err = farm_client
        .try_work(&farmer_1, &hash_0, &nonce_0)
        .unwrap_err()
        .unwrap();

    assert_eq!(err, Errors::ZeroCountTooLow.into());

    // Should be able to update for a higher zero count
    farm_client.work(&farmer_1, &hash_1, &nonce_1); // 6 zeros

    env.ledger().set_sequence_number(sequence + 20);

    farm_client.work(&farmer_2, &hash_2, &nonce_2); // 7 zeros
    farm_client.work(&farmer_3, &hash_3, &nonce_3); // 8 zeros

    env.as_contract(&farm_address, || {
        let block = env
            .storage()
            .temporary()
            .get::<Storage, Block>(&Storage::Block(index))
            .unwrap();

        println!("{} {:?}", index, block);
        print!("\n");
    });

    env.ledger().set_timestamp(timestamp + BLOCK_INTERVAL);

    farm_client.plant(&farmer_1, &0);

    farm_client.harvest(&farmer_1, &index);
    farm_client.harvest(&farmer_2, &index);
    farm_client.harvest(&farmer_3, &index);
    farm_client.harvest(&farmer_4, &index);

    println!(
        "farmer 1 profit: {:?}",
        asset_client.balance(&farmer_1) - amount_1
    );
    println!(
        "farmer 2 profit: {:?}",
        asset_client.balance(&farmer_2) - amount_2
    );
    println!(
        "farmer 3 profit: {:?}",
        asset_client.balance(&farmer_3) - amount_3
    );
    println!(
        "farmer 4 profit: {:?}",
        asset_client.balance(&farmer_4) - amount_4
    );
    print!("\n");

    env.as_contract(&farm_address, || {
        let index = env
            .storage()
            .instance()
            .get::<Storage, u32>(&Storage::FarmIndex)
            .unwrap();

        let block = env
            .storage()
            .temporary()
            .get::<Storage, Block>(&Storage::Block(index))
            .unwrap();

        println!("{} {:?}", index, block);
    });
}

#[test]
#[ignore = "set BLOCKS_PER_MONTH to 1, V2_FARM_INDEX to 0 and disable hash checking"]
fn test_decay() {
    let mut env: Env = Env::default();

    env.set_config(EnvTestConfig {
        capture_snapshot_at_drop: false,
    });

    env.ledger().set_min_temp_entry_ttl(17280);
    env.ledger().set_min_persistent_entry_ttl(2073600);

    env.mock_all_auths();

    let timestamp = env.ledger().timestamp();

    let homesteader: Address = Address::generate(&env);

    let asset_sac = env.register_stellar_asset_contract_v2(homesteader.clone());
    let asset_address = asset_sac.address();
    let asset_homesteader = token::StellarAssetClient::new(&env, &asset_address);

    let farm_address: Address = env.register(Contract, (&homesteader, &asset_address));
    let farm_client = ContractClient::new(&env, &farm_address);

    asset_homesteader.set_admin(&farm_address);

    let farmer_1: Address = Address::generate(&env);

    let iterations = 100;

    env.cost_estimate().budget().reset_unlimited();

    for i in 1..=iterations {
        env.ledger()
            .set_timestamp((timestamp + BLOCK_INTERVAL) * i as u64);
        farm_client.plant(&farmer_1, &0);
    }

    farm_client.work(&farmer_1, &BytesN::random(&env), &0);

    let mut index: u32 = 0;

    env.as_contract(&farm_address, || {
        index = env
            .storage()
            .instance()
            .get::<Storage, u32>(&Storage::FarmIndex)
            .unwrap();

        println!("FarmIndex {:?}", index);
    });

    env.ledger()
        .set_timestamp((timestamp + BLOCK_INTERVAL) * (iterations + 1) as u64);
    farm_client.plant(&farmer_1, &0);

    env.cost_estimate().budget().reset_unlimited();

    let reward = farm_client.harvest(&farmer_1, &index);

    // 100 iterations
    // InvocationResources {
    //     instructions: 364555,
    //     mem_bytes: 132273,
    //     read_entries: 5,
    //     write_entries: 2,
    //     read_bytes: 2012,
    //     write_bytes: 224,
    //     contract_events_size_bytes: 232,
    //     persistent_rent_ledger_bytes: 464486176,
    //     persistent_entry_rent_bumps: 1,
    //     temporary_rent_ledger_bytes: 0,
    //     temporary_entry_rent_bumps: 0,
    // }

    println!("{:#?}", env.cost_estimate().resources());

    println!("{:?}", BLOCK_REWARD);
    println!("{:?}", reward);
}
