use uint::construct_uint;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

use crate::account::*;
pub use crate::enumeration::PoolInfo;
pub use crate::account::AccountJson;
use crate::util::*;

mod account;
mod util;
mod internal;
mod core_impl;
mod enumeration;

pub const NO_DEPOSIT: Balance = 0;
pub const DEPOSIT_ONE_YOCTOR: Balance = 1;
pub const NUM_EPOCHS_TO_UNLOCK: EpochHeight = 1;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Copy)]
pub struct Config {
    // Percent reward per 1 block
    pub reward_numerator: u32,
    pub reward_denumerator: u64,
    pub total_apr: u32
}

impl Default for Config {
    fn default() -> Self {
        // By default APR 15%
        Self { reward_numerator: 715, reward_denumerator: 100000000000, total_apr: 15 }
    }
}

#[derive(BorshDeserialize, BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    AccountKey
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[near_bindgen]
pub struct TreasuryContract {
    pub owner_id: AccountId, // Owner of contract
    pub ft_contract_id: AccountId,
    pub config: Config, // Config reward and apr for contract
    pub total_stake_balance: Balance, // Total token balance lock in contract
    pub total_paid_reward_balance: Balance,
    pub total_staker: Balance,
    pub pre_reward: Balance, // Pre reward before change total balance
    pub last_block_balance_change: BlockHeight,
    pub accounts: LookupMap<AccountId, UpgradableAccount>, // List staking user
    pub paused: bool, // Pause staking pool with limit reward,
    pub paused_in_block: BlockHeight
}

#[near_bindgen]
impl TreasuryContract {

    #[init]
    pub fn new_default_config(owner_id: AccountId, ft_contract_id: AccountId) -> Self {
        Self::new(owner_id, ft_contract_id, Config::default())
    }

    #[init]
    pub fn new(owner_id: AccountId, ft_contract_id: AccountId, config: Config) -> Self {
        TreasuryContract {
            owner_id,
            ft_contract_id,
            config,
            total_stake_balance: 0,
            total_paid_reward_balance: 0,
            total_staker: 0,
            pre_reward: 0,
            last_block_balance_change: env::block_index(),
            accounts: LookupMap::new(StorageKey::AccountKey),
            paused: false,
            paused_in_block: 0
        }
    }

    pub fn get_total_pending_reward(&self) -> U128 {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "ERR_ONLY_OWNER_CONTRACT");
        U128(self.pre_reward + self.internal_calculate_global_reward())
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    #[payable]
    pub fn storage_deposit(&mut self, account_id: Option<AccountId>) {

    }

    // View func get storage balance, return 0 if account need deposit to interact
    pub fn storage_balance_of(&self, account_id: AccountId) -> U128 {
        let account: Option<UpgradableAccount> = self.accounts.get(&account_id);
        if account.is_some() {
            U128(1)
        } else {
            U128(0)
        }
    }

    pub(crate) fn assert_owner(&self) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Only owner contract can be access");
    }

    #[init(ignore_state)]
    #[private]
    pub fn migrate() -> Self {
        let contract: TreasuryContract = env::state_read().expect("ERR_READ_CONTRACT_STATE");
        contract
    }
}
