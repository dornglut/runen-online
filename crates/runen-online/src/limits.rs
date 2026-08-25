/// Explicit finite policy for one bounded in-process RunenOnline authority.
///
/// Every field is implementation policy for this RO2 realization. No numeric
/// value is a normative RunenOnline constant, and zero is permitted to disable
/// the corresponding capability in a particular authority instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityLimits {
    pub max_trusted_external_authorities: usize,
    pub max_external_authority_bytes: usize,
    pub max_external_subject_bytes: usize,
    pub max_players: usize,
    pub max_principal_associations: usize,
    pub max_principal_associations_per_player: usize,
    pub max_assignments: usize,
    pub max_pending_assignment_lifetime: u64,
    pub max_admission_grants: usize,
    pub max_admission_grant_lifetime: u64,
    pub max_live_admission_grants_per_player: usize,
    pub max_live_admission_grants_per_assignment: usize,
    pub max_match_requests: usize,
    pub max_match_request_lifetime: u64,
    pub max_match_request_cohort: usize,
    pub max_matchmaking_input_bytes: usize,
    pub max_pending_match_requests_per_player: usize,
    pub max_match_candidate_requests: usize,
    pub max_match_roster_players: usize,
    pub max_matches: usize,
}
