use runen_online::{
    AdmissionGrantState, AssociationOutcome, AssignmentResolutionOutcome, AssignmentState, Authority,
    AuthorityDomainHandle, AuthorityError, AuthorityLimits, EndOutcome, InvalidInputKind,
    LogicalDestinationHandle, MatchRequestState, ObjectKind, RedemptionOutcome, ResourceLimit,
    TimeDomainHandle, TrustedTime,
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
        max_admission_grant_lifetime: 100,
        max_live_admission_grants_per_player: 4,
        max_live_admission_grants_per_assignment: 8,
        max_match_requests: 64,
        max_match_request_lifetime: 100,
        max_match_request_cohort: 8,
        max_matchmaking_input_bytes: 128,
        max_pending_match_requests_per_player: 4,
        max_match_candidate_requests: 8,
        max_match_roster_players: 32,
        max_matches: 32,
    }
}

fn authority_with_limits(
    domain: u64,
    time_domain: u64,
    limits: AuthorityLimits,
) -> Authority {
    Authority::new(
        AuthorityDomainHandle::new(domain),
        TimeDomainHandle::new(time_domain),
        limits,
        [b"issuer".as_slice()],
    )
    .unwrap()
}

fn authority(domain: u64, time_domain: u64) -> Authority {
    authority_with_limits(domain, time_domain, limits())
}

fn time(domain: u64, value: u64) -> TrustedTime {
    TrustedTime::new(TimeDomainHandle::new(domain), value)
}

#[test]
fn identity_scope_and_principal_association_are_unambiguous() {
    let mut left = authority(1, 10);
    let mut right = authority(2, 20);

    let left_player = left.create_player().unwrap();
    let right_player = right.create_player().unwrap();
    assert_eq!(left_player.local_value(), right_player.local_value());
    assert_ne!(left_player, right_player);

    let principal = left
        .accept_verified_external_principal(b"issuer", b"alice")
        .unwrap();
    assert_eq!(
        left.associate_principal(left_player, &principal),
        Ok(AssociationOutcome::Associated)
    );
    assert_eq!(
        left.associate_principal(left_player, &principal),
        Ok(AssociationOutcome::AlreadyAssociated)
    );
    assert_eq!(left.resolve_principal(&principal), Ok(Some(left_player)));

    let other_player = left.create_player().unwrap();
    assert_eq!(
        left.associate_principal(other_player, &principal),
        Err(AuthorityError::Conflict)
    );
    assert_eq!(
        right.resolve_principal(&principal),
        Err(AuthorityError::AuthorityDomainMismatch)
    );
    assert_eq!(
        left.accept_verified_external_principal(b"unknown", b"alice"),
        Err(AuthorityError::UntrustedExternalAuthority)
    );
}

#[test]
fn assignment_supports_direct_and_pending_paths_without_matchmaking() {
    let mut authority = authority(1, 10);
    let destination = LogicalDestinationHandle::new(7);

    let direct = authority.establish_usable_assignment(destination).unwrap();
    assert_eq!(
        authority.assignment(direct, time(10, 1)).unwrap().state(),
        AssignmentState::Usable { destination }
    );

    let pending = authority
        .establish_pending_assignment(time(10, 5), 10)
        .unwrap();
    assert_eq!(
        authority.resolve_assignment(pending, destination, time(10, 9)),
        Ok(AssignmentResolutionOutcome::Resolved)
    );
    assert_eq!(
        authority.resolve_assignment(pending, destination, time(10, 9)),
        Ok(AssignmentResolutionOutcome::AlreadyUsable)
    );
    assert_eq!(
        authority.resolve_assignment(
            pending,
            LogicalDestinationHandle::new(8),
            time(10, 9)
        ),
        Err(AuthorityError::Conflict)
    );
}

#[test]
fn assignment_expiry_is_irreversible_and_stale_identity_cannot_replace_new_state() {
    let mut authority = authority(1, 10);
    let destination = LogicalDestinationHandle::new(7);
    let old = authority
        .establish_pending_assignment(time(10, 1), 5)
        .unwrap();

    assert_eq!(
        authority.resolve_assignment(old, destination, time(10, 5)),
        Err(AuthorityError::Expired)
    );
    assert_eq!(
        authority.assignment(old, time(10, 2)).unwrap().state(),
        AssignmentState::Ended
    );

    let replacement = authority
        .establish_usable_assignment(LogicalDestinationHandle::new(8))
        .unwrap();
    assert_ne!(old, replacement);
    assert_eq!(
        authority.resolve_assignment(old, destination, time(10, 3)),
        Err(AuthorityError::Terminal)
    );
    assert_eq!(
        authority
            .assignment(replacement, time(10, 3))
            .unwrap()
            .state(),
        AssignmentState::Usable {
            destination: LogicalDestinationHandle::new(8)
        }
    );
}

#[test]
fn time_domain_mismatch_fails_closed() {
    let mut authority = authority(1, 10);
    assert_eq!(
        authority.establish_pending_assignment(time(11, 1), 5),
        Err(AuthorityError::TimeDomainMismatch)
    );

    let assignment = authority
        .establish_pending_assignment(time(10, 1), 5)
        .unwrap();
    assert_eq!(
        authority.assignment(assignment, time(11, 2)),
        Err(AuthorityError::TimeDomainMismatch)
    );
}

#[test]
fn grant_is_single_redemption_and_assignment_bound() {
    let mut authority = authority(1, 10);
    let player = authority.create_player().unwrap();
    let assignment = authority
        .establish_usable_assignment(LogicalDestinationHandle::new(1))
        .unwrap();
    let grant = authority
        .issue_admission_grant(player, assignment, time(10, 1), 10)
        .unwrap();

    assert_eq!(
        authority.redeem_admission_grant(grant, time(10, 2)),
        Ok(RedemptionOutcome::Redeemed)
    );
    assert_eq!(
        authority.redeem_admission_grant(grant, time(10, 3)),
        Ok(RedemptionOutcome::AlreadyRedeemed)
    );
    authority.end_assignment(assignment).unwrap();
    assert_eq!(
        authority.admission_grant(grant, time(10, 4)).unwrap().state(),
        AdmissionGrantState::Redeemed
    );
}

#[test]
fn assignment_end_blocks_only_unredeemed_grants() {
    let mut authority = authority(1, 10);
    let player = authority.create_player().unwrap();
    let assignment = authority
        .establish_usable_assignment(LogicalDestinationHandle::new(1))
        .unwrap();
    let grant = authority
        .issue_admission_grant(player, assignment, time(10, 1), 10)
        .unwrap();

    assert_eq!(authority.end_assignment(assignment), Ok(EndOutcome::Ended));
    assert_eq!(
        authority.admission_grant(grant, time(10, 2)).unwrap().state(),
        AdmissionGrantState::AssignmentEnded
    );
    assert_eq!(
        authority.redeem_admission_grant(grant, time(10, 2)),
        Err(AuthorityError::NotUsable)
    );
}

#[test]
fn grant_expiry_materializes_without_successful_redemption_and_never_revives() {
    let mut authority = authority(1, 10);
    let player = authority.create_player().unwrap();
    let assignment = authority
        .establish_usable_assignment(LogicalDestinationHandle::new(1))
        .unwrap();
    let grant = authority
        .issue_admission_grant(player, assignment, time(10, 1), 5)
        .unwrap();

    assert_eq!(
        authority.redeem_admission_grant(grant, time(10, 5)),
        Err(AuthorityError::Expired)
    );
    assert_eq!(
        authority.admission_grant(grant, time(10, 2)).unwrap().state(),
        AdmissionGrantState::Expired
    );
}

#[test]
fn live_grant_fanout_does_not_count_redeemed_history() {
    let mut configured = limits();
    configured.max_live_admission_grants_per_player = 1;
    configured.max_live_admission_grants_per_assignment = 1;
    let mut authority = authority_with_limits(1, 10, configured);
    let player = authority.create_player().unwrap();
    let assignment = authority
        .establish_usable_assignment(LogicalDestinationHandle::new(1))
        .unwrap();

    let first = authority
        .issue_admission_grant(player, assignment, time(10, 1), 10)
        .unwrap();
    assert_eq!(
        authority.issue_admission_grant(player, assignment, time(10, 1), 10),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::LiveAdmissionGrantsPerPlayer
        ))
    );
    authority
        .redeem_admission_grant(first, time(10, 2))
        .unwrap();
    assert!(authority
        .issue_admission_grant(player, assignment, time(10, 2), 10)
        .is_ok());
}

#[test]
fn matchmaking_request_is_finite_immutable_and_terminal() {
    let mut authority = authority(1, 10);
    let first = authority.create_player().unwrap();
    let second = authority.create_player().unwrap();

    assert_eq!(
        authority.establish_match_request(&[], b"", time(10, 1), 10),
        Err(AuthorityError::InvalidInput(
            InvalidInputKind::EmptyMatchCohort
        ))
    );
    assert_eq!(
        authority.establish_match_request(&[first, first], b"", time(10, 1), 10),
        Err(AuthorityError::InvalidInput(
            InvalidInputKind::DuplicatePlayer
        ))
    );

    let request = authority
        .establish_match_request(&[first, second], b"mode-a", time(10, 1), 5)
        .unwrap();
    let before = authority.match_request(request, time(10, 4)).unwrap();
    assert_eq!(before.cohort(), &[first, second]);
    assert_eq!(before.matching_input(), b"mode-a");
    assert_eq!(
        before.state(),
        MatchRequestState::Pending { deadline: 5 }
    );

    assert_eq!(
        authority.match_request(request, time(10, 5)).unwrap().state(),
        MatchRequestState::Ended
    );
    assert_eq!(
        authority.match_request(request, time(10, 2)).unwrap().state(),
        MatchRequestState::Ended
    );
}

#[test]
fn match_commit_is_atomic_exact_and_immutable() {
    let mut authority = authority(1, 10);
    let a = authority.create_player().unwrap();
    let b = authority.create_player().unwrap();
    let c = authority.create_player().unwrap();
    let request_a = authority
        .establish_match_request(&[a, b], b"one", time(10, 1), 20)
        .unwrap();
    let request_b = authority
        .establish_match_request(&[c], b"two", time(10, 1), 20)
        .unwrap();

    let match_id = authority
        .commit_match(&[request_a, request_b], time(10, 2))
        .unwrap();
    let committed = authority.committed_match(match_id).unwrap();
    assert_eq!(committed.roster(), &[a, b, c]);
    assert_eq!(committed.contributions().len(), 2);
    assert_eq!(committed.contributions()[0].request_id(), request_a);
    assert_eq!(committed.contributions()[0].cohort(), &[a, b]);
    assert_eq!(committed.contributions()[1].request_id(), request_b);
    assert_eq!(committed.contributions()[1].cohort(), &[c]);
    assert_eq!(
        authority
            .match_request(request_a, time(10, 3))
            .unwrap()
            .state(),
        MatchRequestState::Matched(match_id)
    );
    assert_eq!(
        authority.commit_match(&[request_a], time(10, 3)),
        Err(AuthorityError::Terminal)
    );
}

#[test]
fn overlapping_cohorts_reject_without_consuming_requests() {
    let mut authority = authority(1, 10);
    let a = authority.create_player().unwrap();
    let b = authority.create_player().unwrap();
    let c = authority.create_player().unwrap();
    let left = authority
        .establish_match_request(&[a, b], b"", time(10, 1), 20)
        .unwrap();
    let right = authority
        .establish_match_request(&[b, c], b"", time(10, 1), 20)
        .unwrap();

    assert_eq!(
        authority.commit_match(&[left, right], time(10, 2)),
        Err(AuthorityError::InvalidInput(
            InvalidInputKind::OverlappingPlayer
        ))
    );
    assert!(matches!(
        authority.match_request(left, time(10, 2)).unwrap().state(),
        MatchRequestState::Pending { .. }
    ));
    assert!(matches!(
        authority.match_request(right, time(10, 2)).unwrap().state(),
        MatchRequestState::Pending { .. }
    ));
}

#[test]
fn expired_candidate_materializes_expiry_but_consumes_none() {
    let mut authority = authority(1, 10);
    let a = authority.create_player().unwrap();
    let b = authority.create_player().unwrap();
    let expired = authority
        .establish_match_request(&[a], b"", time(10, 1), 5)
        .unwrap();
    let still_pending = authority
        .establish_match_request(&[b], b"", time(10, 1), 10)
        .unwrap();

    assert_eq!(
        authority.commit_match(&[expired, still_pending], time(10, 5)),
        Err(AuthorityError::Expired)
    );
    assert_eq!(
        authority.match_request(expired, time(10, 2)).unwrap().state(),
        MatchRequestState::Ended
    );
    assert_eq!(
        authority
            .match_request(still_pending, time(10, 2))
            .unwrap()
            .state(),
        MatchRequestState::Pending { deadline: 10 }
    );
}

#[test]
fn overlapping_match_candidates_have_exactly_one_winner_in_either_order() {
    fn run(first_ab: bool) -> (MatchRequestState, MatchRequestState, MatchRequestState) {
        let mut authority = authority(1, 10);
        let a = authority.create_player().unwrap();
        let b = authority.create_player().unwrap();
        let c = authority.create_player().unwrap();
        let a_req = authority
            .establish_match_request(&[a], b"", time(10, 1), 20)
            .unwrap();
        let b_req = authority
            .establish_match_request(&[b], b"", time(10, 1), 20)
            .unwrap();
        let c_req = authority
            .establish_match_request(&[c], b"", time(10, 1), 20)
            .unwrap();

        if first_ab {
            authority.commit_match(&[a_req, b_req], time(10, 2)).unwrap();
            assert_eq!(
                authority.commit_match(&[b_req, c_req], time(10, 2)),
                Err(AuthorityError::Terminal)
            );
        } else {
            authority.commit_match(&[b_req, c_req], time(10, 2)).unwrap();
            assert_eq!(
                authority.commit_match(&[a_req, b_req], time(10, 2)),
                Err(AuthorityError::Terminal)
            );
        }

        (
            authority.match_request(a_req, time(10, 2)).unwrap().state(),
            authority.match_request(b_req, time(10, 2)).unwrap().state(),
            authority.match_request(c_req, time(10, 2)).unwrap().state(),
        )
    }

    let (a_first, b_first, c_first) = run(true);
    assert!(matches!(a_first, MatchRequestState::Matched(_)));
    assert!(matches!(b_first, MatchRequestState::Matched(_)));
    assert!(matches!(c_first, MatchRequestState::Pending { .. }));

    let (a_second, b_second, c_second) = run(false);
    assert!(matches!(a_second, MatchRequestState::Pending { .. }));
    assert!(matches!(b_second, MatchRequestState::Matched(_)));
    assert!(matches!(c_second, MatchRequestState::Matched(_)));
}

#[test]
fn identity_and_principal_limits_enforce_exact_boundaries() {
    let mut configured = limits();
    configured.max_trusted_external_authorities = 1;
    configured.max_external_authority_bytes = 6;
    configured.max_external_subject_bytes = 3;
    configured.max_players = 2;
    configured.max_principal_associations = 1;
    configured.max_principal_associations_per_player = 1;

    assert_eq!(
        Authority::new(
            AuthorityDomainHandle::new(1),
            TimeDomainHandle::new(10),
            configured.clone(),
            [b"issuer".as_slice(), b"other".as_slice()],
        )
        .err(),
        Some(AuthorityError::ResourceLimit(
            ResourceLimit::TrustedExternalAuthorities
        ))
    );

    let mut authority = authority_with_limits(1, 10, configured);
    let first = authority.create_player().unwrap();
    let second = authority.create_player().unwrap();
    assert_eq!(
        authority.create_player(),
        Err(AuthorityError::ResourceLimit(ResourceLimit::Players))
    );
    assert_eq!(
        authority.accept_verified_external_principal(b"issuer", b"abcd"),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::ExternalSubjectBytes
        ))
    );

    let first_principal = authority
        .accept_verified_external_principal(b"issuer", b"one")
        .unwrap();
    authority
        .associate_principal(first, &first_principal)
        .unwrap();
    let second_principal = authority
        .accept_verified_external_principal(b"issuer", b"two")
        .unwrap();
    assert_eq!(
        authority.associate_principal(second, &second_principal),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::PrincipalAssociations
        ))
    );
}

#[test]
fn assignment_limits_and_invalid_deadline_do_not_consume_identity() {
    let mut configured = limits();
    configured.max_assignments = 1;
    configured.max_pending_assignment_lifetime = 5;
    let mut authority = authority_with_limits(1, 10, configured);

    assert_eq!(
        authority.establish_pending_assignment(time(10, 5), 5),
        Err(AuthorityError::InvalidDeadline)
    );
    assert_eq!(
        authority.establish_pending_assignment(time(10, 5), 11),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::PendingAssignmentLifetime
        ))
    );
    let accepted = authority
        .establish_pending_assignment(time(10, 5), 10)
        .unwrap();
    assert_eq!(accepted.local_value(), 1);
    assert_eq!(
        authority.establish_usable_assignment(LogicalDestinationHandle::new(1)),
        Err(AuthorityError::ResourceLimit(ResourceLimit::Assignments))
    );
}

#[test]
fn admission_limits_cover_lifetime_live_fanout_and_retained_capacity() {
    let mut configured = limits();
    configured.max_admission_grants = 1;
    configured.max_admission_grant_lifetime = 5;
    configured.max_live_admission_grants_per_player = 1;
    configured.max_live_admission_grants_per_assignment = 1;
    let mut authority = authority_with_limits(1, 10, configured);
    let player = authority.create_player().unwrap();
    let other = authority.create_player().unwrap();
    let assignment = authority
        .establish_usable_assignment(LogicalDestinationHandle::new(1))
        .unwrap();

    assert_eq!(
        authority.issue_admission_grant(player, assignment, time(10, 1), 7),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::AdmissionGrantLifetime
        ))
    );
    let grant = authority
        .issue_admission_grant(player, assignment, time(10, 1), 6)
        .unwrap();
    assert_eq!(
        authority.issue_admission_grant(other, assignment, time(10, 1), 6),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::AdmissionGrants
        ))
    );
    authority
        .redeem_admission_grant(grant, time(10, 2))
        .unwrap();
    assert_eq!(
        authority.issue_admission_grant(other, assignment, time(10, 2), 6),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::AdmissionGrants
        ))
    );
}

#[test]
fn match_request_limits_cover_cohort_input_pending_and_retained_capacity() {
    let mut configured = limits();
    configured.max_match_requests = 2;
    configured.max_match_request_lifetime = 5;
    configured.max_match_request_cohort = 2;
    configured.max_matchmaking_input_bytes = 2;
    configured.max_pending_match_requests_per_player = 1;
    let mut authority = authority_with_limits(1, 10, configured);
    let a = authority.create_player().unwrap();
    let b = authority.create_player().unwrap();
    let c = authority.create_player().unwrap();

    assert_eq!(
        authority.establish_match_request(&[a], b"", time(10, 1), 7),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::MatchRequestLifetime
        ))
    );
    assert_eq!(
        authority.establish_match_request(&[a, b, c], b"", time(10, 1), 6),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::MatchRequestCohort
        ))
    );
    assert_eq!(
        authority.establish_match_request(&[a], b"abc", time(10, 1), 6),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::MatchmakingInputBytes
        ))
    );

    let first = authority
        .establish_match_request(&[a, b], b"ab", time(10, 1), 6)
        .unwrap();
    assert_eq!(
        authority.establish_match_request(&[a], b"", time(10, 1), 6),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::PendingMatchRequestsPerPlayer
        ))
    );
    assert_eq!(authority.end_match_request(first), Ok(EndOutcome::Ended));
    authority
        .establish_match_request(&[a], b"", time(10, 1), 6)
        .unwrap();
    assert_eq!(
        authority.establish_match_request(&[c], b"", time(10, 1), 6),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::MatchRequests
        ))
    );
}

#[test]
fn match_commit_limits_cover_candidate_roster_and_retained_matches() {
    let mut configured = limits();
    configured.max_match_candidate_requests = 1;
    configured.max_match_roster_players = 2;
    configured.max_matches = 1;
    let mut authority = authority_with_limits(1, 10, configured);
    let a = authority.create_player().unwrap();
    let b = authority.create_player().unwrap();
    let c = authority.create_player().unwrap();
    let a_req = authority
        .establish_match_request(&[a], b"", time(10, 1), 20)
        .unwrap();
    let b_req = authority
        .establish_match_request(&[b], b"", time(10, 1), 20)
        .unwrap();

    assert_eq!(
        authority.commit_match(&[], time(10, 2)),
        Err(AuthorityError::InvalidInput(
            InvalidInputKind::EmptyMatchCandidate
        ))
    );
    assert_eq!(
        authority.commit_match(&[a_req, b_req], time(10, 2)),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::MatchCandidateRequests
        ))
    );

    authority.commit_match(&[a_req], time(10, 2)).unwrap();
    assert_eq!(
        authority.commit_match(&[b_req], time(10, 2)),
        Err(AuthorityError::ResourceLimit(ResourceLimit::Matches))
    );

    let mut roster_limited = limits();
    roster_limited.max_match_roster_players = 2;
    let mut other = authority_with_limits(2, 20, roster_limited);
    let x = other.create_player().unwrap();
    let y = other.create_player().unwrap();
    let z = other.create_player().unwrap();
    let too_large = other
        .establish_match_request(&[x, y, z], b"", time(20, 1), 20)
        .unwrap();
    assert_eq!(
        other.commit_match(&[too_large], time(20, 2)),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::MatchRosterPlayers
        ))
    );
    assert!(matches!(
        other
            .match_request(too_large, time(20, 2))
            .unwrap()
            .state(),
        MatchRequestState::Pending { .. }
    ));
}

#[test]
fn cross_domain_objects_are_rejected_before_lookup() {
    let mut left = authority(1, 10);
    let mut right = authority(2, 10);
    let foreign_player = left.create_player().unwrap();

    assert_eq!(
        right.establish_match_request(&[foreign_player], b"", time(10, 1), 5),
        Err(AuthorityError::AuthorityDomainMismatch)
    );

    let local_player = right.create_player().unwrap();
    let local_assignment = right
        .establish_usable_assignment(LogicalDestinationHandle::new(1))
        .unwrap();
    let grant = right
        .issue_admission_grant(local_player, local_assignment, time(10, 1), 5)
        .unwrap();
    assert_eq!(
        left.redeem_admission_grant(grant, time(10, 2)),
        Err(AuthorityError::AuthorityDomainMismatch)
    );
}

#[test]
fn unknown_objects_remain_distinct_from_cross_domain_objects() {
    let mut authority = authority(1, 10);
    let other = authority(1, 10);
    let unknown = other.domain();
    assert_eq!(unknown, authority.domain());

    let player = authority.create_player().unwrap();
    let assignment = authority
        .establish_usable_assignment(LogicalDestinationHandle::new(1))
        .unwrap();
    assert!(authority
        .issue_admission_grant(player, assignment, time(10, 1), 5)
        .is_ok());

    // A second authority can legitimately allocate the same local value in the
    // same host-supplied domain handle. Hosts must therefore give distinct
    // authority incarnations distinct handles when equality must be isolated.
    let mut second = authority(1, 10);
    let second_player = second.create_player().unwrap();
    assert_eq!(player, second_player);
    assert_eq!(
        authority.resolve_principal(
            &second
                .accept_verified_external_principal(b"issuer", b"absent")
                .unwrap()
        ),
        Ok(None)
    );

    assert_eq!(
        authority.assignment(
            runen_online::AssignmentId::from_parts_for_test_only_not_public(),
            time(10, 2)
        ),
        Err(AuthorityError::Unknown(ObjectKind::Assignment))
    );
}
