use runen_online::{
    AssignmentState, AssociationOutcome, Authority, AuthorityDomainHandle, AuthorityError,
    AuthorityLimits, InvalidInputKind, LogicalDestinationHandle, MatchRequestState,
    ResourceLimit, TimeDomainHandle, TrustedTime,
};

fn limits() -> AuthorityLimits {
    AuthorityLimits {
        max_trusted_external_authorities: 4,
        max_external_authority_bytes: 32,
        max_external_subject_bytes: 64,
        max_players: 32,
        max_principal_associations: 64,
        max_principal_associations_per_player: 4,
        max_assignments: 32,
        max_pending_assignment_lifetime: 100,
        max_admission_grants: 64,
        max_admission_grant_lifetime: 20,
        max_live_admission_grants_per_player: 4,
        max_live_admission_grants_per_assignment: 8,
        max_match_requests: 64,
        max_match_request_lifetime: 30,
        max_match_request_cohort: 8,
        max_matchmaking_input_bytes: 64,
        max_pending_match_requests_per_player: 4,
        max_match_candidate_requests: 8,
        max_match_roster_players: 16,
        max_matches: 32,
    }
}

fn authority_with_limits(configured: AuthorityLimits) -> Authority {
    Authority::new(
        AuthorityDomainHandle::new(),
        TimeDomainHandle::new(),
        configured,
        [b"issuer".as_slice()],
    )
    .unwrap()
}

fn authority() -> Authority {
    authority_with_limits(limits())
}

fn at(authority: &Authority, value: u64) -> TrustedTime {
    TrustedTime::new(authority.time_domain(), value)
}

#[test]
fn principal_conflict_preserves_existing_resolution() {
    let mut authority = authority();
    let first = authority.create_player().unwrap();
    let second = authority.create_player().unwrap();
    let principal = authority
        .accept_verified_external_principal(b"issuer", b"subject")
        .unwrap();

    assert_eq!(
        authority.associate_principal(&first, &principal),
        Ok(AssociationOutcome::Associated)
    );
    assert_eq!(
        authority.associate_principal(&second, &principal),
        Err(AuthorityError::Conflict)
    );
    assert_eq!(
        authority.resolve_principal(&principal),
        Ok(Some(first.clone()))
    );
}

#[test]
fn stale_assignment_resolution_cannot_mutate_replacement() {
    let mut authority = authority();
    let now = at(&authority, 1);
    let old = authority.establish_pending_assignment(&now, 10).unwrap();
    let replacement = authority.establish_pending_assignment(&now, 10).unwrap();
    let destination = LogicalDestinationHandle::new(7);
    let later = at(&authority, 2);

    authority
        .resolve_assignment(&old, destination, &later)
        .unwrap();
    assert_eq!(
        authority.assignment(&replacement, &later).unwrap().state(),
        AssignmentState::Pending { deadline: 10 }
    );
}

#[test]
fn conflicting_assignment_resolution_preserves_fixed_destination() {
    let mut authority = authority();
    let now = at(&authority, 1);
    let assignment = authority.establish_pending_assignment(&now, 10).unwrap();
    let fixed = LogicalDestinationHandle::new(7);
    let later = at(&authority, 2);

    authority
        .resolve_assignment(&assignment, fixed, &later)
        .unwrap();
    assert_eq!(
        authority.resolve_assignment(
            &assignment,
            LogicalDestinationHandle::new(8),
            &later,
        ),
        Err(AuthorityError::Conflict)
    );
    assert_eq!(
        authority.assignment(&assignment, &later).unwrap().state(),
        AssignmentState::Usable { destination: fixed }
    );
}

#[test]
fn cross_domain_match_request_fails_before_id_allocation() {
    let mut left = authority();
    let mut right = authority();
    let foreign_player = right.create_player().unwrap();
    let now = at(&left, 1);

    assert_eq!(
        left.establish_match_request(std::slice::from_ref(&foreign_player), b"", &now, 10),
        Err(AuthorityError::AuthorityDomainMismatch)
    );

    let local_player = left.create_player().unwrap();
    let request = left
        .establish_match_request(std::slice::from_ref(&local_player), b"", &now, 10)
        .unwrap();
    assert_eq!(request.local_value(), 1);
}

#[test]
fn match_request_expiry_is_irreversible_under_lower_observation() {
    let mut authority = authority();
    let player = authority.create_player().unwrap();
    let now = at(&authority, 1);
    let request = authority
        .establish_match_request(std::slice::from_ref(&player), b"", &now, 5)
        .unwrap();

    let boundary = at(&authority, 5);
    assert_eq!(
        authority.match_request(&request, &boundary).unwrap().state(),
        MatchRequestState::Ended
    );

    let lower = at(&authority, 2);
    assert_eq!(
        authority.match_request(&request, &lower).unwrap().state(),
        MatchRequestState::Ended
    );
}

#[test]
fn prevalidation_failures_do_not_consume_semantic_ids() {
    let mut configured = limits();
    configured.max_pending_assignment_lifetime = 5;
    configured.max_admission_grant_lifetime = 5;
    let mut authority = authority_with_limits(configured);
    let now = at(&authority, 1);

    assert_eq!(
        authority.establish_pending_assignment(&now, 7),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::PendingAssignmentLifetime
        ))
    );
    let assignment = authority
        .establish_usable_assignment(LogicalDestinationHandle::new(1))
        .unwrap();
    assert_eq!(assignment.local_value(), 1);

    let player = authority.create_player().unwrap();
    assert_eq!(
        authority.issue_admission_grant(&player, &assignment, &now, 7),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::AdmissionGrantLifetime
        ))
    );
    let grant = authority
        .issue_admission_grant(&player, &assignment, &now, 6)
        .unwrap();
    assert_eq!(grant.local_value(), 1);

    assert_eq!(
        authority.establish_match_request(&[], b"", &now, 6),
        Err(AuthorityError::InvalidInput(
            InvalidInputKind::EmptyMatchCohort
        ))
    );
    let request = authority
        .establish_match_request(std::slice::from_ref(&player), b"", &now, 6)
        .unwrap();
    assert_eq!(request.local_value(), 1);

    assert_eq!(
        authority.commit_match(&[], &now),
        Err(AuthorityError::InvalidInput(
            InvalidInputKind::EmptyMatchCandidate
        ))
    );
    let matched = authority
        .commit_match(std::slice::from_ref(&request), &now)
        .unwrap();
    assert_eq!(matched.local_value(), 1);
}
