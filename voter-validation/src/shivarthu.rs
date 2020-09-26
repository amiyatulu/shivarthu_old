use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, wee_alloc, AccountId, Balance, Promise, StorageUsage};

pub mod account;
pub use self::account::Account;
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Price per 1 byte of storage from mainnet genesis config.
pub const STORAGE_PRICE_PER_BYTE: Balance = 100000000000000000000;

/// Contains balance and allowances information for one account.
///

// **** Steps for voter validation ****
// 1) Voters apply their resume
// 2) Voters stake some amount of token, (if > 2 tokens will get 1 tokens, if less than 2 token, will get 50% of the token), will be quadratic in future https://bioinsilico.blogspot.com/2020/08/perfect-price-discovery-and-blockchain.html
// 3) Apply jurors using staking using the id of voter application, also set the time till jurors can apply.
// 4) Draw jurors, 50% of jouror can vote, can't be less than 10
// 5) Juror Vote, set the time for voting through commit
// 6) Reveal Juror vote
// 7) Juror will get the incentives or disinstives 5tokens/total jurors

#[derive(Debug, Default, BorshDeserialize, BorshSerialize)]
pub struct Voter {
    pub profile_hash: String, //IPFS Hash
    pub kyc_done: bool,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleToken {
    /// sha256(AccountID) -> Account details.
    accounts: UnorderedMap<Vec<u8>, Account>,

    /// Total supply of the all token.
    total_supply: Balance,

    // Voter validation
    voter_id: u128,
    voter_map: LookupMap<String, u128>, // <Account_name, voter_id>
    voter_profile_map: LookupMap<u128, Voter>, // <voter_id, Voter>
    voter_if_staked: LookupMap<u128, bool>, // <voter_id, true or false>
    voter_stakes: LookupMap<u128, u128>, // <voter_id, stakes>
}

/// Voter Validation impl
#[near_bindgen]
impl FungibleToken {
    pub fn get_voter_id(&self, account_id: AccountId) -> u128 {
        let voter_id_option = self.voter_map.get(&account_id);
        let voter_id = voter_id_option.unwrap();
        voter_id
    }

    pub fn get_voter_details(&self, voter_id: u128) -> Voter {
        let voter_profile_option = self.voter_profile_map.get(&voter_id);
        let voter = voter_profile_option.unwrap();
        voter
    }

    pub fn create_voter_profile(&mut self, profile_hash: String) {
        let account_id = env::signer_account_id();
        let account_id_exists_option = self.voter_map.get(&account_id);
        let u = Voter {
            profile_hash,
            kyc_done: false,
        };
        match account_id_exists_option {
            Some(_voter_id) => panic!("Voter profile already exists"),
            None => {
                self.voter_id += 1;
                self.voter_map.insert(&account_id, &self.voter_id);
                self.voter_profile_map.insert(&self.voter_id, &u);
            }
        }
    }

    pub fn create_voter_stake(&mut self, stake: u128) {
        let account_id = env::signer_account_id();
        let account_id_exists_option = self.voter_map.get(&account_id);
        match account_id_exists_option {
            Some(voter_id) => {
                let if_staked_bool_option = self.voter_if_staked.get(&voter_id);
                match if_staked_bool_option {
                    Some(if_staked_bool) => {
                        if !if_staked_bool {
                            self.burn(&account_id, stake);
                            self.voter_if_staked.insert(&voter_id, &true);
                            println!("I am in voter_if_staked false ");
                        }
                    }
                    None => {
                        self.burn(&account_id, stake);
                        self.voter_if_staked.insert(&voter_id, &true);
                        println!("I am in voter_if_staked None");
                    }
                }
            }
            None => {
                panic!("Voter id doesnot exist");
            }
        }
    }
}

impl Default for FungibleToken {
    fn default() -> Self {
        panic!("Fun token should be initialized before usage")
    }
}

/// Burn and mint
#[near_bindgen]
impl FungibleToken {
    fn _mint(&mut self, owner_id: &AccountId, amount: u128) {
        if !owner_id.is_empty() {
            let initial_storage = env::storage_usage();
            if amount == 0 {
                env::panic(b"Can't transfer 0 tokens");
            }
            assert!(
                env::is_valid_account_id(owner_id.as_bytes()),
                "New owner's account ID is invalid"
            );
            self.total_supply = self.total_supply + amount;
            let mut account = self.get_account(&owner_id);
            account.balance += amount;
            self.set_account(&owner_id, &account);
            self.refund_storage(initial_storage);
        }
    }

    fn burn(&mut self, owner_id: &AccountId, amount: u128) {
        if !owner_id.is_empty() {
            let initial_storage = env::storage_usage();
            if amount == 0 {
                env::panic(b"Can't transfer 0 tokens");
            }
            assert!(
                env::is_valid_account_id(owner_id.as_bytes()),
                "Owner's account ID is invalid"
            );
         
            self.total_supply = self.total_supply - amount;
            let mut account = self.get_account(&owner_id);

            account.balance -= amount;
            self.set_account(&owner_id, &account);
            self.refund_storage(initial_storage);
        }
    }
}

#[near_bindgen]
impl FungibleToken {
    /// Initializes the contract with the given total supply owned by the given `owner_id`.
    #[init]
    pub fn new(owner_id: AccountId, total_supply: U128) -> Self {
        let total_supply = total_supply.into();
        assert!(!env::state_exists(), "Already initialized");
        let mut ft = Self {
            accounts: UnorderedMap::new(b"a".to_vec()),
            total_supply,
            voter_id: 0,
            voter_map: LookupMap::new(b"2a543bc7-a03f-427f-98c4-aa34012fa358".to_vec()),
            voter_profile_map: LookupMap::new(b"a9d08e6d-fe16-441e-9330-81f45b8a68b3".to_vec()),
            voter_if_staked: LookupMap::new(b"0e9cdb00-e90a-4aed-8541-1fb2ea6a1538".to_vec()),
            voter_stakes: LookupMap::new(b"de89b05f-e35d-4237-bba9-64b2baac1ca8".to_vec()),
        };
        let mut account = ft.get_account(&owner_id);
        account.balance = total_supply;
        ft.set_account(&owner_id, &account);
        ft
    }

    /// Increments the `allowance` for `escrow_account_id` by `amount` on the account of the caller of this contract
    /// (`predecessor_id`) who is the balance owner.
    /// Requirements:
    /// * Caller of the method has to attach deposit enough to cover storage difference at the
    ///   fixed storage price defined in the contract.
    #[payable]
    pub fn inc_allowance(&mut self, escrow_account_id: AccountId, amount: U128) {
        let initial_storage = env::storage_usage();
        assert!(
            env::is_valid_account_id(escrow_account_id.as_bytes()),
            "Escrow account ID is invalid"
        );
        let owner_id = env::predecessor_account_id();
        if escrow_account_id == owner_id {
            env::panic(b"Can not increment allowance for yourself");
        }
        let mut account = self.get_account(&owner_id);
        let current_allowance = account.get_allowance(&escrow_account_id);
        account.set_allowance(
            &escrow_account_id,
            current_allowance.saturating_add(amount.0),
        );
        self.set_account(&owner_id, &account);
        self.refund_storage(initial_storage);
    }

    /// Decrements the `allowance` for `escrow_account_id` by `amount` on the account of the caller of this contract
    /// (`predecessor_id`) who is the balance owner.
    /// Requirements:
    /// * Caller of the method has to attach deposit enough to cover storage difference at the
    ///   fixed storage price defined in the contract.
    #[payable]
    pub fn dec_allowance(&mut self, escrow_account_id: AccountId, amount: U128) {
        let initial_storage = env::storage_usage();
        assert!(
            env::is_valid_account_id(escrow_account_id.as_bytes()),
            "Escrow account ID is invalid"
        );
        let owner_id = env::predecessor_account_id();
        if escrow_account_id == owner_id {
            env::panic(b"Can not decrement allowance for yourself");
        }
        let mut account = self.get_account(&owner_id);
        let current_allowance = account.get_allowance(&escrow_account_id);
        account.set_allowance(
            &escrow_account_id,
            current_allowance.saturating_sub(amount.0),
        );
        self.set_account(&owner_id, &account);
        self.refund_storage(initial_storage);
    }

    /// Transfers the `amount` of tokens from `owner_id` to the `new_owner_id`.
    /// Requirements:
    /// * `amount` should be a positive integer.
    /// * `owner_id` should have balance on the account greater or equal than the transfer `amount`.
    /// * If this function is called by an escrow account (`owner_id != predecessor_account_id`),
    ///   then the allowance of the caller of the function (`predecessor_account_id`) on
    ///   the account of `owner_id` should be greater or equal than the transfer `amount`.
    /// * Caller of the method has to attach deposit enough to cover storage difference at the
    ///   fixed storage price defined in the contract.
    #[payable]
    pub fn transfer_from(&mut self, owner_id: AccountId, new_owner_id: AccountId, amount: U128) {
        let initial_storage = env::storage_usage();
        assert!(
            env::is_valid_account_id(new_owner_id.as_bytes()),
            "New owner's account ID is invalid"
        );
        let amount = amount.into();
        if amount == 0 {
            env::panic(b"Can't transfer 0 tokens");
        }
        assert_ne!(
            owner_id, new_owner_id,
            "The new owner should be different from the current owner"
        );
        // Retrieving the account from the state.
        let mut account = self.get_account(&owner_id);

        // Checking and updating unlocked balance
        if account.balance < amount {
            env::panic(b"Not enough balance");
        }
        account.balance -= amount;

        // If transferring by escrow, need to check and update allowance.
        let escrow_account_id = env::predecessor_account_id();
        if escrow_account_id != owner_id {
            let allowance = account.get_allowance(&escrow_account_id);
            if allowance < amount {
                env::panic(b"Not enough allowance");
            }
            account.set_allowance(&escrow_account_id, allowance - amount);
        }

        // Saving the account back to the state.
        self.set_account(&owner_id, &account);

        // Deposit amount to the new owner and save the new account to the state.
        let mut new_account = self.get_account(&new_owner_id);
        new_account.balance += amount;
        self.set_account(&new_owner_id, &new_account);
        self.refund_storage(initial_storage);
    }

    /// Transfer `amount` of tokens from the caller of the contract (`predecessor_id`) to
    /// `new_owner_id`.
    /// Act the same was as `transfer_from` with `owner_id` equal to the caller of the contract
    /// (`predecessor_id`).
    /// Requirements:
    /// * Caller of the method has to attach deposit enough to cover storage difference at the
    ///   fixed storage price defined in the contract.
    #[payable]
    pub fn transfer(&mut self, new_owner_id: AccountId, amount: U128) {
        // NOTE: New owner's Account ID checked in transfer_from.
        // Storage fees are also refunded in transfer_from.
        self.transfer_from(env::predecessor_account_id(), new_owner_id, amount);
    }

    /// Returns total supply of tokens.
    pub fn get_total_supply(&self) -> U128 {
        self.total_supply.into()
    }

    /// Returns balance of the `owner_id` account.
    pub fn get_balance(&self, owner_id: AccountId) -> U128 {
        self.get_account(&owner_id).balance.into()
    }

    /// Returns current allowance of `escrow_account_id` for the account of `owner_id`.
    ///
    /// NOTE: Other contracts should not rely on this information, because by the moment a contract
    /// receives this information, the allowance may already be changed by the owner.
    /// So this method should only be used on the front-end to see the current allowance.
    pub fn get_allowance(&self, owner_id: AccountId, escrow_account_id: AccountId) -> U128 {
        assert!(
            env::is_valid_account_id(escrow_account_id.as_bytes()),
            "Escrow account ID is invalid"
        );
        self.get_account(&owner_id)
            .get_allowance(&escrow_account_id)
            .into()
    }
}

impl FungibleToken {
    /// Helper method to get the account details for `owner_id`.
    fn get_account(&self, owner_id: &AccountId) -> Account {
        assert!(
            env::is_valid_account_id(owner_id.as_bytes()),
            "Owner's account ID is invalid"
        );
        let account_hash = env::sha256(owner_id.as_bytes());
        self.accounts
            .get(&account_hash)
            .unwrap_or_else(|| Account::new(account_hash))
    }

    /// Helper method to set the account details for `owner_id` to the state.
    fn set_account(&mut self, owner_id: &AccountId, account: &Account) {
        let account_hash = env::sha256(owner_id.as_bytes());
        if account.balance > 0 || !account.allowances.is_empty() {
            self.accounts.insert(&account_hash, &account);
        } else {
            self.accounts.remove(&account_hash);
        }
    }

    fn refund_storage(&self, initial_storage: StorageUsage) {
        let current_storage = env::storage_usage();
        let attached_deposit = env::attached_deposit();
        let refund_amount = if current_storage > initial_storage {
            let required_deposit =
                Balance::from(current_storage - initial_storage) * STORAGE_PRICE_PER_BYTE;
            assert!(
                required_deposit <= attached_deposit,
                "The required attached deposit is {}, but the given attached deposit is is {}",
                required_deposit,
                attached_deposit,
            );
            attached_deposit - required_deposit
        } else {
            attached_deposit
                + Balance::from(initial_storage - current_storage) * STORAGE_PRICE_PER_BYTE
        };
        if refund_amount > 0 {
            env::log(format!("Refunding {} tokens for storage", refund_amount).as_bytes());
            Promise::new(env::predecessor_account_id()).transfer(refund_amount);
        }
    }
}
