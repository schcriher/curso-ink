#![cfg_attr(not(feature = "std"), no_std, no_main)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::new_without_default))]
mod errors;
mod tools;
mod types;
mod voting;

// https://use.ink
// https://paritytech.github.io/ink/ink_env/#functions
// https://github.com/paritytech/ink-examples/tree/main

#[ink::contract]
mod organization {
    use ink::prelude::{string::String, vec::Vec};
    use ink::storage::{Lazy, Mapping};

    use scale::alloc::borrow::ToOwned;

    use nft::Psp34Ref;

    use crate::errors::Error;
    use crate::tools::sqrt_fast;
    use crate::types::{
        Contributor, Reputation, Role, Round, RoundId, Vote, VoteSign, VotesNumber,
    };
    use crate::voting::VoteTrait;

    type Result<T> = core::result::Result<T, Error>;

    //--- Events ----------------------------------------------------------------------//

    /// Vote cast event.
    #[ink(event)]
    pub struct VoteCast {
        #[ink(topic)]
        round_id: RoundId,
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        // ---
        value: VotesNumber,
    }

    /// New round event.
    #[ink(event)]
    pub struct NewRound {
        #[ink(topic)]
        round_id: RoundId,
        // ---
        value: Balance,
        max_votes: VotesNumber,
        finish_at: Timestamp,
    }

    /// Close round event.
    #[ink(event)]
    pub struct CloseRound {
        #[ink(topic)]
        round_id: RoundId,
        // ---
        total_votes: VotesNumber,
        total_reputation: Reputation,
    }

    //---------------------------------------------------------------------------------//

    #[ink(storage)]
    pub struct Organization {
        /// Map with all rounds.
        rounds: Mapping<RoundId, Round>,

        /// Current round of voting, starts at 1,
        /// limitation: can be active only one round at a time.
        current_round_id: RoundId,

        /// Minimum time for a round
        min_elapsed_milliseconds: Timestamp,

        /// Map with all members and their role.
        members: Mapping<AccountId, Role>,

        /// Map with all contributors and their current reputation.
        contributors: Mapping<AccountId, Contributor>,

        /// List of all contributors, necessary to distribute the funds.
        contributors_list: Lazy<Vec<AccountId>>,

        /// Reference to the NFT contract, which is the proof of vote.
        nft_ref: Psp34Ref,
    }

    //---------------------------------------------------------------------------------//

    /// Function that computes the reputation of a contributor,
    /// receives the current reputation and returns the new one.
    fn get_reputation(receiver: Reputation, emitter: Reputation, vote: Vote) -> Reputation {
        let receiver = receiver as i64;
        let emitter = emitter as i64;
        let value = vote.value as i64;
        let sign = if vote.sign == VoteSign::Positive {
            1
        } else {
            -1
        };
        let reputation = receiver + sign * value * sqrt_fast(emitter);
        if reputation < 1 {
            1
        } else if reputation > Reputation::MAX as i64 {
            Reputation::MAX
        } else {
            reputation as Reputation // overflow is not possible
        }
    }

    /////////////////////////////////////////////////////////////////////////////////////

    impl Organization {
        /// The constructor initializes the organization,
        /// including the administrator as a Admin member and instantiates the nft contract.
        #[ink(constructor)]
        pub fn new(
            administrator_id: AccountId,
            nft_code_hash: Hash,
            min_elapsed_hours: u32,
        ) -> Self {
            let rounds = Mapping::default();
            let mut members = Mapping::default();
            let contributors = Mapping::default();
            let mut contributors_list = Lazy::new();

            members.insert(administrator_id, &Role::Admin);
            contributors_list.set(&Vec::new());

            let min_elapsed_milliseconds = (min_elapsed_hours * 60 * 60 * 1000) as Timestamp;

            Self {
                rounds,
                members,
                contributors,
                contributors_list,
                current_round_id: 0,
                min_elapsed_milliseconds,
                nft_ref: Psp34Ref::new()
                    .code_hash(nft_code_hash)
                    .endowment(0)
                    .salt_bytes(Vec::new())
                    .instantiate(),
            }
        }

        // ------------------------------------------------------------------------------

        fn add_member(&mut self, contributor_id: AccountId, role: Role) {
            self.members.insert(contributor_id, &role);

            if role == Role::Contributor {
                self.contributors
                    .insert(contributor_id, &Contributor::default());

                let mut list = self.contributors_list.get().unwrap();
                list.push(contributor_id);
                self.contributors_list.set(&list);
            }
        }

        fn rem_member(&mut self, contributor_id: AccountId) {
            self.members.remove(contributor_id);

            self.contributors.remove(contributor_id);

            let mut list = self.contributors_list.get().unwrap();
            list.retain(|x| *x != contributor_id);
            self.contributors_list.set(&list);
        }

        fn contributor_reset_if_necessary(&self, mut contributor: Contributor) {
            if contributor.round_id != self.current_round_id {
                contributor.round_id = self.current_round_id;
                contributor.reputation = 1;
                contributor.votes_submitted = 0;
            }
        }

        fn is_caller_admin(&self) -> Result<()> {
            let caller_id = self.env().caller();
            let caller_member = self.members.get(caller_id);

            if caller_member.is_none() || caller_member.unwrap() != Role::Admin {
                return Err(Error::AdministrativeFunction);
            }

            Ok(())
        }

        fn is_active_round(&self) -> Result<()> {
            if let Some(round) = self.rounds.get(self.current_round_id) {
                if !round.is_finished {
                    return Err(Error::IsAnNoFinishedRound);
                }
            }
            Ok(())
        }

        // ------------------------------------------------------------------------------

        /// Administrative function: adding a administrator.
        #[ink(message)]
        pub fn add_admin(&mut self, contributor_id: AccountId) -> Result<()> {
            self.is_caller_admin()?;

            if self.members.contains(contributor_id) {
                return Err(Error::MemberAlreadyExists);
            }

            self.add_member(contributor_id, Role::Admin);

            Ok(())
        }

        /// Administrative function: removing a administrator.
        #[ink(message)]
        pub fn rem_admin(&mut self, contributor_id: AccountId) -> Result<()> {
            self.is_caller_admin()?;

            if !self.members.contains(contributor_id) {
                return Err(Error::MemberNotExist);
            }

            if self.env().caller() == contributor_id {
                // this prevents the contract from running out of administrators
                return Err(Error::CannotRemoveYourself);
            }

            self.rem_member(contributor_id);

            Ok(())
        }

        /// Administrative function: adding a contributor, there must be no active round.
        #[ink(message)]
        pub fn add_contributor(&mut self, contributor_id: AccountId) -> Result<()> {
            self.is_caller_admin()?;
            self.is_active_round()?;

            if self.members.contains(contributor_id) {
                return Err(Error::MemberAlreadyExists);
            }

            self.add_member(contributor_id, Role::Contributor);

            Ok(())
        }

        /// Administrative function: removing a contributor, there must be no active round.
        #[ink(message)]
        pub fn rem_contributor(&mut self, contributor_id: AccountId) -> Result<()> {
            self.is_caller_admin()?;
            self.is_active_round()?;

            if !self.members.contains(contributor_id) {
                return Err(Error::MemberNotExist);
            }

            self.rem_member(contributor_id);

            Ok(())
        }

        /// Administrative function: adds a new round of distribution.
        #[ink(message)]
        pub fn open_round(
            &mut self,
            name: String,
            value: Balance,
            max_votes: VotesNumber,
            finish_at: Timestamp,
        ) -> Result<()> {
            self.is_caller_admin()?;
            self.is_active_round()?;

            if value <= self.env().balance() + self.env().minimum_balance() {
                return Err(Error::InsufficientFunds);
            }

            if max_votes < 1 {
                return Err(Error::InvalidRoundParameter);
            }

            if finish_at < self.env().block_timestamp() + self.min_elapsed_milliseconds {
                return Err(Error::InvalidRoundParameter);
            }

            let round = Round {
                name,
                value,
                max_votes,
                finish_at,
                is_finished: false,
            };
            self.current_round_id += 1;
            self.rounds.insert(self.current_round_id, &round);

            self.env().emit_event(NewRound {
                round_id: self.current_round_id,
                value,
                max_votes,
                finish_at,
            });

            Ok(())
        }

        /// Administrative function: distributing funds to contributors.
        #[ink(message)]
        pub fn close_round(&mut self) -> Result<()> {
            self.is_caller_admin()?;

            let round = self.rounds.get(self.current_round_id);

            if round.is_none() {
                return Err(Error::IsNoActiveRound);
            }

            let mut round = round.unwrap(); // unwrap is safe here

            if round.is_finished {
                return Err(Error::IsNoActiveRound);
            }
            if round.finish_at > self.env().block_timestamp() {
                return Err(Error::NotYetFinishedRound);
            }

            let mut total_votes = 0;
            let mut total_reputation = 0;
            let mut contributors = Vec::new();

            for contributor_id in self.contributors_list.get().unwrap().iter() {
                let contributor = self.contributors.get(contributor_id).unwrap();

                // if he did not cast a vote or a vote was cast for him,
                // he may have the data from the previous round.
                self.contributor_reset_if_necessary(contributor);
                // NOTE: If no votes were cast, the funds are distributed equally.

                total_votes += contributor.votes_submitted;
                total_reputation += contributor.reputation;
                contributors.push((contributor_id.to_owned(), contributor.reputation));
            }

            // unwrap is safe here: total_reputation != 0
            let min_fraction = round.value.checked_div(total_reputation.into()).unwrap();

            for contributor in contributors.iter() {
                let amount = min_fraction
                    .checked_mul(contributor.1.into())
                    .ok_or(Error::MulOverflow(min_fraction, contributor.1.into()))?; // should not happen

                self.env()
                    .transfer(contributor.0, amount)
                    .map_err(|_| Error::TransferFailed(contributor.0, amount))?;
            }

            contributors.sort_by(|a, b| a.1.cmp(&b.1)); // sorted from lowest to highest by reputation

            // FIXME: NFTs must be of three types

            // The last element is the most reputable - Gold NFT
            if let Some(contributor) = contributors.pop() {
                self.nft_ref
                    .mint_to(contributor.0)
                    .map_err(|_| Error::NftNotSent)?;
            }

            // The second to last element is the second in reputation - Silver NFT
            if let Some(contributor) = contributors.pop() {
                self.nft_ref
                    .mint_to(contributor.0)
                    .map_err(|_| Error::NftNotSent)?;
            }

            // The third to last element is the third in reputation - Bronze NFT
            if let Some(contributor) = contributors.pop() {
                self.nft_ref
                    .mint_to(contributor.0)
                    .map_err(|_| Error::NftNotSent)?;
            }

            round.is_finished = true;
            self.rounds.insert(self.current_round_id, &round);

            self.env().emit_event(CloseRound {
                round_id: self.current_round_id,
                total_votes,
                total_reputation,
            });

            Ok(())
        }

        /// Administrative function: set the minimum time for a round, for the next round.
        #[ink(message)]
        pub fn set_min_elapsed_milliseconds(&mut self, milliseconds: Timestamp) -> Result<()> {
            self.is_caller_admin()?;
            self.min_elapsed_milliseconds = milliseconds;
            Ok(())
        }

        /// Get the minimum time for a round.
        #[ink(message)]
        pub fn get_min_elapsed_milliseconds(&self) -> Timestamp {
            self.min_elapsed_milliseconds
        }

        /// Get the address of the contract
        #[ink(message)]
        pub fn get_contract_account_id(&self) -> AccountId {
            self.env().account_id()
        }

        #[ink(message)]
        pub fn get_block_timestamp(&self) -> Timestamp {
            self.env().block_timestamp()
        }
    }

    /////////////////////////////////////////////////////////////////////////////////////

    impl VoteTrait for Organization {
        /// Submit a vote, the caller (`emitter_id`) gives the vote to `receiver_id`.
        #[ink(message)]
        fn submit_vote(&mut self, receiver_id: AccountId, vote: Vote) -> Result<()> {
            let emitter_id = self.env().caller();
            let emitter_member = self.members.get(emitter_id);
            let receiver_member = self.members.get(receiver_id);

            if emitter_member.is_none() || emitter_member.unwrap() != Role::Contributor {
                return Err(Error::OnlyContributorCanVote);
            }

            if receiver_member.is_none() || receiver_member.unwrap() != Role::Contributor {
                return Err(Error::OnlyContributorCanVote);
            }

            if emitter_id == receiver_id {
                return Err(Error::CannotVoteItself);
            }

            let round = self.rounds.get(self.current_round_id);
            if round.is_none() || round.clone().unwrap().finish_at < self.env().block_timestamp() {
                return Err(Error::IsNoActiveRound);
            }

            // unwraps is safe here
            let round = round.unwrap();
            let emitter = self.contributors.get(emitter_id).unwrap();
            let mut receiver = self.contributors.get(receiver_id).unwrap();

            if vote.value > round.max_votes {
                return Err(Error::ExceedsVoteLimit(round.max_votes));
            }

            // for efficiency reasons, this check is performed here, instead of in `close_round`
            self.contributor_reset_if_necessary(emitter);
            self.contributor_reset_if_necessary(receiver);

            if emitter.votes_submitted + vote.value > round.max_votes {
                return Err(Error::ExceedsYourVoteLimit(
                    round.max_votes - emitter.votes_submitted,
                ));
            }

            receiver.reputation = get_reputation(receiver.reputation, emitter.reputation, vote);

            // persist contributor data
            self.contributors.insert(emitter_id, &emitter); // may not update anything
            self.contributors.insert(receiver_id, &receiver);

            self.env().emit_event(VoteCast {
                round_id: self.current_round_id,
                from: emitter_id,
                to: receiver_id,
                value: vote.value,
            });

            Ok(())
        }

        /// Getting the reputation of a contributor, from whom it is consulted.
        #[ink(message)]
        fn get_reputation(&self) -> Result<Reputation> {
            let caller_id = self.env().caller();
            let caller_member = self.members.get(caller_id);

            if caller_member.is_none() || caller_member.unwrap() != Role::Admin {
                return Err(Error::YouAreNotContributor);
            }

            let round = self.rounds.get(self.current_round_id);
            if round.is_none() || round.clone().unwrap().finish_at < self.env().block_timestamp() {
                return Err(Error::IsNoActiveRound);
            }

            let caller = self.contributors.get(caller_id).unwrap(); // unwrap is safe here
            Ok(caller.reputation)
        }
    }

    /////////////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    mod unit_tests {
        use super::*;

        #[test]
        fn get_reputation_test() {
            let vote1positive = Vote {
                sign: VoteSign::Positive,
                value: 1,
            };

            let vote1negative = Vote {
                sign: VoteSign::Negative,
                value: 1,
            };

            // get_reputation(receiver, emitter, vote) -> receiver reputation

            assert_eq!(get_reputation(1, 1, vote1positive), 2);
            assert_eq!(get_reputation(1, 10, vote1positive), 4);

            assert_eq!(get_reputation(1, 1, vote1negative), 1);
            assert_eq!(get_reputation(1, 10, vote1negative), 1);

            assert_eq!(get_reputation(10, 1, vote1positive), 11);
            assert_eq!(get_reputation(10, 10, vote1positive), 13);

            assert_eq!(get_reputation(10, 1, vote1negative), 9);
            assert_eq!(get_reputation(10, 10, vote1negative), 7);

            let vote10positive = Vote {
                sign: VoteSign::Positive,
                value: 10,
            };

            let vote10negative = Vote {
                sign: VoteSign::Negative,
                value: 10,
            };

            // get_reputation(receiver, emitter, vote) -> receiver reputation

            assert_eq!(get_reputation(1, 1, vote10positive), 11);
            assert_eq!(get_reputation(1, 10, vote10positive), 31);

            assert_eq!(get_reputation(1, 1, vote10negative), 1);
            assert_eq!(get_reputation(1, 10, vote10negative), 1);

            assert_eq!(get_reputation(10, 1, vote10positive), 20);
            assert_eq!(get_reputation(10, 10, vote10positive), 40);

            assert_eq!(get_reputation(10, 1, vote10negative), 1);
            assert_eq!(get_reputation(10, 10, vote10negative), 1);
        }
    }

    //---------------------------------------------------------------------------------//

    #[cfg(test)]
    mod integration_tests {
        //
        // This contract instance a second contract in its constructor,
        // but off-chain environment does not support contract instantiation,
        // so it is not possible to make integration tests.
        //
    }

    //---------------------------------------------------------------------------------//

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::{build_message, Keypair};

        type E2EResult = std::result::Result<(), Box<dyn std::error::Error>>;

        pub struct E2EAccount {
            pub key: Keypair,
            pub id: AccountId,
        }

        // Account: alice, bob, charlie, dave, eve, ferdie, one, two
        macro_rules! get_e2e_account {
            ($account:ident) => {
                let $account = E2EAccount {
                    key: ink_e2e::$account(),
                    id: AccountId::from(ink_e2e::$account().public_key().0),
                };
            };
        }

        macro_rules! init_e2e_test {
            ($client:expr, $contract_id:ident, $admin_account:ident $(, $account:ident)*) => {
                get_e2e_account!($admin_account);

                let nft_code_hash = $client
                    .upload("nft", &$admin_account.key, None)
                    .await
                    .expect("nft contract upload failed")
                    .code_hash;

                let min_elapsed_hours = 24; // 1 day
                let contract_ref = OrganizationRef::new($admin_account.id, nft_code_hash, min_elapsed_hours);
                let $contract_id = $client
                    .instantiate("organization", &$admin_account.key, contract_ref, 0, None)
                    .await
                    .expect("organization contract instantiate failed")
                    .account_id;

                $( get_e2e_account!($account); )*
            };
        }

        //--------------------------------//

        #[ink_e2e::test]
        async fn add_contributor_test(mut client: ink_e2e::Client<C, E>) -> E2EResult {
            init_e2e_test!(client, contract_id, alice, bob);

            let add_contributor = build_message::<OrganizationRef>(contract_id.clone())
                .call(|contract| contract.add_contributor(bob.id));
            let add_contributor_return = client.call(&alice.key, add_contributor, 0, None).await;

            assert!(add_contributor_return.is_ok());

            Ok(())
        }

        #[ink_e2e::test]
        async fn rem_contributor_test(mut client: ink_e2e::Client<C, E>) -> E2EResult {
            init_e2e_test!(client, contract_id, alice, bob);

            let add_contributor = build_message::<OrganizationRef>(contract_id.clone())
                .call(|contract| contract.add_contributor(bob.id));
            let add_contributor_return = client.call(&alice.key, add_contributor, 0, None).await;

            assert!(add_contributor_return.is_ok());

            let rem_contributor = build_message::<OrganizationRef>(contract_id.clone())
                .call(|contract| contract.rem_contributor(bob.id));
            let rem_contributor_return = client.call(&alice.key, rem_contributor, 0, None).await;

            assert!(rem_contributor_return.is_ok());

            Ok(())
        }

        // #[ink_e2e::test]
        // async fn submit_vote_test(mut client: ink_e2e::Client<C, E>) -> E2EResult {
        //     init_e2e_test!(client, contract_id, alice, bob, dave);

        //     let add_contributor = build_message::<OrganizationRef>(contract_id.clone())
        //         .call(|contract| contract.add_contributor(bob.id));
        //     let add_contributor_return = client.call(&alice.key, add_contributor, 0, None).await;

        //     assert!(add_contributor_return.is_ok());

        //     let add_contributor = build_message::<OrganizationRef>(contract_id.clone())
        //         .call(|contract| contract.add_contributor(dave.id));
        //     let add_contributor_return = client.call(&alice.key, add_contributor, 0, None).await;

        //     assert!(add_contributor_return.is_ok());

        //     let submit_vote = build_message::<OrganizationRef>(contract_id.clone())
        //         .call(|contract| contract.submit_vote(dave.id));
        //     let submit_vote_return = client.call(&bob.key, submit_vote, 0, None).await;

        //     assert!(submit_vote_return.is_ok());

        //     let get_reputation = build_message::<OrganizationRef>(contract_id.clone())
        //         .call(|contract| contract.get_reputation());
        //     let get_reputation_return = client
        //         .call_dry_run(&dave.key, &get_reputation, 0, None)
        //         .await
        //         .return_value();

        //     ///////////////////////////////////////////////////////////////////////////
        //     let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
        //     assert_eq!(emitted_events.len(), 2);
        //     // Check first transfer event related to ERC-20 instantiation.
        //     assert_transfer_event(
        //         &emitted_events[0],
        //         None,
        //         Some(AccountId::from([0x01; 32])),
        //         100,
        //     );
        //     // Check the second transfer event relating to the actual trasfer.
        //     assert_transfer_event(
        //         &emitted_events[1],
        //         Some(AccountId::from([0x01; 32])),
        //         Some(AccountId::from([0x02; 32])),
        //         10,
        //     );
        //     ///////////////////////////////////////////////////////////////////////////

        //     assert!(matches!(get_reputation_return, Ok(1)));

        //     Ok(())
        // }
    }
}
