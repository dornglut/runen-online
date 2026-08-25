#![forbid(unsafe_code)]

//! Standalone provider-neutral RunenOnline semantic core.
//!
//! The normative specification under the repository `spec/` tree remains the
//! authority for portable RunenOnline behavior. Public Rust representations in
//! this provisional crate are implementation choices and do not standardize
//! wire, storage, provider, or service contracts.

mod admission;
mod assignment;
mod authority;
mod error;
mod identity;
mod limits;
mod matchmaking;
mod time;

pub use admission::{AdmissionGrantState, AdmissionGrantView, RedemptionOutcome};
pub use assignment::{
    AssignmentResolutionOutcome, AssignmentState, AssignmentView, LogicalDestinationHandle,
};
pub use authority::{AssociationOutcome, Authority, EndOutcome};
pub use error::{AuthorityError, IdKind, InvalidInputKind, ObjectKind, ResourceLimit};
pub use identity::{
    AdmissionGrantId, AssignmentId, AuthorityDomainHandle, MatchId, MatchRequestId, PlayerId,
    VerifiedExternalPrincipal,
};
pub use limits::AuthorityLimits;
pub use matchmaking::{MatchContribution, MatchRequestState, MatchRequestView, MatchView};
pub use time::{TimeDomainHandle, TrustedTime};
