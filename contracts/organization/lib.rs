#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod errors;
mod types;
mod voting;

#[cfg_attr(feature = "cargo-clippy", allow(clippy::new_without_default))]
#[ink::contract]
mod organization {
    use ink::storage::Mapping;

    use crate::errors::Error;
    use crate::types::{Contributor, Reputation};
    use crate::voting::VoteTrait;

    use nft::Psp34Ref;

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
                    .salt_bytes([0x46, 0x55, 0x43, 0x4B]) // 🤷🏼
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

            // FIXME: temporary implementation, until the business logic is better defined
            let sum = if emitter.reputation < 10 { 1 } else { 2 };

            receiver.reputation += sum;
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

    /// End-to-End or integration tests
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::build_message;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_test(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            Ok(())
        }
    }
}
