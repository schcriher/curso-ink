#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod organization {
    // necessary for a warning from "cargo contract build" (for non-use of StorageLayout) 🤷🏼
    #[allow(unused_imports)]
    use ink::storage::{traits::StorageLayout, Mapping};
    use scale::{Decode, Encode};

    // ------------------------------------
    // DATA TYPES

    pub type Reputation = u32;

    /// Contributor information
    #[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
    pub struct Contributor {
        /// Reputation of the contributor: sum of votes received
        reputation: Reputation,
    }

    // ------------------------------------
    // ERRORS

    /// Possible erroneous results
    #[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
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

    // ------------------------------------
    // EVENTS

    /// Voting event
    #[ink(event)]
    pub struct Vote {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
    }

    // ------------------------------------
    // CONTRACT

    #[ink(storage)]
    pub struct Organization {
        /// Administrator wallet, is who can add or remove contributors
        administrator: AccountId,
        /// List of contributors with their information
        contributors: Mapping<AccountId, Contributor>,
    }

    impl Organization {
        /// Constructor initializes the `administrator` and an empty map of `contributors`
        #[ink(constructor)]
        pub fn new(administrator: AccountId) -> Self {
            Self {
                administrator,
                contributors: Mapping::default(),
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

            if !self.contributors.contains(emitter_id) {
                return Err(Error::YouAreNotContributor);
            }

            if !self.contributors.contains(receiver_id) {
                return Err(Error::ContributorNotExist);
            }

            // unwrap is safe here
            let emitter = self.contributors.get(emitter_id).unwrap();
            let mut receiver = self.contributors.get(receiver_id).unwrap();

            // FIXME: temporary implementation, until the business logic is better defined
            let sum = if emitter.reputation < 10 { 1 } else { 2 };

            receiver.reputation += sum;
            self.contributors.insert(receiver_id, &receiver); // update

            self.env().emit_event(Vote {
                from: emitter_id,
                to: receiver_id,
            });

            Ok(())
        }

        /// Getting the reputation of a contributor, from whom it is consulted
        #[ink(message)]
        pub fn get_reputation(&self) -> Result<Reputation, Error> {
            let caller_id = self.env().caller();

            if !self.contributors.contains(caller_id) {
                return Err(Error::YouAreNotContributor);
            }

            let contributor = self.contributors.get(caller_id).unwrap(); // unwrap is safe here
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
            contract: Organization,
            admin: AccountId,
            user0: AccountId,
            user1: AccountId,
            user2: AccountId,
        }

        impl Context {
            fn new() -> Self {
                let admin = AccountId::from([u8::MAX; 32]);
                let contract = Organization::new(admin);

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

            fn add_contributor(&mut self, account_id: AccountId, reputation: Reputation) {
                self.contract
                    .contributors
                    .insert(account_id, &Contributor { reputation });
            }

            fn get_reputation(&self, account_id: AccountId) -> Reputation {
                self.contract
                    .contributors
                    .get(account_id)
                    .unwrap()
                    .reputation
            }

            fn contains(&self, account_id: AccountId) -> bool {
                self.contract.contributors.contains(account_id)
            }
        }

        // ------------------------------------------------------------------------------

        #[ink::test]
        fn constructor_test() {
            let ctx = Context::new();

            assert_eq!(ctx.contract.administrator, ctx.admin);

            assert!(!ctx.contains(ctx.admin));
            assert!(!ctx.contains(ctx.user0));
            assert!(!ctx.contains(ctx.user1));
            assert!(!ctx.contains(ctx.user2));
        }

        #[ink::test]
        fn add_contributor_test() {
            let mut ctx = Context::new();

            // admin
            set_caller::<DefaultEnvironment>(ctx.admin);
            assert!(!ctx.contains(ctx.user0));
            assert_eq!(ctx.contract.add_contributor(ctx.user0), Ok(()));
            assert!(ctx.contains(ctx.user0));
            assert_eq!(ctx.get_reputation(ctx.user0), 0);

            assert_eq!(
                ctx.contract.add_contributor(ctx.user0),
                Err(Error::ContributorAlreadyExists)
            );

            assert_eq!(
                ctx.contract.add_contributor(ctx.admin),
                Err(Error::AdminCannotBeContributor)
            );
            assert!(!ctx.contains(ctx.admin));

            // user0 is contributor
            set_caller::<DefaultEnvironment>(ctx.user0);
            assert_eq!(
                ctx.contract.add_contributor(ctx.user1),
                Err(Error::AdministrativeFunction)
            );
            assert!(!ctx.contains(ctx.user1));

            // user1 is not contributor
            set_caller::<DefaultEnvironment>(ctx.user1);
            assert_eq!(
                ctx.contract.add_contributor(ctx.user2),
                Err(Error::AdministrativeFunction)
            );
            assert!(!ctx.contains(ctx.user2));
        }

        #[ink::test]
        fn rem_contributor_test() {
            let mut ctx = Context::new();
            ctx.add_contributor(ctx.user0, 0);

            set_caller::<DefaultEnvironment>(ctx.user0);
            assert_eq!(
                ctx.contract.rem_contributor(ctx.user1),
                Err(Error::AdministrativeFunction)
            );
            assert!(ctx.contains(ctx.user0));

            set_caller::<DefaultEnvironment>(ctx.admin);
            assert_eq!(ctx.contract.rem_contributor(ctx.user0), Ok(()));
            assert!(!ctx.contains(ctx.user0));

            assert_eq!(
                ctx.contract.rem_contributor(ctx.user0),
                Err(Error::ContributorNotExist)
            );

            assert_eq!(
                ctx.contract.rem_contributor(ctx.user1),
                Err(Error::ContributorNotExist)
            );

            assert_eq!(
                ctx.contract.rem_contributor(ctx.admin),
                Err(Error::AdminCannotBeContributor)
            );
        }

        #[ink::test]
        fn submit_vote_test() {
            let mut ctx = Context::new();
            ctx.add_contributor(ctx.user0, 0);
            ctx.add_contributor(ctx.user1, 0);

            set_caller::<DefaultEnvironment>(ctx.admin);
            assert_eq!(
                ctx.contract.submit_vote(ctx.user0),
                Err(Error::AdminCannotSubmitOrReceivedVote)
            );

            set_caller::<DefaultEnvironment>(ctx.user0);
            assert_eq!(ctx.contract.submit_vote(ctx.user1), Ok(()));
            assert_eq!(ctx.get_reputation(ctx.user0), 0);
            assert_eq!(ctx.get_reputation(ctx.user1), 1);
            assert_eq!(ctx.contract.submit_vote(ctx.user1), Ok(()));
            assert_eq!(ctx.get_reputation(ctx.user0), 0);
            assert_eq!(ctx.get_reputation(ctx.user1), 2);

            assert_eq!(
                ctx.contract.submit_vote(ctx.user0),
                Err(Error::CannotVoteItself)
            );
            assert_eq!(ctx.get_reputation(ctx.user0), 0);

            assert_eq!(
                ctx.contract.submit_vote(ctx.user2),
                Err(Error::ContributorNotExist)
            );

            assert_eq!(
                ctx.contract.submit_vote(ctx.admin),
                Err(Error::AdminCannotSubmitOrReceivedVote)
            );

            set_caller::<DefaultEnvironment>(ctx.user1);
            assert_eq!(ctx.contract.submit_vote(ctx.user0), Ok(()));
            assert_eq!(ctx.get_reputation(ctx.user0), 1);
            assert_eq!(ctx.get_reputation(ctx.user1), 2);

            set_caller::<DefaultEnvironment>(ctx.user2);
            assert_eq!(
                ctx.contract.submit_vote(ctx.user0),
                Err(Error::YouAreNotContributor)
            );
            assert_eq!(ctx.get_reputation(ctx.user0), 1);
        }

        #[ink::test]
        fn get_reputation_test() {
            let mut ctx = Context::new();
            ctx.add_contributor(ctx.user0, 1);
            ctx.add_contributor(ctx.user1, 2);

            set_caller::<DefaultEnvironment>(ctx.user0);
            assert_eq!(ctx.contract.get_reputation(), Ok(1));

            set_caller::<DefaultEnvironment>(ctx.user1);
            assert_eq!(ctx.contract.get_reputation(), Ok(2));

            set_caller::<DefaultEnvironment>(ctx.user2);
            assert_eq!(
                ctx.contract.get_reputation(),
                Err(Error::YouAreNotContributor)
            );
        }

        #[ink::test]
        fn get_reputation_levels_test() {
            let mut ctx = Context::new();
            ctx.add_contributor(ctx.user0, 10);
            ctx.add_contributor(ctx.user1, 9);
            ctx.add_contributor(ctx.user2, 0);

            set_caller::<DefaultEnvironment>(ctx.user0);
            assert_eq!(ctx.contract.submit_vote(ctx.user2), Ok(()));
            set_caller::<DefaultEnvironment>(ctx.user2);
            assert_eq!(ctx.contract.get_reputation(), Ok(2)); // +2 (user0 rep>=10)

            set_caller::<DefaultEnvironment>(ctx.user1);
            assert_eq!(ctx.contract.submit_vote(ctx.user2), Ok(()));
            set_caller::<DefaultEnvironment>(ctx.user2);
            assert_eq!(ctx.contract.get_reputation(), Ok(3)); // +1 (user1 rep<10)
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
