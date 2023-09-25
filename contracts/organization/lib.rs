#![cfg_attr(not(feature = "std"), no_std, no_main)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::new_without_default))]
mod errors;
mod types;
mod voting;

#[ink::contract]
mod organization {
    use ink::prelude::{string::String, vec::Vec};
    use ink::storage::{Lazy, Mapping};

    use scale::alloc::borrow::ToOwned;

    use nft::Psp34Ref;

    use crate::errors::Error;
    use crate::types::{
        Contributor, Reputation, Role, Round, RoundId, Vote, VoteSign, VotesNumber,
    };
    use crate::voting::VoteTrait;

    /// Voting event
    #[ink(event)]
    pub struct VoteEvent {
        #[ink(topic)]
        from: AccountId,

        #[ink(topic)]
        to: AccountId,

        #[ink(topic)]
        round_id: RoundId,

        value: VotesNumber,
    }

    /// New round event
    #[ink(event)]
    pub struct NewRoundEvent {
        #[ink(topic)]
        round_id: RoundId,

        value: Balance,
        finish_at: Timestamp,
        max_votes: VotesNumber,
    }

    #[ink(storage)]
    pub struct Organization {
        /// Map with all rounds
        rounds: Mapping<RoundId, Round>,

        /// Current round of voting,
        /// limitation: can be active only one round at a time
        current_round: RoundId,

        /// Map with all members and their role
        members: Mapping<AccountId, Role>,

        /// Map with all contributors and their current reputation
        contributors: Mapping<AccountId, Contributor>,

        /// List of all contributors
        contributors_list: Lazy<Vec<AccountId>>,

        /// Reference to the NFT contract, which is the proof of vote
        nft_ref: Psp34Ref,
    }

    /////////////////////////////////////////////////////////////////////////////////////

    /// Function that computes the approximate square root of a number (fast)
    fn sqrt(v: i64) -> i64 {
        // https://github.com/chmike/fpsqrt/blob/df099181030e95d663d89e87d4bf2d36534776a5/fpsqrt.c#L51
        assert!(v >= 0, "sqrt input should be non-negative");

        let mut b: u64 = 1 << 62;
        let mut q: u64 = 0;
        let mut r: u64 = v as u64;

        while b > r {
            b >>= 2;
        }
        while b > 0 {
            let t = q + b;
            q >>= 1;
            if r >= t {
                r -= t;
                q += b;
            }
            b >>= 2;
        }

        q as i64
    }

    /// Function that computes the reputation of a contributor,
    /// receives the current reputation and returns the new one
    fn get_reputation(receiver: Reputation, emitter: Reputation, vote: Vote) -> Reputation {
        let receiver = receiver as i64;
        let emitter = emitter as i64;
        let value = vote.value as i64;
        let sign = if vote.sign == VoteSign::Positive {
            1
        } else {
            -1
        };
        let reputation = receiver + sign * value * sqrt(emitter);
        if reputation < 1 {
            1
        } else if reputation > Reputation::MAX as i64 {
            Reputation::MAX
        } else {
            reputation as Reputation
        }
    }

    /////////////////////////////////////////////////////////////////////////////////////

    impl Organization {
        /// The constructor initializes the organization,
        /// including the administrator as a Admin member and instantiates the nft contract.
        #[ink(constructor)]
        pub fn new(administrator_id: AccountId, nft_code_hash: Hash) -> Self {
            let rounds = Mapping::default();
            let mut members = Mapping::default();
            let contributors = Mapping::default();
            let mut contributors_list = Lazy::new();

            contributors_list.set(&Vec::new());
            members.insert(administrator_id, &Role::Admin);

            Self {
                rounds,
                members,
                contributors,
                contributors_list,
                current_round: 0,
                nft_ref: Psp34Ref::new()
                    .code_hash(nft_code_hash)
                    .endowment(0)
                    .salt_bytes(Vec::new())
                    .instantiate(),
            }
        }

        // ------------------------------------------------------------------------------

        /// Administrative function: adding a contributor
        #[ink(message)]
        pub fn add_contributor(&mut self, contributor_id: AccountId) -> Result<(), Error> {
            let caller_id = self.env().caller();
            let caller_member = self.members.get(caller_id);

            if caller_member.is_none() || caller_member.unwrap() != Role::Admin {
                return Err(Error::AdministrativeFunction);
            }

            if caller_id == contributor_id {
                return Err(Error::AdminCannotBeContributor);
            }

            if self.members.contains(contributor_id) {
                return Err(Error::MemberAlreadyExists);
            }

            self.members.insert(contributor_id, &Role::Contributor);

            self.contributors.insert(
                contributor_id,
                &Contributor {
                    round_id: 0,
                    reputation: 1,
                    votes_submitted: 0,
                },
            );

            let mut list = self.contributors_list.get().unwrap();
            list.push(contributor_id);
            self.contributors_list.set(&list);

            Ok(())
        }

        /// Administrative function: removing a contributor
        #[ink(message)]
        pub fn rem_contributor(&mut self, contributor_id: AccountId) -> Result<(), Error> {
            let caller_id = self.env().caller();
            let caller_member = self.members.get(caller_id);

            if caller_member.is_none() || caller_member.unwrap() != Role::Admin {
                return Err(Error::AdministrativeFunction);
            }

            if !self.members.contains(contributor_id) {
                return Err(Error::MemberNotExist);
            }

            self.members.remove(contributor_id);
            self.contributors.remove(contributor_id);

            let mut list = self.contributors_list.get().unwrap();
            list.retain(|x| *x != contributor_id);
            self.contributors_list.set(&list);

            Ok(())
        }

        /// Administrative function: adds a new round of distribution
        #[ink(message)]
        pub fn open_round(
            &mut self,
            name: String,
            value: Balance,
            finish_at: Timestamp,
            max_votes: VotesNumber,
        ) -> Result<(), Error> {
            let caller_id = self.env().caller();
            let caller_member = self.members.get(caller_id);

            if caller_member.is_none() || caller_member.unwrap() != Role::Admin {
                return Err(Error::AdministrativeFunction);
            }

            if let Some(round) = self.rounds.get(self.current_round) {
                if !round.is_finished {
                    return Err(Error::IsOneActiveRound);
                }
            }

            // TODO: A minimum time could be established

            if finish_at < self.env().block_timestamp() {
                return Err(Error::InvalidRoundParameter);
            }

            // TODO: value of the round must be greater than the contract funds

            let round = Round {
                name,
                value,
                finish_at,
                max_votes,
                is_finished: false,
            };
            self.current_round += 1;
            self.rounds.insert(self.current_round, &round);

            self.env().emit_event(NewRoundEvent {
                round_id: self.current_round,
                value,
                finish_at,
                max_votes,
            });

            Ok(())
        }

        /// Administrative function: distributing funds to contributors
        #[ink(message)]
        pub fn close_round(&mut self) -> Result<(), Error> {
            let caller_id = self.env().caller();
            let caller_member = self.members.get(caller_id);

            if caller_member.is_none() || caller_member.unwrap() != Role::Admin {
                return Err(Error::AdministrativeFunction);
            }

            let round = self.rounds.get(self.current_round);
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

            let mut total_reputation = 0;
            let mut contributors = Vec::new();

            for contributor_id in self.contributors_list.get().unwrap().iter() {
                let contributor = self.contributors.get(contributor_id).unwrap();
                total_reputation += contributor.reputation;
                contributors.push((contributor_id.to_owned(), contributor.reputation));
            }

            for contributor in contributors.iter() {
                let amount = round
                    .value
                    .checked_mul(contributor.1 as u128)
                    .unwrap_or(0)
                    .checked_div(total_reputation as u128)
                    .unwrap_or(0);

                self.env()
                    .transfer(contributor.0, amount)
                    .map_err(|_| Error::TransferFailed(contributor.0, amount))?;
            }

            contributors.sort_by(|a, b| a.1.cmp(&b.1));

            ink::env::debug_println!("{:?}", contributors); ////////////////////////////////// XXX

            // FIXME: NFTs must be of three types

            // First contributor
            if let Some(contributor) = contributors.pop() {
                self.nft_ref
                    .mint_to(contributor.0)
                    .map_err(|_| Error::NftNotSent)?;
            }

            // Second contributor
            if let Some(contributor) = contributors.pop() {
                self.nft_ref
                    .mint_to(contributor.0)
                    .map_err(|_| Error::NftNotSent)?;
            }

            // Third contributor
            if let Some(contributor) = contributors.pop() {
                self.nft_ref
                    .mint_to(contributor.0)
                    .map_err(|_| Error::NftNotSent)?;
            }

            round.is_finished = true;
            self.rounds.insert(self.current_round, &round);

            Ok(())
        }
    }

    /////////////////////////////////////////////////////////////////////////////////////

    impl VoteTrait for Organization {
        /// Submit a vote, the caller (`emitter_id`) gives the vote to `receiver_id`
        #[ink(message)]
        fn submit_vote(&mut self, receiver_id: AccountId, vote: Vote) -> Result<(), Error> {
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

            let round = self.rounds.get(self.current_round);
            if round.is_none() || round.clone().unwrap().finish_at < self.env().block_timestamp() {
                return Err(Error::IsNoActiveRound);
            }

            // unwrap is safe here
            let round = round.unwrap();
            let mut emitter = self.contributors.get(emitter_id).unwrap();
            let mut receiver = self.contributors.get(receiver_id).unwrap();

            if vote.value > round.max_votes {
                return Err(Error::ExceedsVoteLimit(round.max_votes));
            }

            if emitter.round_id != self.current_round {
                emitter.round_id = self.current_round;
                emitter.reputation = 1;
                emitter.votes_submitted = 0;
            }

            if receiver.round_id != self.current_round {
                receiver.round_id = self.current_round;
                receiver.reputation = 1;
                receiver.votes_submitted = 0;
            }

            if emitter.votes_submitted + vote.value > round.max_votes {
                return Err(Error::ExceedsYourVoteLimit(
                    round.max_votes - emitter.votes_submitted,
                ));
            }

            receiver.reputation = get_reputation(receiver.reputation, emitter.reputation, vote);
            self.contributors.insert(receiver_id, &receiver); // update

            self.env().emit_event(VoteEvent {
                from: emitter_id,
                to: receiver_id,
                value: vote.value,
                round_id: self.current_round,
            });

            Ok(())
        }

        /// Getting the reputation of a contributor, from whom it is consulted
        #[ink(message)]
        fn get_reputation(&self) -> Result<Reputation, Error> {
            let caller_id = self.env().caller();
            let caller_member = self.members.get(caller_id);

            if caller_member.is_none() || caller_member.unwrap() != Role::Admin {
                return Err(Error::YouAreNotContributor);
            }

            let round = self.rounds.get(self.current_round);
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
        fn get_sqrt_test() {
            assert_eq!(sqrt(1), 1); //      1
            assert_eq!(sqrt(2), 1); //      1.41…
            assert_eq!(sqrt(10), 3); //     3.16…
            assert_eq!(sqrt(16), 4); //     4
            assert_eq!(sqrt(100), 10); //  10
            assert_eq!(sqrt(500), 22); //  22.36…
        }

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

                let contract_ref = OrganizationRef::new($admin_account.id, nft_code_hash);
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

        //     assert!(matches!(get_reputation_return, Ok(1)));

        //     Ok(())
        // }
    }
}
