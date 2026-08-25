use runen_online::{
    AdmissionGrantState, AssociationOutcome, AssignmentResolutionOutcome, AssignmentState, Authority,
    AuthorityDomainHandle, AuthorityError, AuthorityLimits, EndOutcome, InvalidInputKind,
    LogicalDestinationHandle, MatchRequestState, RedemptionOutcome, ResourceLimit, TimeDomainHandle,
    TrustedTime,
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

fn authority() -> Authority {
    authority_with_limits(limits())
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
fn independent_authorities_have_non_equivalent_semantic_ids() {
    let mut left = authority();
    let mut right = authority();

    let left_player = left.create_player().unwrap();
    let right_player = right.create_player().unwrap();

    assert_eq!(left_player.local_value(), right_player.local_value());
    assert_ne!(left_player, right_player);
    assert_ne!(left.domain(), right.domain());
}

#[test]
fn trusted_external_principal_resolution_is_scoped_and_unambiguous() {
    let mut authority = authority();
    let first = authority.create_player().unwrap();
    let second = authority.create_player().unwrap();

    assert_eq!(
        authority.accept_verified_external_principal(b"untrusted", b"subject"),
        Err(AuthorityError::UntrustedExternalAuthority)
    );

    let principal = authority
        .accept_verified_external_principal(b"issuer", b"subject")
        .unwrap();
    assert_eq!(authority.resolve_principal(&principal), Ok(None));
    assert_eq!(
        authority.associate_principal(&first, &principal),
        Ok(AssociationOutcome::Associated)
    );
    assert_eq!(
        authority.associate_principal(&first, &principal),
        Ok(AssociationOutcome::AlreadyAssociated)
    );
    assert_eq!(authority.resolve_principal(&principal), Ok(Some(first)));
    assert_eq!(
        authority.associate_principal(&second, &principal),
        Err(AuthorityError::Conflict)
    );
}

#[test]
fn principal_representation_and_association_limits_are_enforced() {
    let mut configured = limits();
    configured.max_external_authority_bytes = 6;
    configured.max_external_subject_bytes = 3;
    configured.max_principal_associations = 1;
    configured.max_principal_associations_per_player = 1;
    let mut authority = authority_with_limits(configured);
    let first = authority.create_player().unwrap();
    let second = authority.create_player().unwrap();

    assert_eq!(
        authority.accept_verified_external_principal(b"issuer", b"four"),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::ExternalSubjectBytes
        ))
    );

    let one = authority
        .accept_verified_external_principal(b"issuer", b"one")
        .unwrap();
    let two = authority
        .accept_verified_external_principal(b"issuer", b"two")
        .unwrap();
    authority.associate_principal(&first, &one).unwrap();
    assert_eq!(
        authority.associate_principal(&first, &two),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::PrincipalAssociationsPerPlayer
        ))
    );
    assert_eq!(
        authority.associate_principal(&second, &two),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::PrincipalAssociations
        ))
    );
}

#[test]
fn trusted_authority_configuration_is_bounded_before_activation() {
    let mut configured = limits();
    configured.max_trusted_external_authorities = 1;
    assert_eq!(
        Authority::new(
            AuthorityDomainHandle::new(),
            TimeDomainHandle::new(),
            configured,
            [b"issuer".as_slice(), b"other".as_slice()],
        )
        .err(),
        Some(AuthorityError::ResourceLimit(
            ResourceLimit::TrustedExternalAuthorities
        ))
    );
}

#[test]
fn player_capacity_rejects_without_advancing_successful_identity_sequence() {
    let mut configured = limits();
    configured.max_players = 2;
    let mut authority = authority_with_limits(configured);

    let first = authority.create_player().unwrap();
    let second = authority.create_player().unwrap();
    assert_eq!(first.local_value(), 1);
    assert_eq!(second.local_value(), 2);
    assert_eq!(
        authority.create_player(),
        Err(AuthorityError::ResourceLimit(ResourceLimit::Players))
    );
}

#[test]
fn pending_assignment_resolves_once_and_expiry_is_irreversible() {
    let mut authority = authority();
    let now = at(&authority, 1);
    let assignment = authority.establish_pending_assignment(&now, 5).unwrap();
    let destination = LogicalDestinationHandle::new(7);

    let before = at(&authority, 4);
    assert_eq!(
        authority.resolve_assignment(&assignment, destination, &before),
        Ok(AssignmentResolutionOutcome::Resolved)
    );
    assert_eq!(
        authority.resolve_assignment(&assignment, destination, &before),
        Ok(AssignmentResolutionOutcome::AlreadyUsable)
    );
    assert_eq!(
        authority.resolve_assignment(
            &assignment,
            LogicalDestinationHandle::new(8),
            &before,
        ),
        Err(AuthorityError::Conflict)
    );

    let mut expiry_authority = authority();
    let now = at(&expiry_authority, 1);
    let expired = expiry_authority
        .establish_pending_assignment(&now, 5)
        .unwrap();
    let boundary = at(&expiry_authority, 5);
    assert_eq!(
        expiry_authority.resolve_assignment(&expired, destination, &boundary),
        Err(AuthorityError::Expired)
    );
    let lower_later_observation = at(&expiry_authority, 2);
    assert_eq!(
        expiry_authority
            .assignment(&expired, &lower_later_observation)
            .unwrap()
            .state(),
        AssignmentState::Ended
    );
}

#[test]
fn assignment_end_and_resolution_have_one_terminal_winner_in_either_order() {
    let destination = LogicalDestinationHandle::new(7);

    let mut end_first = authority();
    let now = at(&end_first, 1);
    let pending = end_first.establish_pending_assignment(&now, 10).unwrap();
    assert_eq!(end_first.end_assignment(&pending), Ok(EndOutcome::Ended));
    let later = at(&end_first, 2);
    assert_eq!(
        end_first.resolve_assignment(&pending, destination, &later),
        Err(AuthorityError::Terminal)
    );

    let mut resolve_first = authority();
    let now = at(&resolve_first, 1);
    let pending = resolve_first.establish_pending_assignment(&now, 10).unwrap();
    let later = at(&resolve_first, 2);
    assert_eq!(
        resolve_first.resolve_assignment(&pending, destination, &later),
        Ok(AssignmentResolutionOutcome::Resolved)
    );
    assert_eq!(resolve_first.end_assignment(&pending), Ok(EndOutcome::Ended));
}

#[test]
fn pending_assignment_lifetime_and_time_domain_fail_closed() {
    let mut configured = limits();
    configured.max_pending_assignment_lifetime = 5;
    let mut authority = authority_with_limits(configured);
    let now = at(&authority, 1);

    assert_eq!(
        authority.establish_pending_assignment(&now, 7),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::PendingAssignmentLifetime
        ))
    );

    let wrong_time = TrustedTime::new(TimeDomainHandle::new(), 1);
    assert_eq!(
        authority.establish_pending_assignment(&wrong_time, 3),
        Err(AuthorityError::TimeDomainMismatch)
    );
}

#[test]
fn admission_requires_usable_assignment_and_fixes_exact_binding() {
    let mut authority = authority();
    let player = authority.create_player().unwrap();
    let now = at(&authority, 1);
    let pending = authority.establish_pending_assignment(&now, 10).unwrap();

    assert_eq!(
        authority.issue_admission_grant(&player, &pending, &now, 5),
        Err(AuthorityError::NotUsable)
    );

    let usable = authority
        .establish_usable_assignment(LogicalDestinationHandle::new(1))
        .unwrap();
    let grant = authority
        .issue_admission_grant(&player, &usable, &now, 5)
        .unwrap();
    let view = authority.admission_grant(&grant, &now).unwrap();
    assert_eq!(view.player(), &player);
    assert_eq!(view.assignment(), &usable);
    assert_eq!(
        view.state(),
        AdmissionGrantState::Redeemable { deadline: 5 }
    );
}

#[test]
fn admission_redeems_once_and_success_survives_assignment_end() {
    let mut authority = authority();
    let player = authority.create_player().unwrap();
    let assignment = authority
        .establish_usable_assignment(LogicalDestinationHandle::new(1))
        .unwrap();
    let now = at(&authority, 1);
    let grant = authority
        .issue_admission_grant(&player, &assignment, &now, 10)
        .unwrap();
    let redeem = at(&authority, 2);

    assert_eq!(
        authority.redeem_admission_grant(&grant, &redeem),
        Ok(RedemptionOutcome::Redeemed)
    );
    assert_eq!(
        authority.redeem_admission_grant(&grant, &redeem),
        Ok(RedemptionOutcome::AlreadyRedeemed)
    );
    authority.end_assignment(&assignment).unwrap();
    assert_eq!(
        authority.admission_grant(&grant, &redeem).unwrap().state(),
        AdmissionGrantState::Redeemed
    );
}

#[test]
fn assignment_end_before_redemption_permanently_blocks_grant() {
    let mut authority = authority();
    let player = authority.create_player().unwrap();
    let assignment = authority
        .establish_usable_assignment(LogicalDestinationHandle::new(1))
        .unwrap();
    let now = at(&authority, 1);
    let grant = authority
        .issue_admission_grant(&player, &assignment, &now, 10)
        .unwrap();

    authority.end_assignment(&assignment).unwrap();
    let later = at(&authority, 2);
    assert_eq!(
        authority.redeem_admission_grant(&grant, &later),
        Err(AuthorityError::NotUsable)
    );
    assert_eq!(
        authority.admission_grant(&grant, &later).unwrap().state(),
        AdmissionGrantState::AssignmentEnded
    );
}

#[test]
fn grant_expiry_is_irreversible() {
    let mut authority = authority();
    let player = authority.create_player().unwrap();
    let assignment = authority
        .establish_usable_assignment(LogicalDestinationHandle::new(1))
        .unwrap();
    let now = at(&authority, 1);
    let grant = authority
        .issue_admission_grant(&player, &assignment, &now, 5)
        .unwrap();

    let boundary = at(&authority, 5);
    assert_eq!(
        authority.redeem_admission_grant(&grant, &boundary),
        Err(AuthorityError::Expired)
    );
    let lower = at(&authority, 2);
    assert_eq!(
        authority.admission_grant(&grant, &lower).unwrap().state(),
        AdmissionGrantState::Expired
    );
}

#[test]
fn live_grant_quota_reconciles_lazy_expiry_before_rejecting_new_work() {
    let mut configured = limits();
    configured.max_live_admission_grants_per_player = 1;
    configured.max_live_admission_grants_per_assignment = 1;
    let mut authority = authority_with_limits(configured);
    let player = authority.create_player().unwrap();
    let assignment = authority
        .establish_usable_assignment(LogicalDestinationHandle::new(1))
        .unwrap();
    let now = at(&authority, 1);
    authority
        .issue_admission_grant(&player, &assignment, &now, 5)
        .unwrap();

    // No explicit read/redeem of the old grant occurs. Issuance itself must
    // reconcile its lazy expiry before applying the live fan-out quota.
    let boundary = at(&authority, 5);
    assert!(authority
        .issue_admission_grant(&player, &assignment, &boundary, 10)
        .is_ok());
}

#[test]
fn live_grant_fanout_is_bounded_per_player_and_assignment() {
    let mut configured = limits();
    configured.max_live_admission_grants_per_player = 1;
    configured.max_live_admission_grants_per_assignment = 1;
    let mut authority = authority_with_limits(configured);
    let first_player = authority.create_player().unwrap();
    let second_player = authority.create_player().unwrap();
    let first_assignment = authority
        .establish_usable_assignment(LogicalDestinationHandle::new(1))
        .unwrap();
    let second_assignment = authority
        .establish_usable_assignment(LogicalDestinationHandle::new(2))
        .unwrap();
    let now = at(&authority, 1);

    authority
        .issue_admission_grant(&first_player, &first_assignment, &now, 10)
        .unwrap();
    assert_eq!(
        authority.issue_admission_grant(&first_player, &second_assignment, &now, 10),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::LiveAdmissionGrantsPerPlayer
        ))
    );
    assert_eq!(
        authority.issue_admission_grant(&second_player, &first_assignment, &now, 10),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::LiveAdmissionGrantsPerAssignment
        ))
    );
}

#[test]
fn admission_lifetime_and_retained_capacity_are_explicit() {
    let mut configured = limits();
    configured.max_admission_grants = 1;
    configured.max_admission_grant_lifetime = 5;
    let mut authority = authority_with_limits(configured);
    let player = authority.create_player().unwrap();
    let assignment = authority
        .establish_usable_assignment(LogicalDestinationHandle::new(1))
        .unwrap();
    let now = at(&authority, 1);

    assert_eq!(
        authority.issue_admission_grant(&player, &assignment, &now, 7),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::AdmissionGrantLifetime
        ))
    );
    let grant = authority
        .issue_admission_grant(&player, &assignment, &now, 6)
        .unwrap();
    let later = at(&authority, 2);
    authority.redeem_admission_grant(&grant, &later).unwrap();
    assert_eq!(
        authority.issue_admission_grant(&player, &assignment, &later, 6),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::AdmissionGrants
        ))
    );
}

#[test]
fn match_request_is_immutable_bounded_and_same_domain() {
    let mut authority = authority();
    let first = authority.create_player().unwrap();
    let second = authority.create_player().unwrap();
    let now = at(&authority, 1);
    let request = authority
        .establish_match_request(&[first.clone(), second.clone()], b"ranked", &now, 10)
        .unwrap();
    let view = authority.match_request(&request, &now).unwrap();

    assert_eq!(view.cohort(), &[first, second]);
    assert_eq!(view.matching_input(), b"ranked");
    assert_eq!(view.state(), MatchRequestState::Pending { deadline: 10 });
}

#[test]
fn match_request_rejects_empty_duplicate_and_oversized_input() {
    let mut configured = limits();
    configured.max_match_request_cohort = 1;
    configured.max_matchmaking_input_bytes = 3;
    let mut authority = authority_with_limits(configured);
    let player = authority.create_player().unwrap();
    let other = authority.create_player().unwrap();
    let now = at(&authority, 1);

    assert_eq!(
        authority.establish_match_request(&[], b"", &now, 10),
        Err(AuthorityError::InvalidInput(
            InvalidInputKind::EmptyMatchCohort
        ))
    );
    assert_eq!(
        authority.establish_match_request(&[player.clone(), player.clone()], b"", &now, 10),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::MatchRequestCohort
        ))
    );
    assert_eq!(
        authority.establish_match_request(&[player.clone(), other], b"", &now, 10),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::MatchRequestCohort
        ))
    );
    assert_eq!(
        authority.establish_match_request(&[player], b"four", &now, 10),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::MatchmakingInputBytes
        ))
    );
}

#[test]
fn pending_request_quota_reconciles_lazy_expiry_before_rejecting_new_work() {
    let mut configured = limits();
    configured.max_pending_match_requests_per_player = 1;
    let mut authority = authority_with_limits(configured);
    let player = authority.create_player().unwrap();
    let now = at(&authority, 1);
    authority
        .establish_match_request(std::slice::from_ref(&player), b"first", &now, 5)
        .unwrap();

    // No explicit observation of the old request occurs before establishing a
    // new one. The quota check must materialize the reached deadline itself.
    let boundary = at(&authority, 5);
    assert!(authority
        .establish_match_request(std::slice::from_ref(&player), b"next", &boundary, 10)
        .is_ok());
}

#[test]
fn match_commit_is_all_or_nothing_exact_and_immutable() {
    let mut authority = authority();
    let a = authority.create_player().unwrap();
    let b = authority.create_player().unwrap();
    let now = at(&authority, 1);
    let left = authority
        .establish_match_request(std::slice::from_ref(&a), b"a", &now, 10)
        .unwrap();
    let right = authority
        .establish_match_request(std::slice::from_ref(&b), b"b", &now, 10)
        .unwrap();
    let commit_time = at(&authority, 2);
    let matched = authority
        .commit_match(&[left.clone(), right.clone()], &commit_time)
        .unwrap();

    assert_eq!(
        authority.match_request(&left, &commit_time).unwrap().state(),
        MatchRequestState::Matched(matched.clone())
    );
    assert_eq!(
        authority.match_request(&right, &commit_time).unwrap().state(),
        MatchRequestState::Matched(matched.clone())
    );
    let view = authority.committed_match(&matched).unwrap();
    assert_eq!(view.roster(), &[a, b]);
    assert_eq!(view.contributions().len(), 2);
    assert_eq!(view.contributions()[0].request_id(), &left);
    assert_eq!(view.contributions()[1].request_id(), &right);
}

#[test]
fn duplicate_and_overlapping_candidates_reject_without_consumption() {
    let mut authority = authority();
    let a = authority.create_player().unwrap();
    let b = authority.create_player().unwrap();
    let c = authority.create_player().unwrap();
    let now = at(&authority, 1);
    let left = authority
        .establish_match_request(&[a, b.clone()], b"", &now, 10)
        .unwrap();
    let right = authority
        .establish_match_request(&[b, c], b"", &now, 10)
        .unwrap();
    let commit_time = at(&authority, 2);

    assert_eq!(
        authority.commit_match(&[left.clone(), left.clone()], &commit_time),
        Err(AuthorityError::InvalidInput(
            InvalidInputKind::DuplicateMatchRequest
        ))
    );
    assert_eq!(
        authority.commit_match(&[left.clone(), right.clone()], &commit_time),
        Err(AuthorityError::InvalidInput(
            InvalidInputKind::OverlappingPlayer
        ))
    );
    assert!(matches!(
        authority.match_request(&left, &commit_time).unwrap().state(),
        MatchRequestState::Pending { .. }
    ));
    assert!(matches!(
        authority.match_request(&right, &commit_time).unwrap().state(),
        MatchRequestState::Pending { .. }
    ));
}

#[test]
fn expired_match_candidate_materializes_expiry_but_consumes_none() {
    let mut authority = authority();
    let a = authority.create_player().unwrap();
    let b = authority.create_player().unwrap();
    let now = at(&authority, 1);
    let expired = authority
        .establish_match_request(std::slice::from_ref(&a), b"", &now, 5)
        .unwrap();
    let live = authority
        .establish_match_request(std::slice::from_ref(&b), b"", &now, 10)
        .unwrap();
    let boundary = at(&authority, 5);

    assert_eq!(
        authority.commit_match(&[expired.clone(), live.clone()], &boundary),
        Err(AuthorityError::Expired)
    );
    assert_eq!(
        authority.match_request(&expired, &boundary).unwrap().state(),
        MatchRequestState::Ended
    );
    assert_eq!(
        authority.match_request(&live, &boundary).unwrap().state(),
        MatchRequestState::Pending { deadline: 10 }
    );
}

#[test]
fn match_request_end_and_commit_have_one_terminal_winner_in_either_order() {
    let mut end_first = authority();
    let player = end_first.create_player().unwrap();
    let now = at(&end_first, 1);
    let request = end_first
        .establish_match_request(std::slice::from_ref(&player), b"", &now, 10)
        .unwrap();
    assert_eq!(
        end_first.end_match_request(&request),
        Ok(EndOutcome::Ended)
    );
    let later = at(&end_first, 2);
    assert_eq!(
        end_first.commit_match(std::slice::from_ref(&request), &later),
        Err(AuthorityError::Terminal)
    );

    let mut commit_first = authority();
    let player = commit_first.create_player().unwrap();
    let now = at(&commit_first, 1);
    let request = commit_first
        .establish_match_request(std::slice::from_ref(&player), b"", &now, 10)
        .unwrap();
    let later = at(&commit_first, 2);
    let matched = commit_first
        .commit_match(std::slice::from_ref(&request), &later)
        .unwrap();
    assert_eq!(
        commit_first.end_match_request(&request),
        Ok(EndOutcome::AlreadyTerminal)
    );
    assert_eq!(
        commit_first.match_request(&request, &later).unwrap().state(),
        MatchRequestState::Matched(matched)
    );
}

#[test]
fn overlapping_match_candidates_have_exactly_one_winner_in_either_order() {
    fn run(commit_ab_first: bool) -> (MatchRequestState, MatchRequestState, MatchRequestState) {
        let mut authority = authority();
        let a = authority.create_player().unwrap();
        let b = authority.create_player().unwrap();
        let c = authority.create_player().unwrap();
        let now = at(&authority, 1);
        let a_req = authority
            .establish_match_request(std::slice::from_ref(&a), b"", &now, 10)
            .unwrap();
        let b_req = authority
            .establish_match_request(std::slice::from_ref(&b), b"", &now, 10)
            .unwrap();
        let c_req = authority
            .establish_match_request(std::slice::from_ref(&c), b"", &now, 10)
            .unwrap();
        let later = at(&authority, 2);

        if commit_ab_first {
            authority
                .commit_match(&[a_req.clone(), b_req.clone()], &later)
                .unwrap();
            assert_eq!(
                authority.commit_match(&[b_req.clone(), c_req.clone()], &later),
                Err(AuthorityError::Terminal)
            );
        } else {
            authority
                .commit_match(&[b_req.clone(), c_req.clone()], &later)
                .unwrap();
            assert_eq!(
                authority.commit_match(&[a_req.clone(), b_req.clone()], &later),
                Err(AuthorityError::Terminal)
            );
        }

        (
            authority.match_request(&a_req, &later).unwrap().state(),
            authority.match_request(&b_req, &later).unwrap().state(),
            authority.match_request(&c_req, &later).unwrap().state(),
        )
    }

    let first = run(true);
    assert!(matches!(first.0, MatchRequestState::Matched(_)));
    assert!(matches!(first.1, MatchRequestState::Matched(_)));
    assert!(matches!(first.2, MatchRequestState::Pending { .. }));

    let second = run(false);
    assert!(matches!(second.0, MatchRequestState::Pending { .. }));
    assert!(matches!(second.1, MatchRequestState::Matched(_)));
    assert!(matches!(second.2, MatchRequestState::Matched(_)));
}

#[test]
fn match_candidate_roster_and_retained_match_limits_are_enforced() {
    let mut configured = limits();
    configured.max_match_candidate_requests = 1;
    configured.max_match_roster_players = 1;
    configured.max_matches = 1;
    let mut authority = authority_with_limits(configured);
    let a = authority.create_player().unwrap();
    let b = authority.create_player().unwrap();
    let now = at(&authority, 1);
    let a_req = authority
        .establish_match_request(std::slice::from_ref(&a), b"", &now, 10)
        .unwrap();
    let b_req = authority
        .establish_match_request(std::slice::from_ref(&b), b"", &now, 10)
        .unwrap();
    let later = at(&authority, 2);

    assert_eq!(
        authority.commit_match(&[a_req.clone(), b_req.clone()], &later),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::MatchCandidateRequests
        ))
    );
    authority
        .commit_match(std::slice::from_ref(&a_req), &later)
        .unwrap();
    assert_eq!(
        authority.commit_match(std::slice::from_ref(&b_req), &later),
        Err(AuthorityError::ResourceLimit(ResourceLimit::Matches))
    );
}

#[test]
fn match_request_lifetime_and_retained_capacity_are_bounded() {
    let mut configured = limits();
    configured.max_match_requests = 1;
    configured.max_match_request_lifetime = 5;
    let mut authority = authority_with_limits(configured);
    let player = authority.create_player().unwrap();
    let now = at(&authority, 1);

    assert_eq!(
        authority.establish_match_request(std::slice::from_ref(&player), b"", &now, 7),
        Err(AuthorityError::ResourceLimit(
            ResourceLimit::MatchRequestLifetime
        ))
    );
    authority
        .establish_match_request(std::slice::from_ref(&player), b"", &now, 6)
        .unwrap();
    assert_eq!(
        authority.establish_match_request(std::slice::from_ref(&player), b"", &now, 6),
        Err(AuthorityError::ResourceLimit(ResourceLimit::MatchRequests))
    );
}

#[test]
fn direct_assignment_path_requires_no_matchmaking_objects() {
    let mut authority = authority();
    let player = authority.create_player().unwrap();
    let assignment = authority
        .establish_usable_assignment(LogicalDestinationHandle::new(99))
        .unwrap();
    let now = at(&authority, 1);
    let grant = authority
        .issue_admission_grant(&player, &assignment, &now, 5)
        .unwrap();
    assert_eq!(
        authority.redeem_admission_grant(&grant, &now),
        Ok(RedemptionOutcome::Redeemed)
    );
}

#[test]
fn cross_domain_objects_fail_before_lookup_or_mutation() {
    let mut left = authority();
    let mut right = authority();
    let right_player = right.create_player().unwrap();
    let left_assignment = left
        .establish_usable_assignment(LogicalDestinationHandle::new(1))
        .unwrap();
    let now = at(&left, 1);

    assert_eq!(
        left.issue_admission_grant(&right_player, &left_assignment, &now, 5),
        Err(AuthorityError::AuthorityDomainMismatch)
    );
}
