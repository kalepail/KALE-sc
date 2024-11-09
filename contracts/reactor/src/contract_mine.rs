use soroban_sdk::{contractimpl, panic_with_error, token, Address, BytesN, Env};

use crate::{
    errors::Errors,
    storage::{extend_instance_ttl, get_mine, has_mine, set_mine},
    types::Mine,
    MineContractTrait, MineKalepailContract, MineKalepailContractClient,
};

#[contractimpl]
impl MineContractTrait for MineKalepailContract {
    fn discover_mine(env: Env, admin: Address, asset: Address) {
        admin.require_auth();

        if has_mine(&env) {
            panic_with_error!(&env, &Errors::AlreadyDiscovered);
        }

        if token::StellarAssetClient::new(&env, &asset).admin() != env.current_contract_address() {
            panic_with_error!(&env, &Errors::AssetAdminMismatch);
        }

        let mine = Mine {
            index: 0,
            admin,
            asset,
            paused: false,
        };

        set_mine(&env, &mine);

        extend_instance_ttl(&env);
    }

    fn upgrade_mine(env: Env, hash: BytesN<32>) {
        let mine = get_mine(&env).unwrap_or_else(|| panic_with_error!(&env, &Errors::MineNotFound));

        mine.admin.require_auth();

        env.deployer().update_current_contract_wasm(hash);

        extend_instance_ttl(&env);
    }

    fn pause_mine(env: Env) {
        let mut mine =
            get_mine(&env).unwrap_or_else(|| panic_with_error!(&env, &Errors::MineNotFound));

        if mine.paused {
            panic_with_error!(&env, &Errors::MineIsPaused);
        }

        mine.admin.require_auth();

        mine.paused = true;

        set_mine(&env, &mine);
    }

    fn unpause_mine(env: Env) {
        let mut mine =
            get_mine(&env).unwrap_or_else(|| panic_with_error!(&env, &Errors::MineNotFound));

        mine.admin.require_auth();

        mine.paused = false;

        set_mine(&env, &mine);

        extend_instance_ttl(&env);
    }
}