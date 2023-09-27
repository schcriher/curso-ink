use ink::env::{DefaultEnvironment, Environment};
use scale::{Decode, Encode};

pub type AccountId = <DefaultEnvironment as Environment>::AccountId;
pub type Timestamp = <DefaultEnvironment as Environment>::Timestamp;
pub type Balance = <DefaultEnvironment as Environment>::Balance;
pub type String = ink::prelude::string::String;

/// Identifier of a round, sequential numbers, starting at one
pub type RoundId = u32;

/// Contributor reputation, starting at one
pub type Reputation = u32;

/// Number of votes
pub type VotesNumber = u8;

/// Member role
#[derive(Debug, PartialEq, Eq, Clone, Copy, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum Role {
    Admin,
    Contributor,
}

/// Information on a contributor's reputation in a specific round
#[derive(Debug, PartialEq, Eq, Clone, Copy, Encode, Decode, Default)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Contributor {
    pub round_id: RoundId,
    pub reputation: Reputation,
    pub votes_submitted: VotesNumber,
}

/// Voting sign, positive adds, negative subtracts,
/// the final sum or subtraction depends on the reputations of the sender and the receiver of votes
#[derive(Debug, PartialEq, Eq, Clone, Copy, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum VoteSign {
    Positive,
    Negative,
}

/// Information on the vote to emit
#[derive(Debug, PartialEq, Eq, Clone, Copy, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Vote {
    pub sign: VoteSign,
    pub value: VotesNumber,
}

/// Information on a round
#[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Round {
    /// Name of the round
    pub name: String,
    /// Funds to be distributed in the round
    pub value: Balance,
    /// Maximum number of votes per contributor
    pub max_votes: VotesNumber,
    /// End date of the round (timestamp), in milliseconds
    pub finish_at: Timestamp,
    /// Establishes whether the distribution was completed
    pub is_finished: bool,
}
