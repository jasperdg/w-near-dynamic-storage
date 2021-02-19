use super::*;
use near_sdk::json_types::{ValidAccountId, U128};

/// Price per 1 byte of storage from mainnet config after `0.18` release and protocol version `42`.
/// It's 10 times lower than the genesis price.
const STORAGE_PRICE_PER_BYTE: Balance = 10_000_000_000_000_000_000;

pub(crate) fn assert_one_yocto() {
    assert_eq!(
        env::attached_deposit(),
        1,
        "Requires attached deposit of exactly 1 yoctoNEAR"
    )
}

pub trait StorageManager {
    fn storage_deposit(&mut self, account_id: Option<ValidAccountId>) -> WrappedAccountStorageBalance;

    fn storage_withdraw(&mut self, amount: U128) -> WrappedAccountStorageBalance;

    fn storage_minimum_balance(&self) -> U128;

    fn storage_balance_of(&self, account_id: ValidAccountId) -> WrappedAccountStorageBalance;
}

#[near_bindgen]
impl StorageManager for Contract {
    #[payable]
    fn storage_deposit(&mut self, account_id: Option<ValidAccountId>) -> WrappedAccountStorageBalance {
        let amount = env::attached_deposit();
        assert!(amount > 0, "Requires NEAR deposit");

        let account_id = account_id
            .map(|a| a.into())
            .unwrap_or_else(|| env::predecessor_account_id());

        let new_balances = match self.accounts.get(&account_id) {
            Some(balances) => AccountStorageBalance {
                total: balances.total + amount,
                available: balances.available + amount,
            }, 
            None => {
                assert!(amount >= u128::from(self.storage_minimum_balance()), "amount is less than minimum amount required");
                AccountStorageBalance {
                    total: amount,
                    available: amount - u128::from(self.storage_minimum_balance()),
                }
            }
        };
      
        self.accounts.insert(&account_id, &new_balances);

        new_balances.wrap()
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: U128) -> WrappedAccountStorageBalance {
        assert_one_yocto();

        let amount: Balance = amount.into();
        let account_id = env::predecessor_account_id();

        if let Some(balances) = self.accounts.get(&account_id) {
            assert!(balances.available >= amount, "Not enough available balance to withdraw");
            let new_balances = AccountStorageBalance {
                total: balances.total - amount,
                available: balances.available - amount
            };

            self.accounts.insert(&account_id, &new_balances);
            Promise::new(account_id).transfer(amount + 1);

            new_balances.wrap()
        } else {
            env::panic(b"The account is not registered");
        }
    }

    fn storage_minimum_balance(&self) -> U128 {
        (Balance::from(self.account_storage_usage) * STORAGE_PRICE_PER_BYTE).into()
    }

    fn storage_balance_of(&self, account_id: ValidAccountId) -> WrappedAccountStorageBalance {
        if let Some(balances) = self.accounts.get(account_id.as_ref()) {
            balances.wrap()
        } else {
            WrappedAccountStorageBalance {
                total: 0.into(),
                available: 0.into(),
            }
        }
    }
}

