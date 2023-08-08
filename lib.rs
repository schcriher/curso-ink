#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod flipper {
    use ink::prelude::vec::Vec;
    use scale::{Decode, Encode};

    #[derive(PartialEq, Debug, Eq, Clone, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        AdministrativeFunction,
        AdminCannotBeContributor,
        AdminCannotSubmitOrReceivedVote,
        ContributorAlreadyExists,
        ContributorNotExist,
        CannotVoteItself,
        YouAreNotContributor,
    }

    /// Contributor information
    #[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Contributor {
        /// Contributor wallet
        account_id: AccountId,
        /// Reputation of the contributor, votes received
        reputation: u32,
    }

    #[ink(storage)]
    pub struct Flipper {
        /// Administrator wallet, is who can add or remove contributors
        administrator: AccountId,
        /// List of contributors with their information
        contributors: Vec<Contributor>,
    }

    impl Flipper {
        /// Constructor initializes the `administrator` and an empty list of `contributors`
        #[ink(constructor)]
        pub fn new(administrator: AccountId) -> Self {
            Self {
                administrator,
                contributors: Vec::new(),
            }
        }

        // ------------------------------------------------------------------------------

        fn get_index(&self, contributor_id: AccountId) -> Option<usize> {
            self.contributors
                .iter()
                .position(|c| c.account_id == contributor_id)
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

            if self.get_index(contributor_id).is_some() {
                return Err(Error::ContributorAlreadyExists);
            }

            self.contributors.push(Contributor {
                account_id: contributor_id,
                reputation: 0,
            });
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

            let index = self.get_index(contributor_id);

            if index.is_none() {
                return Err(Error::ContributorNotExist);
            }

            self.contributors.remove(index.unwrap()); // unwrap is safe here
            Ok(())
        }

        // ------------------------------------------------------------------------------

        /// Submit a vote, the caller gives the vote to `receiver_id`
        #[ink(message)]
        pub fn submit_vote(&mut self, receiver_id: AccountId) -> Result<(), Error> {
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

            if self.get_index(emitter_id).is_none() {
                return Err(Error::YouAreNotContributor);
            }

            let index = self.get_index(receiver_id);

            if index.is_none() {
                return Err(Error::ContributorNotExist);
            }

            let receiver = self.contributors.get_mut(index.unwrap()).unwrap(); // unwrap is safe here
            receiver.reputation += 1;
            Ok(())
        }

        /// Getting the reputation of a contributor
        #[ink(message)]
        pub fn get_reputation(&self, contributor_id: AccountId) -> Result<u32, Error> {
            // FIXME: It is assumed that anyone can see the contributor's reputation

            let index = self.get_index(contributor_id);

            if index.is_none() {
                return Err(Error::ContributorNotExist);
            }

            let contributor = self.contributors.get(index.unwrap()).unwrap(); // unwrap is safe here
            Ok(contributor.reputation)
        }
    }

    /////////////////////////////////////////////////////////////////////////////////////

    /// Unit tests
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{test::set_caller, DefaultEnvironment};

        pub struct Context {
            contract: Flipper,
            admin: AccountId,
            user0: AccountId,
            user1: AccountId,
            user2: AccountId,
        }

        impl Context {
            pub fn new() -> Self {
                let admin = AccountId::from([u8::MAX; 32]);
                let contract = Flipper::new(admin);

                let user0 = AccountId::from([0; 32]);
                let user1 = AccountId::from([1; 32]);
                let user2 = AccountId::from([2; 32]);

                Self {
                    contract,
                    admin,
                    user0,
                    user1,
                    user2,
                }
            }

            pub fn get_contributor(&self, index: usize) -> &Contributor {
                self.contract.contributors.get(index).unwrap()
            }
        }

        // ------------------------------------------------------------------------------

        #[ink::test]
        fn constructor_test() {
            let context = Context::new();

            assert_eq!(context.contract.administrator, context.admin);
            assert_eq!(context.contract.contributors.len(), 0);
        }

        #[ink::test]
        fn add_contributor_test() {
            let mut context = Context::new();

            set_caller::<DefaultEnvironment>(context.admin);
            assert_eq!(context.contract.add_contributor(context.user0), Ok(()));
            assert_eq!(
                context.contract.add_contributor(context.user0),
                Err(Error::ContributorAlreadyExists)
            );
            assert_eq!(
                context.contract.add_contributor(context.admin),
                Err(Error::AdminCannotBeContributor)
            );

            // user0 is contributor
            set_caller::<DefaultEnvironment>(context.user0);
            assert_eq!(
                context.contract.add_contributor(context.user1),
                Err(Error::AdministrativeFunction)
            );

            // user1 is not contributor
            set_caller::<DefaultEnvironment>(context.user1);
            assert_eq!(
                context.contract.add_contributor(context.user0),
                Err(Error::AdministrativeFunction)
            );
        }

        #[ink::test]
        fn rem_contributor_test() {
            let mut context = Context::new();

            set_caller::<DefaultEnvironment>(context.admin);
            let _ = context.contract.add_contributor(context.user0);

            set_caller::<DefaultEnvironment>(context.user0);
            assert_eq!(
                context.contract.rem_contributor(context.user1),
                Err(Error::AdministrativeFunction)
            );

            set_caller::<DefaultEnvironment>(context.admin);
            assert_eq!(context.contract.rem_contributor(context.user0), Ok(()));
            assert_eq!(
                context.contract.rem_contributor(context.user0),
                Err(Error::ContributorNotExist)
            );
            assert_eq!(
                context.contract.rem_contributor(context.user1),
                Err(Error::ContributorNotExist)
            );
            assert_eq!(
                context.contract.rem_contributor(context.admin),
                Err(Error::AdminCannotBeContributor)
            );
        }

        #[ink::test]
        fn submit_vote_test() {
            let mut context = Context::new();

            set_caller::<DefaultEnvironment>(context.admin);
            assert_eq!(context.contract.contributors.len(), 0);
            assert_eq!(context.contract.add_contributor(context.user0), Ok(()));
            assert_eq!(context.contract.add_contributor(context.user1), Ok(()));
            assert_eq!(context.get_contributor(0).reputation, 0);
            assert_eq!(context.get_contributor(1).reputation, 0);
            assert_eq!(
                context.contract.submit_vote(context.user0),
                Err(Error::AdminCannotSubmitOrReceivedVote)
            );

            set_caller::<DefaultEnvironment>(context.user0);
            assert_eq!(context.contract.submit_vote(context.user1), Ok(()));
            assert_eq!(context.get_contributor(0).reputation, 0);
            assert_eq!(context.get_contributor(1).reputation, 1);
            assert_eq!(context.contract.submit_vote(context.user1), Ok(()));
            assert_eq!(context.get_contributor(0).reputation, 0);
            assert_eq!(context.get_contributor(1).reputation, 2);
            assert_eq!(
                context.contract.submit_vote(context.user0),
                Err(Error::CannotVoteItself)
            );
            assert_eq!(
                context.contract.submit_vote(context.admin),
                Err(Error::AdminCannotSubmitOrReceivedVote)
            );

            set_caller::<DefaultEnvironment>(context.user1);
            assert_eq!(context.contract.submit_vote(context.user0), Ok(()));
            assert_eq!(context.get_contributor(0).reputation, 1);
            assert_eq!(context.get_contributor(1).reputation, 2);

            set_caller::<DefaultEnvironment>(context.user0);
            assert_eq!(
                context.contract.submit_vote(context.user2),
                Err(Error::ContributorNotExist)
            );

            set_caller::<DefaultEnvironment>(context.user2);
            assert_eq!(
                context.contract.submit_vote(context.user0),
                Err(Error::YouAreNotContributor)
            );
        }

        #[ink::test]
        fn get_reputation_test() {
            let mut context = Context::new();

            set_caller::<DefaultEnvironment>(context.admin);
            assert_eq!(context.contract.contributors.len(), 0);
            assert_eq!(context.contract.add_contributor(context.user0), Ok(()));
            assert_eq!(context.contract.add_contributor(context.user1), Ok(()));

            set_caller::<DefaultEnvironment>(context.user0);
            assert_eq!(context.contract.submit_vote(context.user1), Ok(()));
            assert_eq!(context.contract.get_reputation(context.user0), Ok(0));
            assert_eq!(context.contract.get_reputation(context.user1), Ok(1));
            assert_eq!(context.contract.submit_vote(context.user1), Ok(()));
            assert_eq!(context.contract.get_reputation(context.user0), Ok(0));
            assert_eq!(context.contract.get_reputation(context.user1), Ok(2));
            assert_eq!(
                context.contract.get_reputation(context.user2),
                Err(Error::ContributorNotExist)
            );

            set_caller::<DefaultEnvironment>(context.user1);
            assert_eq!(context.contract.get_reputation(context.user0), Ok(0));
            assert_eq!(context.contract.get_reputation(context.user1), Ok(2));
            assert_eq!(
                context.contract.get_reputation(context.user2),
                Err(Error::ContributorNotExist)
            );

            set_caller::<DefaultEnvironment>(context.admin);
            assert_eq!(context.contract.get_reputation(context.user0), Ok(0));
            assert_eq!(context.contract.get_reputation(context.user1), Ok(2));
            assert_eq!(
                context.contract.get_reputation(context.user2),
                Err(Error::ContributorNotExist)
            );

            set_caller::<DefaultEnvironment>(context.user2);
            assert_eq!(context.contract.get_reputation(context.user0), Ok(0));
            assert_eq!(context.contract.get_reputation(context.user1), Ok(2));
            assert_eq!(
                context.contract.get_reputation(context.user2),
                Err(Error::ContributorNotExist)
            );
        }
    }

    /////////////////////////////////////////////////////////////////////////////////////

    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = FlipperRef::default();

            // When
            let contract_account_id = client
                .instantiate("flipper", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<FlipperRef>(contract_account_id.clone())
                .call(|flipper| flipper.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = FlipperRef::new(false);
            let contract_account_id = client
                .instantiate("flipper", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<FlipperRef>(contract_account_id.clone())
                .call(|flipper| flipper.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<FlipperRef>(contract_account_id.clone())
                .call(|flipper| flipper.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<FlipperRef>(contract_account_id.clone())
                .call(|flipper| flipper.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
