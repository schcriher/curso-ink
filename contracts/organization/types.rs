use scale::{Decode, Encode};

/// Contributor reputation
pub type Reputation = u32;

/// Contributor information
#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Contributor {
    /// Reputation of the contributor (for now: sum of votes received)
    pub reputation: Reputation,
}

/// Wallet identifier
pub type AccountId = ink::primitives::AccountId;
