/// Coarse object category used by provisional implementation errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectKind {
    Player,
    Assignment,
    AdmissionGrant,
    MatchRequest,
    Match,
}

/// Semantic identity allocator whose finite local space was exhausted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdKind {
    Player,
    Assignment,
    AdmissionGrant,
    MatchRequest,
    Match,
}

/// Input-shape failures required to preserve deterministic semantic behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidInputKind {
    EmptyMatchCohort,
    EmptyMatchCandidate,
    DuplicatePlayer,
    DuplicateMatchRequest,
    OverlappingPlayer,
}

/// Explicit finite-policy boundary reached by the in-process RO2 realization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceLimit {
    TrustedExternalAuthorities,
    ExternalAuthorityBytes,
    ExternalSubjectBytes,
    Players,
    PrincipalAssociations,
    PrincipalAssociationsPerPlayer,
    Assignments,
    PendingAssignmentLifetime,
    AdmissionGrants,
    AdmissionGrantLifetime,
    LiveAdmissionGrantsPerPlayer,
    LiveAdmissionGrantsPerAssignment,
    MatchRequests,
    MatchRequestLifetime,
    MatchRequestCohort,
    MatchmakingInputBytes,
    PendingMatchRequestsPerPlayer,
    MatchCandidateRequests,
    MatchRosterPlayers,
    Matches,
}

/// Provisional Rust error surface for the deterministic RO2 implementation.
///
/// These variants are implementation diagnostics, not a normative portable
/// RunenOnline failure taxonomy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorityError {
    AuthorityDomainMismatch,
    TimeDomainMismatch,
    Unknown(ObjectKind),
    InvalidInput(InvalidInputKind),
    ResourceLimit(ResourceLimit),
    IdExhausted(IdKind),
    InvalidDeadline,
    Expired,
    Terminal,
    NotUsable,
    Conflict,
    UntrustedExternalAuthority,
}
