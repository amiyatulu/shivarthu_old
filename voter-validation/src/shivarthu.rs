use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, TreeMap, Vector, LookupSet};
use near_sdk::{near_bindgen, wee_alloc, Balance};

mod account;
use self::account::Account;
mod token;
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

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SortitionSumTree {
    k: u128,
    stack: Vector<u128>,
    nodes: Vector<u128>,
    ids_to_node_indexes: TreeMap<String, u128>,
    node_indexes_to_ids: TreeMap<u128, String>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleToken {
    /// sha256(AccountID) -> Account details.
    accounts: UnorderedMap<Vec<u8>, Account>,

    /// Total supply of the all token.
    total_supply: Balance,

    // Voter validation
    user_id: u128,
    user_map: LookupMap<String, u128>, // <Account_name, user_id>
    voter_profile_map: LookupMap<u128, Voter>, // <user_id, Voter>
    voter_if_staked: LookupMap<u128, bool>, // <user_id, true or false>
    voter_stakes: LookupMap<u128, u128>, // <user_id, stakes>
    // juror_stakes: LookupMap<u128, LookupMap<u128, u128>>, //<juror user_id, <voter userid, stakes>>
    // juror_if_staked: LookupMap<u128, Vector<LookupMap<u128, u128>>>, // <juror user_id, <voter_user_id, true or false>>
    // juror_applied_for: LookupMap<u128, LookupSet<u128>>, //<juror user_id, voter user id set>
    user_juror_stakes: LookupMap<u128, LookupMap<u128, u128>>, // <voter_user_id, <jurorid, stakes>>
    user_juror_stakes_clone: LookupMap<u128, TreeMap<u128, u128>>,
    juror_stake_unique_id: u128,
    selected_juror: LookupMap<u128, LookupSet<u128>> // <voter_user_id, jurorid>
}

