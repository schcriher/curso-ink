use crate::errors::Error;
use crate::types::{AccountId, Reputation};

#[ink::trait_definition]
pub trait VoteTrait {
    /// Submit a vote, the caller gives the vote to `receiver_id`
    #[ink(message)]
    fn submit_vote(&mut self, receiver_id: AccountId) -> Result<(), Error>;

    /// Getting the reputation of a contributor, from whom it is consulted
    #[ink(message)]
    fn get_reputation(&self) -> Result<Reputation, Error>;
}
