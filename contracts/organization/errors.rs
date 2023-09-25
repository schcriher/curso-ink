use scale::{Decode, Encode};

use crate::types::VotesNumber;

/// Possible erroneous results
#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    AdministrativeFunction,
    AdminCannotBeContributor,

    MemberAlreadyExists,
    MemberNotExist,

    OnlyContributorCanVote,
    CannotVoteItself,
    YouAreNotContributor,

    IsNoActiveRound,
    ExceedsVoteLimit(VotesNumber),
    ExceedsYourVoteLimit(VotesNumber),

    NftNotSent,
}
