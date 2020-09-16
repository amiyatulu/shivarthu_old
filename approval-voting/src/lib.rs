/*
 * Learn more about writing NEAR smart contracts with Rust:
 * https://github.com/near/near-sdk-rs
 *
 */

// To conserve gas, efficient serialization is achieved through Borsh (http://borsh.io/)
use chrono::{Duration, NaiveDateTime};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::wee_alloc;
use near_sdk::{env, near_bindgen};
use sha3::{Digest, Keccak256};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct CommitRevealElection {
    candidate_id: u128,
    candidate_map: LookupMap<u128, String>, // candidateId, candidateName
    candidate_votes: LookupMap<u128, u128>, //candiateId, candidateVotes
    commit_phase_end_time: String,
    number_of_votes_cast: u128,
    vote_commits: Vector<String>,
    vote_statuses: LookupMap<String, bool>,
}

impl Default for CommitRevealElection {
    fn default() -> Self {
        panic!("Please intialize the contract first")
    }
}

#[near_bindgen]
impl CommitRevealElection {
    #[init]
    pub fn new(commit_phase_length_in_secs: u128) -> Self {
        assert!(env::state_read::<Self>().is_none(), "Already initialized");
        if commit_phase_length_in_secs < 20 {
            panic!("Commit phase length can't be less than 20 secs");
        }
        let timestamp = env::block_timestamp();
        println!("{}, timestamp", timestamp);
        let naive = NaiveDateTime::from_timestamp(timestamp as i64, 0);
        println!("{}, now", naive);
        let seconds = Duration::seconds(commit_phase_length_in_secs as i64);
        let endtime = naive + seconds;
        println!("{}, time after addition", endtime);

        let commitreveal = Self {
            candidate_id: 0,
            candidate_map: LookupMap::new(b"4008c2a0-370c-4749-82ba-cc9a013c9416".to_vec()),
            candidate_votes: LookupMap::new(b"05943884-211d-4063-91c1-b802c2352fa4".to_vec()),
            commit_phase_end_time: endtime.to_string(),
            number_of_votes_cast: 0,
            vote_commits: Vector::new(b"a2e2c5d2-26e8-4eda-8b64-63786b9a1cad".to_vec()),
            vote_statuses: LookupMap::new(b"76234189-ba21-49d8-a77f-2c5f728891e0".to_vec()),
        };
        commitreveal
    }

    pub fn add_candidate(&mut self, name: String) {
        self.candidate_id += 1;
        self.candidate_map.insert(&self.candidate_id, &name);
    }

    pub fn commit_vote(&mut self, vote_commit: String) {
        let timestamp = env::block_timestamp();
        let naive_now = NaiveDateTime::from_timestamp(timestamp as i64, 0);
        println!("{}, now2", naive_now);
        let naive_end_time =
            NaiveDateTime::parse_from_str(&self.commit_phase_end_time, "%Y-%m-%d %H:%M:%S")
                .unwrap();
        println!("{}, naive_end_time", naive_end_time);
        if naive_now > naive_end_time {
            panic!("Commiting time has ended")
        }
        let votecommit = self.vote_statuses.get(&vote_commit);
        match votecommit {
            Some(_commit) => panic!("Vote commit is already done"),
            None => {
                println!("{} vote commit in commit fn", vote_commit);
                self.vote_commits.push(&vote_commit);
                self.vote_statuses.insert(&vote_commit, &true);
                self.number_of_votes_cast = self.number_of_votes_cast + 1;
            }
        }
    }

    pub fn reveal_vote(&mut self, vote: String, vote_commit: String) {
        let timestamp = env::block_timestamp();
        let naive_now = NaiveDateTime::from_timestamp(timestamp as i64, 0);
        let naive_end_time =
            NaiveDateTime::parse_from_str(&self.commit_phase_end_time, "%Y-%m-%d %H:%M:%S")
                .unwrap();
        if naive_now < naive_end_time {
            panic!("Commiting time has not ended");
        }
        println!("{} vote commit in reveal fn", vote_commit);
        let votecommit = self.vote_statuses.get(&vote_commit);
        match votecommit {
            Some(commit) => {
                if commit == false {
                    panic!("The vote was already casted");
                }
            }
            None => {
                panic!("Vote with this commit was not cast");
            }
        }
        let mut hasher = Keccak256::new();
        hasher.update(vote.as_bytes());
        let result = hasher.finalize();
        let vote_hex = format!("{:x}", result);
        println!("{} vote hex in reveal fn", vote_hex);
        if vote_commit == vote_hex {
            println!("commit and vote matches");
        }
        if vote_commit != vote_hex {
            panic!("Vote hash doesn't match the vote commit");
        }
        println!("{}", &vote[0..1]);
        let my_candidate_id_string = format!("{}", &vote[0..1]);
        match my_candidate_id_string.parse::<u128>() {
            Ok(n) => {
                if n > self.candidate_id {
                    panic!("Candidate id doesnot exist");
                } else {
                    let candidate_votes_option = self.candidate_votes.get(&n);
                    match candidate_votes_option {
                        Some(candidate_votes) => {
                            let votecount = candidate_votes + 1;
                            self.candidate_votes.insert(&n, &votecount);
                            println!("{} candidate got {} votes", n, votecount);
                        }
                        None => {
                            self.candidate_votes.insert(&n, &1);
                            println!("First vote");
                        }
                    }
                }
            }

            Err(e) => {
                panic!("{}", e);
            }
        }

        self.vote_statuses.insert(&vote_commit, &false);
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 *
 * To run from contract directory:
 * cargo test -- --nocapture
 *
 * From project root, to run in combination with frontend tests:
 * yarn test
 *
 */
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};
    use sha3::{Digest, Keccak256};
    use std::{thread, time};

    fn get_timstamp() -> u64 {
        let now: DateTime<Utc> = Utc::now();
        now.timestamp() as u64
    }

    // mock the context for testing, notice "signer_account_id" that was accessed above from env::
    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: get_timstamp(),
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 500,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn contract_test() {
        let mut context = get_context(vec![], false);
        testing_env!(context.clone());
        let mut contract = CommitRevealElection::new(20);
        let breaktime = time::Duration::from_secs(10);
        contract.add_candidate("Paul".to_owned());
        thread::sleep(breaktime);
        // Vote commit 1
        let vote = "1password".to_owned();
        let mut hasher = Keccak256::new();
        hasher.update(vote.as_bytes());
        let result = hasher.finalize();
        let commit = format!("{:x}", result);
        println!("{} commit in test", commit);
        context.block_timestamp = get_timstamp();
        testing_env!(context.clone());
        contract.commit_vote(commit.clone());

        // Vote commit 2
        let vote = "1mypass".to_owned();
        let mut hasher = Keccak256::new();
        hasher.update(vote.as_bytes());
        let result = hasher.finalize();
        let commit2 = format!("{:x}", result);
        println!("{} commit in test", commit);
        contract.commit_vote(commit2.clone());

        let breaktime2 = time::Duration::from_secs(15);
        thread::sleep(breaktime2);
        context.block_timestamp = get_timstamp();
        testing_env!(context.clone());
        // Vote reveal 1
        contract.reveal_vote("1password".to_owned(), commit.clone());
        // Vote reveal 2
        contract.reveal_vote("1mypass".to_owned(), commit2.clone());

    }
}
