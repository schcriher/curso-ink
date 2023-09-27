use scale::{Decode, Encode};

use crate::types::{AccountId, Balance, VotesNumber};

/// Possible erroneous results
#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    AdministrativeFunction,
    CannotRemoveYourself,

    MemberAlreadyExists,
    MemberNotExist,

    OnlyContributorCanVote,
    CannotVoteItself,
    YouAreNotContributor,

    InvalidRoundParameter,
    IsNoActiveRound,
    IsAnNoFinishedRound,
    NotYetFinishedRound,
    InsufficientFunds,

    MulOverflow(Balance, Balance),
    TransferFailed(AccountId, Balance),

    ExceedsVoteLimit(VotesNumber),
    ExceedsYourVoteLimit(VotesNumber),

    NftNotSent,
}
