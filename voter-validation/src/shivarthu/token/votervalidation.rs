use super::super::{FungibleToken, Voter};
use near_sdk::collections::LookupMap;
use near_sdk::{env, near_bindgen, AccountId};

/// Voter Validation impl
#[near_bindgen]
impl FungibleToken {
    pub fn get_user_id(&self, account_id: &AccountId) -> u128 {
        let user_id_option = self.user_map.get(&account_id);
        match user_id_option {
            Some(user_id) => user_id,
            None => {
                panic!("User id doesnot exist for AccountId");
            }
        }
    }

    pub fn get_voter_details(&self, user_id: u128) -> Voter {
        let voter_profile_option = self.voter_profile_map.get(&user_id);
        let voter = voter_profile_option.unwrap();
        voter
    }

    pub fn get_voter_stake(&self, user_id: u128) -> u128 {
        let voter_stake_option = self.voter_stakes.get(&user_id);
        let voter_stake = voter_stake_option.unwrap();
        voter_stake
    }

    pub fn create_voter_profile(&mut self, profile_hash: String) {
        let account_id = env::signer_account_id();
        let account_id_exists_option = self.user_map.get(&account_id);
        let u = Voter {
            profile_hash,
            kyc_done: false,
        };
        match account_id_exists_option {
            Some(_user_id) => panic!("Voter profile already exists"),
            None => {
                self.user_id += 1;
                self.user_map.insert(&account_id, &self.user_id);
                self.voter_profile_map.insert(&self.user_id, &u);
            }
        }
    }

    pub fn create_voter_stake(&mut self, stake: u128) {
        let account_id = env::signer_account_id();
        let account_id_exists_option = self.user_map.get(&account_id);
        match account_id_exists_option {
            Some(user_id) => {
                let if_staked_bool_option = self.voter_if_staked.get(&user_id);
                // Memo: Test setting voter_if_staked to false
                match if_staked_bool_option {
                    Some(if_staked_bool) => {
                        if !if_staked_bool {
                            self.burn(&account_id, stake);
                            self.voter_if_staked.insert(&user_id, &true);
                            self.voter_stakes.insert(&user_id, &stake);
                            println!("I am in voter_if_staked false ");
                        }
                    }
                    None => {
                        self.burn(&account_id, stake);
                        self.voter_if_staked.insert(&user_id, &true);
                        self.voter_stakes.insert(&user_id, &stake);
                        println!("I am in voter_if_staked None");
                    }
                }
            }
            None => {
                panic!("User id doesnot exist");
            }
        }
    }

    /// Apply Jurors with stake

    pub fn apply_jurors(&mut self, voter_username: AccountId, stake: u128) {
        let account_id = env::signer_account_id();
        let singer_juror_user = self.get_user_id(&account_id);
        let voter_user_id = self.get_user_id(&voter_username);
        let user_juror_stakes_option = self.user_juror_stakes.get(&voter_user_id);
        match user_juror_stakes_option {
            Some(mut stake_entries) => {
                let stake_entries_option = stake_entries.get(&singer_juror_user);
                match stake_entries_option {
                    Some(stake) => {
                        if stake > 0 {
                            panic!("You have already staked")
                        } else {
                            stake_entries.insert(&singer_juror_user, &stake);
                            self.burn(&account_id, stake);
                            self.user_juror_stakes
                                .insert(&voter_user_id, &stake_entries);
                        }
                    }
                    None => {
                        stake_entries.insert(&singer_juror_user, &stake);
                        self.burn(&account_id, stake);
                        self.user_juror_stakes
                            .insert(&voter_user_id, &stake_entries);
                    }
                }
            }
            None => {
                let stakeidstring = format!(
                    "stakevoterid{}uniqueid{}",
                    voter_user_id, self.juror_stake_unique_id
                );
                let stakeid = stakeidstring.to_string().into_bytes();
                let mut stake_entries = LookupMap::new(stakeid);
                stake_entries.insert(&singer_juror_user, &stake);
                self.burn(&account_id, stake);
                self.user_juror_stakes
                    .insert(&voter_user_id, &stake_entries);
            }
        }
    }


    pub fn get_juror_stakes(&self, voter_user_id: u128, juror_user_id: u128) -> u128 {
        let juror_list_option = self.user_juror_stakes.get(&voter_user_id);
        match juror_list_option {
            Some(juror_list) => {
                let juror_stake = juror_list.get(&juror_user_id).unwrap();
                juror_stake
            }
            None => panic!("No one has staked for the voter"),
        }
    }
}

impl Default for FungibleToken {
    fn default() -> Self {
        panic!("Fun token should be initialized before usage")
    }
}
