pub mod shivarthu;
#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use crate::shivarthu::{FungibleToken, STORAGE_PRICE_PER_BYTE};
    use near_sdk::MockedBlockchain;
    use near_sdk::{env, AccountId, Balance};
    use near_sdk::{testing_env, VMContext};

    fn alice() -> AccountId {
        "alice.near".to_string()
    }
    fn bob() -> AccountId {
        "bob.near".to_string()
    }
    fn carol() -> AccountId {
        "carol.near".to_string()
    }

    fn get_context(predecessor_account_id: AccountId) -> VMContext {
        VMContext {
            current_account_id: alice(), //The id of the account that owns the current contract.
            signer_account_id: bob(), // The id of the account that either signed the original transaction or issued the initial cross-contract call
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id, //The id of the account that was the previous contract in the chain of cross-contract calls. If this is the first contract, it is equal to signer_account_id
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 1_000_000_000_000_000_000_000_000_000u128,
            account_locked_balance: 0,
            storage_usage: 10u64.pow(6),
            attached_deposit: 0, //The balance that was attached to the call that will be immediately deposited before the contract execution starts
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    #[test]
    fn test_initialize_new_token() {
        let context = get_context(carol());
        testing_env!(context);
        let total_supply = 1_000_000_000_000_000u128;
        let contract = FungibleToken::new(bob(), total_supply.into());
        assert_eq!(contract.get_total_supply().0, total_supply);
        assert_eq!(contract.get_balance(bob()).0, total_supply);
    }

    #[test]
    #[should_panic]
    fn test_initialize_new_token_twice_fails() {
        let context = get_context(carol());
        testing_env!(context);
        let total_supply = 1_000_000_000_000_000u128;
        {
            let _contract = FungibleToken::new(bob(), total_supply.into());
        }
        FungibleToken::new(bob(), total_supply.into());
    }

    #[test]
    fn test_transfer_to_a_different_account_works() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = FungibleToken::new(carol(), total_supply.into());
        context.storage_usage = env::storage_usage();

        context.attached_deposit = 1000 * STORAGE_PRICE_PER_BYTE;
        testing_env!(context.clone());
        let transfer_amount = total_supply / 3;
        contract.transfer(bob(), transfer_amount.into());
        context.storage_usage = env::storage_usage();
        context.account_balance = env::account_balance();

        context.is_view = true;
        context.attached_deposit = 0;
        testing_env!(context.clone());
        assert_eq!(
            contract.get_balance(carol()).0,
            (total_supply - transfer_amount)
        );
        assert_eq!(contract.get_balance(bob()).0, transfer_amount);
    }

    #[test]
    #[should_panic(expected = "The new owner should be different from the current owner")]
    fn test_transfer_to_self_fails() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = FungibleToken::new(carol(), total_supply.into());
        context.storage_usage = env::storage_usage();

        context.attached_deposit = 1000 * STORAGE_PRICE_PER_BYTE;
        testing_env!(context.clone());
        let transfer_amount = total_supply / 3;
        contract.transfer(carol(), transfer_amount.into());
    }

    #[test]
    #[should_panic(expected = "Can not increment allowance for yourself")]
    fn test_increment_allowance_to_self_fails() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = FungibleToken::new(carol(), total_supply.into());
        context.attached_deposit = STORAGE_PRICE_PER_BYTE * 1000;
        testing_env!(context.clone());
        contract.inc_allowance(carol(), (total_supply / 2).into());
    }

    #[test]
    #[should_panic(expected = "Can not decrement allowance for yourself")]
    fn test_decrement_allowance_to_self_fails() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = FungibleToken::new(carol(), total_supply.into());
        context.attached_deposit = STORAGE_PRICE_PER_BYTE * 1000;
        testing_env!(context.clone());
        contract.dec_allowance(carol(), (total_supply / 2).into());
    }

    #[test]
    fn test_decrement_allowance_after_allowance_was_saturated() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = FungibleToken::new(carol(), total_supply.into());
        context.attached_deposit = STORAGE_PRICE_PER_BYTE * 1000;
        testing_env!(context.clone());
        contract.dec_allowance(bob(), (total_supply / 2).into());
        assert_eq!(contract.get_allowance(carol(), bob()), 0.into())
    }

    #[test]
    fn test_increment_allowance_does_not_overflow() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = std::u128::MAX;
        let mut contract = FungibleToken::new(carol(), total_supply.into());
        context.attached_deposit = STORAGE_PRICE_PER_BYTE * 1000;
        testing_env!(context.clone());
        contract.inc_allowance(bob(), total_supply.into());
        contract.inc_allowance(bob(), total_supply.into());
        assert_eq!(
            contract.get_allowance(carol(), bob()),
            std::u128::MAX.into()
        )
    }

    #[test]
    #[should_panic(
        expected = "The required attached deposit is 33100000000000000000000, but the given attached deposit is is 0"
    )]
    fn test_increment_allowance_with_insufficient_attached_deposit() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = FungibleToken::new(carol(), total_supply.into());
        context.attached_deposit = 0;
        testing_env!(context.clone());
        contract.inc_allowance(bob(), (total_supply / 2).into());
    }

    #[test]
    fn test_carol_escrows_to_bob_transfers_to_alice() {
        // Acting as carol
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = FungibleToken::new(carol(), total_supply.into());
        context.storage_usage = env::storage_usage();

        context.is_view = true;
        testing_env!(context.clone());
        assert_eq!(contract.get_total_supply().0, total_supply);

        let allowance = total_supply / 3;
        let transfer_amount = allowance / 3;
        context.is_view = false;
        context.attached_deposit = STORAGE_PRICE_PER_BYTE * 1000;
        testing_env!(context.clone());
        contract.inc_allowance(bob(), allowance.into());
        context.storage_usage = env::storage_usage();
        context.account_balance = env::account_balance();

        context.is_view = true;
        context.attached_deposit = 0;
        testing_env!(context.clone());
        assert_eq!(contract.get_allowance(carol(), bob()).0, allowance);

        // Acting as bob now
        context.is_view = false;
        context.attached_deposit = STORAGE_PRICE_PER_BYTE * 1000;
        context.predecessor_account_id = bob();
        testing_env!(context.clone());
        contract.transfer_from(carol(), alice(), transfer_amount.into());
        context.storage_usage = env::storage_usage();
        context.account_balance = env::account_balance();

        context.is_view = true;
        context.attached_deposit = 0;
        testing_env!(context.clone());
        assert_eq!(
            contract.get_balance(carol()).0,
            total_supply - transfer_amount
        );
        assert_eq!(contract.get_balance(alice()).0, transfer_amount);
        assert_eq!(
            contract.get_allowance(carol(), bob()).0,
            allowance - transfer_amount
        );
    }

    #[test]
    fn test_carol_escrows_to_bob_locks_and_transfers_to_alice() {
        // Acting as carol
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = FungibleToken::new(carol(), total_supply.into());
        context.storage_usage = env::storage_usage();

        context.is_view = true;
        testing_env!(context.clone());
        assert_eq!(contract.get_total_supply().0, total_supply);

        let allowance = total_supply / 3;
        let transfer_amount = allowance / 3;
        context.is_view = false;
        context.attached_deposit = STORAGE_PRICE_PER_BYTE * 1000;
        testing_env!(context.clone());
        contract.inc_allowance(bob(), allowance.into());
        context.storage_usage = env::storage_usage();
        context.account_balance = env::account_balance();

        context.is_view = true;
        context.attached_deposit = 0;
        testing_env!(context.clone());
        assert_eq!(contract.get_allowance(carol(), bob()).0, allowance);
        assert_eq!(contract.get_balance(carol()).0, total_supply);

        // Acting as bob now
        context.is_view = false;
        context.attached_deposit = STORAGE_PRICE_PER_BYTE * 1000;
        context.predecessor_account_id = bob();
        testing_env!(context.clone());
        contract.transfer_from(carol(), alice(), transfer_amount.into());
        context.storage_usage = env::storage_usage();
        context.account_balance = env::account_balance();

        context.is_view = true;
        context.attached_deposit = 0;
        testing_env!(context.clone());
        assert_eq!(
            contract.get_balance(carol()).0,
            (total_supply - transfer_amount)
        );
        assert_eq!(contract.get_balance(alice()).0, transfer_amount);
        assert_eq!(
            contract.get_allowance(carol(), bob()).0,
            allowance - transfer_amount
        );
    }

    #[test]
    fn test_self_allowance_set_for_refund() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = FungibleToken::new(carol(), total_supply.into());
        context.storage_usage = env::storage_usage();

        let initial_balance = context.account_balance;
        let initial_storage = context.storage_usage;
        context.attached_deposit = STORAGE_PRICE_PER_BYTE * 1000;
        testing_env!(context.clone());
        contract.inc_allowance(bob(), (total_supply / 2).into());
        context.storage_usage = env::storage_usage();
        context.account_balance = env::account_balance();
        assert_eq!(
            context.account_balance,
            initial_balance
                + Balance::from(context.storage_usage - initial_storage) * STORAGE_PRICE_PER_BYTE
        );

        let initial_balance = context.account_balance;
        let initial_storage = context.storage_usage;
        testing_env!(context.clone());
        context.attached_deposit = 0;
        testing_env!(context.clone());
        contract.dec_allowance(bob(), (total_supply / 2).into());
        context.storage_usage = env::storage_usage();
        context.account_balance = env::account_balance();
        assert!(context.storage_usage < initial_storage);
        assert!(context.account_balance < initial_balance);
        assert_eq!(
            context.account_balance,
            initial_balance
                - Balance::from(initial_storage - context.storage_usage) * STORAGE_PRICE_PER_BYTE
        );
    }

    #[test]
    fn test_voter_addition() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = FungibleToken::new(carol(), total_supply.into());
        context.storage_usage = env::storage_usage();
        contract.create_voter_profile("c1d1a89574c6e744d982e0f2bf1154ef05c13".to_owned());
        let voter_id = contract.get_user_id(bob());
        assert_eq!(voter_id, 1);
        let voter = contract.get_voter_details(1);
        assert_eq!(
            "c1d1a89574c6e744d982e0f2bf1154ef05c13".to_owned(),
            voter.profile_hash
        );
    }

    #[test]
    fn test_voter_stake() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = FungibleToken::new(carol(), total_supply.into());
        context.storage_usage = env::storage_usage();
        contract.create_voter_profile("c1d1a89574c6e744d982e0f2bf1154ef05c13".to_owned());
        let voter_id = contract.get_user_id(bob());
        assert_eq!(voter_id, 1);
        let voter = contract.get_voter_details(1);
        assert_eq!(
            "c1d1a89574c6e744d982e0f2bf1154ef05c13".to_owned(),
            voter.profile_hash
        );
        context.storage_usage = env::storage_usage();

        context.attached_deposit = 1000 * STORAGE_PRICE_PER_BYTE;
        testing_env!(context.clone());
        let transfer_amount = total_supply / 3;
        contract.transfer(bob(), transfer_amount.into());
        context.storage_usage = env::storage_usage();
        context.account_balance = env::account_balance();

        context.is_view = true;
        context.attached_deposit = 0;
        testing_env!(context.clone());
        assert_eq!(
            contract.get_balance(carol()).0,
            (total_supply - transfer_amount)
        );
        assert_eq!(contract.get_balance(bob()).0, transfer_amount);
        context.is_view = false;
        testing_env!(context.clone());
        // println!("{}", contract.get_balance(bob()).0);
        let intialtotalsupply = contract.get_total_supply().0;
        contract.create_voter_stake(50);
        // println!("{}", contract.get_balance(bob()).0);
        assert_eq!(contract.get_balance(bob()).0, transfer_amount - 50);
        // let totalsupply = contract.get_total_supply();
        // println!("{}", totalsupply.0);
        assert_eq!(contract.get_total_supply().0, intialtotalsupply - 50);
        let stake = contract.get_voter_stake(voter_id);
        // println!(">>>>>{}<<<<<<<", stake);
        assert_eq!(stake, 50);
    }
}
