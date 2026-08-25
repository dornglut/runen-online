use runen_online::{
    Authority, AuthorityDomainHandle, AuthorityError, AuthorityLimits, LogicalDestinationHandle,
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

fn at(authority: &Authority, value: u64) -> TrustedTime {
    TrustedTime::new(authority.time_domain(), value)
}

#[test]
fn trusted_authority_count_and_bytes_cover_exact_and_one_over() {
    let mut configured = limits();
    configured.max_trusted_external_authorities = 1;
    configured.max_external_authority_bytes = 6;

    assert!(Authority::new(
        AuthorityDomainHandle::new(),
        TimeDomainHandle::new(),
        configured.clone(),
        [b"issuer".as_slice()],
    )
    .is_ok());

    assert_eq!(
        Authority::new(
            AuthorityDomainHandle::new(),
            TimeDomainHandle::new(),
            configured.clone(),
            [b"issuer".as_slice(), b"other".as_slice()],
        )
        .err(),
        Some(AuthorityError::ResourceLimit(
            ResourceLimit::TrustedExternalAuthorities
        ))
    );

    assert_eq!(
        Authority::new(
            AuthorityDomainHandle::new(),
            TimeDomainHandle::new(),
            configured,
            [b"toolong".as_slice()],
        )
        .err(),
        Some(AuthorityError::ResourceLimit(
            ResourceLimit::ExternalAuthorityBytes
        ))
    );
}

#[test]
fn assignment_retention_and_lifetime_cover_exact_and_one_over() {
    let mut configured = limits();
    configured.max_assignments = 1;
    configured.max_pending_assignment_lifetime = 5;
    let mut authority = authority_with_limits(configured);
    let now = at(&authority, 1);

    assert_eq!(
        authority.establish_pending_assignment(&now, 7),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::PendingAssignmentLifetime
        ))
    );

    authority.establish_pending_assignment(&now, 6).unwrap();
    assert_eq!(
        authority.establish_usable_assignment(LogicalDestinationHandle::new(1)),
        Err(AuthorityError::ResourceLimit(ResourceLimit::Assignments))
    );
}

#[test]
fn match_request_shape_and_pending_quota_cover_exact_and_one_over() {
    let mut configured = limits();
    configured.max_match_request_cohort = 2;
    configured.max_matchmaking_input_bytes = 3;
    configured.max_pending_match_requests_per_player = 1;
    let mut authority = authority_with_limits(configured);
    let first = authority.create_player().unwrap();
    let second = authority.create_player().unwrap();
    let third = authority.create_player().unwrap();
    let now = at(&authority, 1);

    assert_eq!(
        authority.establish_match_request(
            &[first.clone(), second.clone(), third.clone()],
            b"",
            &now,
            10,
        ),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::MatchRequestCohort
        ))
    );
    assert_eq!(
        authority.establish_match_request(std::slice::from_ref(&third), b"four", &now, 10),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::MatchmakingInputBytes
        ))
    );

    authority
        .establish_match_request(&[first.clone(), second], b"abc", &now, 10)
        .unwrap();
    assert_eq!(
        authority.establish_match_request(std::slice::from_ref(&first), b"", &now, 10),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::PendingMatchRequestsPerPlayer
        ))
    );
}

#[test]
fn match_roster_limit_covers_exact_and_one_over() {
    let mut configured = limits();
    configured.max_match_candidate_requests = 2;
    configured.max_match_roster_players = 1;

    let mut exact = authority_with_limits(configured.clone());
    let player = exact.create_player().unwrap();
    let now = at(&exact, 1);
    let request = exact
        .establish_match_request(std::slice::from_ref(&player), b"", &now, 10)
        .unwrap();
    let later = at(&exact, 2);
    exact
        .commit_match(std::slice::from_ref(&request), &later)
        .unwrap();

    let mut overflow = authority_with_limits(configured);
    let first = overflow.create_player().unwrap();
    let second = overflow.create_player().unwrap();
    let now = at(&overflow, 1);
    let first_request = overflow
        .establish_match_request(std::slice::from_ref(&first), b"", &now, 10)
        .unwrap();
    let second_request = overflow
        .establish_match_request(std::slice::from_ref(&second), b"", &now, 10)
        .unwrap();
    let later = at(&overflow, 2);

    assert_eq!(
        overflow.commit_match(&[first_request, second_request], &later),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::MatchRosterPlayers
        ))
    );
}
