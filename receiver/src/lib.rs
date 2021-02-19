/**
* wNear NEP-141 Token contract
*
* The aim of the contract is to enable the wrapping of the native NEAR token into a NEP-141 compatible token.
* It supports methods `near_deposit` and `near_withdraw` that wraps and unwraps NEAR tokens.
* They are effectively mint and burn underlying wNEAR tokens.
*
* lib.rs is the main entry point.
* fungible_token_core.rs implements NEP-146 standard
* storage_manager.rs implements NEP-145 standard for allocating storage per account
* fungible_token_metadata.rs implements NEP-148 standard for providing token-specific metadata.
* w_near.rs contains interfaces for depositing and withdrawing
* internal.rs contains internal methods for fungible token.
*/
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise, StorageUsage};
use near_sdk::serde::Serialize;

mod storage_manager;
const STORAGE_PRICE_PER_BYTE: Balance = 100_000_000_000_000_000_000;

#[global_allocator]
static ALLOC: near_sdk::wee_alloc::WeeAlloc<'_> = near_sdk::wee_alloc::WeeAlloc::INIT;

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct WrappedAccountStorageBalance {
    total: U128,
    available: U128
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountStorageBalance { 
    total: Balance,
    available: Balance
}

impl AccountStorageBalance {
    pub fn wrap(&self) -> WrappedAccountStorageBalance {
        WrappedAccountStorageBalance {
            total: self.total.into(),
            available: self.available.into()
        }
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// AccountID -> Account balance.
    pub accounts: LookupMap<AccountId, AccountStorageBalance>,

    /// Total supply of the all token.
    pub total_supply: Balance,

    /// The storage size in bytes for one account.
    pub account_storage_usage: StorageUsage,

    /// Dynamic storage map used for demo purposes
    pub essay: LookupMap<AccountId, String>
}

impl Default for Contract {
    fn default() -> Self {
        env::panic(b"Contract is not initialized");
    }
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let mut this = Self {
            accounts: LookupMap::new(b"a".to_vec()),
            total_supply: 0,
            account_storage_usage: 0,
            essay: LookupMap::new(b"b".to_vec()),
        };
        let initial_storage_usage = env::storage_usage();
        let tmp_account_id = unsafe { String::from_utf8_unchecked(vec![b'a'; 64]) };
        let tmp_account_balance = AccountStorageBalance {
            total: 0u128, 
            available: 0u128
        };
        this.accounts.insert(&tmp_account_id, &tmp_account_balance);
        this.account_storage_usage = env::storage_usage() - initial_storage_usage;
        this.accounts.remove(&tmp_account_id);
        this
    }

    pub fn on_transfer_call(
        &mut self, 
        sender: AccountId, 
        amount: U128, 
        msg: String, 
        _memo: Option<String>
    ) -> U128 {
        let amount: u128 = amount.into();
        assert_eq!(sender, "wnear.near".to_string(), "only accepts WNEAR as payment");
        assert_eq!(amount, 10_u128.pow(24), "it costs 1 WNEAR to write a new essay");
        let mut account_storage_balances = self.accounts.get(&sender).expect("sender is not registered with the contract");

        let pre_insert_storage_usage: u128 = env::storage_usage().into();
        
        self.essay.insert(&sender, &msg);

        let post_insert_storage_usage: u128 = env::storage_usage().into();

        if post_insert_storage_usage > pre_insert_storage_usage {
            let cost  = post_insert_storage_usage - pre_insert_storage_usage * STORAGE_PRICE_PER_BYTE;
            assert!(cost <= account_storage_balances.available, "err not enough storage balance for payment");
            account_storage_balances.available -= cost;
        } else if post_insert_storage_usage < pre_insert_storage_usage {
            let refund  = pre_insert_storage_usage - post_insert_storage_usage * STORAGE_PRICE_PER_BYTE;
            // Can refund (subtract from total and transfer) or add to available balance.account_storage_balances
            account_storage_balances.available += refund;
        }

        // Store updated storage balances
        self.accounts.insert(&sender, &account_storage_balances);

        return amount.into()
    }
}