use runen_online::{
    AdmissionGrantState, AssignmentResolutionOutcome, AssignmentState, AssociationOutcome,
    Authority, AuthorityDomainHandle, AuthorityError, AuthorityLimits, EndOutcome,
    LogicalDestinationHandle, RedemptionOutcome, TimeDomainHandle, TrustedTime,
};

const TRUSTED_ISSUER: &[u8] = b"example-issuer";

fn limits() -> AuthorityLimits {
    AuthorityLimits {
        max_trusted_external_authorities: 2,
        max_external_authority_bytes: 32,
        max_external_subject_bytes: 64,
        max_players: 8,
        max_principal_associations: 8,
        max_principal_associations_per_player: 2,
        max_assignments: 4,
        max_pending_assignment_lifetime: 100,
        max_admission_grants: 8,
        max_admission_grant_lifetime: 50,
        max_live_admission_grants_per_player: 2,
        max_live_admission_grants_per_assignment: 4,
        max_match_requests: 4,
        max_match_request_lifetime: 100,
        max_match_request_cohort: 2,
        max_matchmaking_input_bytes: 32,
        max_pending_match_requests_per_player: 2,
        max_match_candidate_requests: 2,
        max_match_roster_players: 4,
        max_matches: 2,
    }
}

fn new_authority() -> Result<Authority, AuthorityError> {
    // These domain handles, finite policy values, and trusted issuer are host
    // configuration. The example does not standardize them as RunenOnline
    // constants or provider identities.
    Authority::new(
        AuthorityDomainHandle::new(),
        TimeDomainHandle::new(),
        limits(),
        [TRUSTED_ISSUER],
    )
}

fn at(authority: &Authority, value: u64) -> TrustedTime {
    // The host owns the trusted comparison domain and supplies every explicit
    // observation. RunenOnline does not read a wall clock or run timers here.
    TrustedTime::new(authority.time_domain(), value)
}

fn direct_assignment_and_admission() -> Result<(), AuthorityError> {
    // This authority never creates MatchRequest or Match state. The scenario
    // therefore proves that direct Assignment/Admission consumption has no
    // matchmaking dependency.
    let mut authority = new_authority()?;
    let player = authority.create_player()?;

    // Credential verification is deliberately outside RunenOnline. The host
    // supplies evidence that it has already verified.
    let principal =
        authority.accept_verified_external_principal(TRUSTED_ISSUER, b"direct-player")?;
    assert_eq!(
        authority.associate_principal(&player, &principal)?,
        AssociationOutcome::Associated
    );
    assert_eq!(
        authority.resolve_principal(&principal)?,
        Some(player.clone())
    );

    // Destination selection and the decision to establish a directly Usable
    // Assignment are host/application policy.
    let destination = LogicalDestinationHandle::new(100);
    let assignment = authority.establish_usable_assignment(destination)?;

    let issue_time = at(&authority, 10);
    let grant = authority.issue_admission_grant(&player, &assignment, &issue_time, 30)?;
    let redeem_time = at(&authority, 11);
    assert_eq!(
        authority.redeem_admission_grant(&grant, &redeem_time)?,
        RedemptionOutcome::Redeemed
    );

    assert_eq!(authority.end_assignment(&assignment)?, EndOutcome::Ended);
    let after_end = at(&authority, 12);
    assert_eq!(
        authority.admission_grant(&grant, &after_end)?.state(),
        AdmissionGrantState::Redeemed
    );

    Ok(())
}

fn optional_matchmaking_composition() -> Result<(), AuthorityError> {
    let mut authority = new_authority()?;
    let first = authority.create_player()?;
    let second = authority.create_player()?;

    let request_time = at(&authority, 1);
    let first_request = authority.establish_match_request(
        std::slice::from_ref(&first),
        b"first-host-intent",
        &request_time,
        20,
    )?;
    let second_request = authority.establish_match_request(
        std::slice::from_ref(&second),
        b"second-host-intent",
        &request_time,
        20,
    )?;

    // Candidate selection belongs to the host. RunenOnline validates and
    // atomically commits the exact candidate; it does not choose or score it.
    let match_time = at(&authority, 2);
    let candidate = [first_request.clone(), second_request.clone()];
    let match_id = authority.commit_match(&candidate, &match_time)?;
    let committed = authority.committed_match(&match_id)?;

    assert_eq!(committed.id(), &match_id);
    assert_eq!(
        committed.roster(),
        [first.clone(), second.clone()].as_slice()
    );
    assert_eq!(committed.contributions().len(), 2);
    assert_eq!(committed.contributions()[0].request_id(), &first_request);
    assert_eq!(
        committed.contributions()[0].cohort(),
        std::slice::from_ref(&first)
    );
    assert_eq!(committed.contributions()[1].request_id(), &second_request);
    assert_eq!(
        committed.contributions()[1].cohort(),
        std::slice::from_ref(&second)
    );

    // This Match-to-Assignment composition is example host policy only. It is
    // not a RunenOnline cardinality rule or mandatory workflow.
    let pending_time = at(&authority, 3);
    let assignment = authority.establish_pending_assignment(&pending_time, 30)?;
    let chosen_destination = LogicalDestinationHandle::new(200);
    let placement_time = at(&authority, 4);
    assert_eq!(
        authority.resolve_assignment(&assignment, chosen_destination, &placement_time)?,
        AssignmentResolutionOutcome::Resolved
    );
    assert_eq!(
        authority.assignment(&assignment, &placement_time)?.state(),
        AssignmentState::Usable {
            destination: chosen_destination
        }
    );

    // Grant issuance is also host policy. The example chooses one grant per
    // matched player without making that a portable Match/Assignment rule.
    let first_grant = authority.issue_admission_grant(&first, &assignment, &placement_time, 25)?;
    let second_grant =
        authority.issue_admission_grant(&second, &assignment, &placement_time, 25)?;

    let admission_time = at(&authority, 5);
    assert_eq!(
        authority.redeem_admission_grant(&first_grant, &admission_time)?,
        RedemptionOutcome::Redeemed
    );
    assert_eq!(authority.end_assignment(&assignment)?, EndOutcome::Ended);
    assert_eq!(
        authority.redeem_admission_grant(&second_grant, &admission_time),
        Err(AuthorityError::NotUsable)
    );

    let after_end = at(&authority, 6);
    assert_eq!(
        authority.admission_grant(&first_grant, &after_end)?.state(),
        AdmissionGrantState::Redeemed
    );
    assert_eq!(
        authority
            .admission_grant(&second_grant, &after_end)?
            .state(),
        AdmissionGrantState::AssignmentEnded
    );

    Ok(())
}

fn run_proof() -> Result<(), AuthorityError> {
    direct_assignment_and_admission()?;
    optional_matchmaking_composition()?;
    Ok(())
}

fn main() {
    run_proof().expect("standalone RunenOnline proof must succeed");
    println!("RunenOnline standalone proof passed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standalone_public_surface_proof() {
        run_proof().unwrap();
    }
}
