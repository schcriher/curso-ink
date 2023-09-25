#![cfg_attr(not(feature = "std"), no_std, no_main)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::new_without_default))]
mod errors;
mod types;
mod voting;

#[ink::contract]
mod organization {
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    use nft::Psp34Ref;

    use crate::errors::Error;
    use crate::types::{Contributor, Reputation};
    use crate::voting::VoteTrait;

    /// Voting event
    #[ink(event)]
    pub struct VoteEvent {
        #[ink(topic)]
        from: AccountId,

        #[ink(topic)]
        to: AccountId,
    }

    #[ink(storage)]
    pub struct Organization {
        /// Administrator wallet, is who can add or remove contributors
        administrator: AccountId,

        /// List of contributors with their information
        contributors: Mapping<AccountId, Contributor>,

        /// Contract generating NFTs, proof of vote
        nft_contract: Psp34Ref,
    }

    fn get_score(reputation: Reputation) -> Reputation {
        // FIXME: temporary implementation, until the business logic is better defined
        if reputation < 10 {
            1
        } else {
            2
        }
    }

    /////////////////////////////////////////////////////////////////////////////////////

    impl Organization {
        /// Constructor initializes the `administrator`, an empty map of `contributors`
        /// and code hash of NFT contract
        #[ink(constructor)]
        pub fn new(administrator: AccountId, nft_contract_code_hash: Hash) -> Self {
            Self {
                administrator,
                contributors: Mapping::default(),
                nft_contract: Psp34Ref::new()
                    .code_hash(nft_contract_code_hash)
                    .endowment(0)
                    .salt_bytes(Vec::new())
                    .instantiate(),
            }
        }

        // ------------------------------------------------------------------------------

        /// Administrative function: adding a contributor
        #[ink(message)]
        pub fn add_contributor(&mut self, contributor_id: AccountId) -> Result<(), Error> {
            if self.administrator != self.env().caller() {
                return Err(Error::AdministrativeFunction);
            }

            if self.administrator == contributor_id {
                return Err(Error::AdminCannotBeContributor);
            }

            if self.contributors.contains(contributor_id) {
                return Err(Error::ContributorAlreadyExists);
            }

            self.contributors
                .insert(contributor_id, &Contributor { reputation: 0 });
            Ok(())
        }

        /// Administrative function: removing a contributor
        #[ink(message)]
        pub fn rem_contributor(&mut self, contributor_id: AccountId) -> Result<(), Error> {
            if self.administrator != self.env().caller() {
                return Err(Error::AdministrativeFunction);
            }

            if self.administrator == contributor_id {
                return Err(Error::AdminCannotBeContributor);
            }

            if !self.contributors.contains(contributor_id) {
                return Err(Error::ContributorNotExist);
            }

            self.contributors.remove(contributor_id);
            Ok(())
        }
    }

    /////////////////////////////////////////////////////////////////////////////////////

    impl VoteTrait for Organization {
        /// Submit a vote, the caller gives the vote to `receiver_id`
        #[ink(message)]
        fn submit_vote(&mut self, receiver_id: AccountId) -> Result<(), Error> {
            // FIXME: it is assumed that the sum of votes is the reputation

            let emitter_id = self.env().caller();

            if self.administrator == emitter_id {
                return Err(Error::AdminCannotSubmitOrReceivedVote);
            }

            if self.administrator == receiver_id {
                return Err(Error::AdminCannotSubmitOrReceivedVote);
            }

            if emitter_id == receiver_id {
                return Err(Error::CannotVoteItself);
            }

            if !self.contributors.contains(emitter_id) {
                return Err(Error::YouAreNotContributor);
            }

            if !self.contributors.contains(receiver_id) {
                return Err(Error::ContributorNotExist);
            }

            if self.nft_contract.mint_to(emitter_id).is_err() {
                // FIXME: information should be extracted from the PSP34Error
                return Err(Error::NftNotSent);
            }

            // unwrap is safe here
            let emitter = self.contributors.get(emitter_id).unwrap();
            let mut receiver = self.contributors.get(receiver_id).unwrap();

            receiver.reputation += get_score(emitter.reputation);
            self.contributors.insert(receiver_id, &receiver); // update

            self.env().emit_event(VoteEvent {
                from: emitter_id,
                to: receiver_id,
            });

            Ok(())
        }

        /// Getting the reputation of a contributor, from whom it is consulted
        #[ink(message)]
        fn get_reputation(&self) -> Result<Reputation, Error> {
            let caller_id = self.env().caller();

            if !self.contributors.contains(caller_id) {
                return Err(Error::YouAreNotContributor);
            }

            let contributor = self.contributors.get(caller_id).unwrap(); // unwrap is safe here
            Ok(contributor.reputation)
        }
    }

    /////////////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    mod unit_tests {
        use super::*;

        #[test]
        fn get_score_test() {
            assert_eq!(get_score(0), 1);
            assert_eq!(get_score(9), 1);
            assert_eq!(get_score(10), 2);
            assert_eq!(get_score(100), 2);
            assert_eq!(get_score(1000), 2);
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

            let get_reputation = build_message::<OrganizationRef>(contract_id.clone())
                .call(|contract| contract.get_reputation());
            let get_reputation_return = client
                .call_dry_run(&bob.key, &get_reputation, 0, None)
                .await
                .return_value();

            assert!(matches!(get_reputation_return, Ok(0)));

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

        #[ink_e2e::test]
        async fn submit_vote_test(mut client: ink_e2e::Client<C, E>) -> E2EResult {
            init_e2e_test!(client, contract_id, alice, bob, dave);

            let add_contributor = build_message::<OrganizationRef>(contract_id.clone())
                .call(|contract| contract.add_contributor(bob.id));
            let add_contributor_return = client.call(&alice.key, add_contributor, 0, None).await;

            assert!(add_contributor_return.is_ok());

            let add_contributor = build_message::<OrganizationRef>(contract_id.clone())
                .call(|contract| contract.add_contributor(dave.id));
            let add_contributor_return = client.call(&alice.key, add_contributor, 0, None).await;

            assert!(add_contributor_return.is_ok());

            let submit_vote = build_message::<OrganizationRef>(contract_id.clone())
                .call(|contract| contract.submit_vote(dave.id));
            let submit_vote_return = client.call(&bob.key, submit_vote, 0, None).await;

            assert!(submit_vote_return.is_ok());

            let get_reputation = build_message::<OrganizationRef>(contract_id.clone())
                .call(|contract| contract.get_reputation());
            let get_reputation_return = client
                .call_dry_run(&dave.key, &get_reputation, 0, None)
                .await
                .return_value();

            assert!(matches!(get_reputation_return, Ok(1)));

            Ok(())
        }
    }
}
