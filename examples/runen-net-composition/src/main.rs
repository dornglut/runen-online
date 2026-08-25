use std::num::{NonZeroU64, NonZeroUsize};

use runen_net::identity::{ConnectionHandle, ParticipantId, SessionId};
use runen_net::protocol::{
    CompatibilityOffer, NegotiatedContract, NegotiationManager, NegotiationManagerLimits,
    NegotiationRequirements, NegotiationStatus, OfferLimits, ProtocolContract, ProtocolId,
    ProtocolRevision,
};
use runen_net::session::{
    ConnectionLossOutcome, MembershipState, RetentionPolicy, Session, SessionError, SessionLimits,
};
use runen_online::{
    AdmissionGrantState, AssignmentId, Authority, AuthorityDomainHandle, AuthorityError,
    AuthorityLimits, EndOutcome, LogicalDestinationHandle, PlayerId, RedemptionOutcome,
    TimeDomainHandle, TrustedTime,
};

struct HostMembership {
    player: PlayerId,
    assignment: AssignmentId,
    session: SessionId,
    participant: ParticipantId,
}

fn online_limits() -> AuthorityLimits {
    AuthorityLimits {
        max_trusted_external_authorities: 1,
        max_external_authority_bytes: 32,
        max_external_subject_bytes: 64,
        max_players: 8,
        max_principal_associations: 8,
        max_principal_associations_per_player: 2,
        max_assignments: 8,
        max_pending_assignment_lifetime: 100,
        max_admission_grants: 16,
        max_admission_grant_lifetime: 50,
        max_live_admission_grants_per_player: 4,
        max_live_admission_grants_per_assignment: 8,
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

fn new_authority() -> Authority {
    Authority::new(
        AuthorityDomainHandle::new(),
        TimeDomainHandle::new(),
        online_limits(),
        std::iter::empty::<&[u8]>(),
    )
    .unwrap()
}

fn at(authority: &Authority, value: u64) -> TrustedTime {
    TrustedTime::new(authority.time_domain(), value)
}

fn session_limits(max_memberships: usize) -> SessionLimits {
    SessionLimits::new(
        NonZeroUsize::new(16).unwrap(),
        NonZeroUsize::new(max_memberships).unwrap(),
    )
    .unwrap()
}

fn protocol() -> ProtocolContract {
    ProtocolContract::new(ProtocolId::new(1), ProtocolRevision::new(1))
}

fn negotiation_manager() -> NegotiationManager {
    NegotiationManager::new(OfferLimits::default(), NegotiationManagerLimits::default()).unwrap()
}

fn establish_negotiation(manager: &mut NegotiationManager, connection: ConnectionHandle) {
    let offer = CompatibilityOffer::new(vec![protocol()], vec![], vec![], None);
    manager.start(connection, offer.clone(), offer).unwrap();

    let contract = NegotiatedContract::new(protocol());
    manager
        .propose(
            connection,
            contract.clone(),
            &NegotiationRequirements::default(),
        )
        .unwrap();
    assert_ne!(
        manager.validate_authority(connection, &contract).unwrap(),
        NegotiationStatus::Established
    );
    assert_eq!(
        manager.validate_peer(connection, &contract).unwrap(),
        NegotiationStatus::Established
    );
}

fn create_grant(
    authority: &mut Authority,
    destination: u64,
    issue_time: u64,
    deadline: u64,
) -> (PlayerId, AssignmentId, runen_online::AdmissionGrantId) {
    let player = authority.create_player().unwrap();
    let assignment = authority
        .establish_usable_assignment(LogicalDestinationHandle::new(destination))
        .unwrap();
    let issued_at = at(authority, issue_time);
    let grant = authority
        .issue_admission_grant(&player, &assignment, &issued_at, deadline)
        .unwrap();
    (player, assignment, grant)
}

fn successful_handoff_and_rebind() {
    let mut authority = new_authority();
    let (player, assignment, grant) = create_grant(&mut authority, 100, 1, 20);

    let first_connection = ConnectionHandle::new(1);
    let replacement_connection = ConnectionHandle::new(2);
    let mut negotiation = negotiation_manager();
    establish_negotiation(&mut negotiation, first_connection);

    let session_id = SessionId::new(700);
    let participant = ParticipantId::new(900);
    let mut session = Session::new(session_id, session_limits(4));

    assert_eq!(session.live_memberships(), 0);
    assert_eq!(
        authority
            .admission_grant(&grant, &at(&authority, 1))
            .unwrap()
            .state(),
        AdmissionGrantState::Redeemable { deadline: 20 }
    );

    assert_eq!(
        authority
            .redeem_admission_grant(&grant, &at(&authority, 2))
            .unwrap(),
        RedemptionOutcome::Redeemed
    );

    // RunenOnline redemption is host evidence only. It does not mutate the
    // RunenNet session or create membership implicitly.
    assert_eq!(session.live_memberships(), 0);

    let mapping = HostMembership {
        player: player.clone(),
        assignment: assignment.clone(),
        session: session_id,
        participant,
    };

    session
        .admit_new(
            participant,
            negotiation.established(first_connection).unwrap(),
        )
        .unwrap();

    assert_eq!(mapping.player, player);
    assert_eq!(mapping.assignment, assignment);
    assert_eq!(mapping.session, session_id);
    assert_eq!(mapping.participant, participant);
    assert_eq!(
        session.membership_state(participant),
        Some(MembershipState::Bound(first_connection))
    );
    assert!(session.is_authorized(participant, first_connection));

    assert_eq!(authority.end_assignment(&assignment).unwrap(), EndOutcome::Ended);
    assert_eq!(
        authority
            .admission_grant(&grant, &at(&authority, 3))
            .unwrap()
            .state(),
        AdmissionGrantState::Redeemed
    );

    // Ending the RunenOnline Assignment does not silently terminate a
    // separately established RunenNet membership.
    assert_eq!(
        session.membership_state(participant),
        Some(MembershipState::Bound(first_connection))
    );

    assert_eq!(
        session
            .connection_lost(
                participant,
                first_connection,
                RetentionPolicy::RetainForRecovery {
                    duration: NonZeroU64::new(10).unwrap(),
                },
            )
            .unwrap(),
        ConnectionLossOutcome::Retained { expires_at: 10 }
    );
    assert_eq!(
        session.membership_state(participant),
        Some(MembershipState::Unbound { expires_at: 10 })
    );

    // A redeemed grant is not reconnect authority and cannot become fresh
    // authorization again merely because the transport connection changed.
    assert_eq!(
        authority
            .redeem_admission_grant(&grant, &at(&authority, 4))
            .unwrap(),
        RedemptionOutcome::AlreadyRedeemed
    );

    establish_negotiation(&mut negotiation, replacement_connection);
    session
        .bind_replacement(
            participant,
            negotiation.established(replacement_connection).unwrap(),
        )
        .unwrap();

    assert_eq!(
        session.membership_state(participant),
        Some(MembershipState::Bound(replacement_connection))
    );
    assert!(!session.is_authorized(participant, first_connection));
    assert!(session.is_authorized(participant, replacement_connection));
}

fn failed_authorization_does_not_admit() {
    let mut authority = new_authority();
    let (_player, assignment, grant) = create_grant(&mut authority, 200, 1, 20);

    let connection = ConnectionHandle::new(10);
    let mut negotiation = negotiation_manager();
    establish_negotiation(&mut negotiation, connection);

    let mut session = Session::new(SessionId::new(701), session_limits(2));
    assert_eq!(session.live_memberships(), 0);
    assert_eq!(
        authority
            .admission_grant(&grant, &at(&authority, 1))
            .unwrap()
            .state(),
        AdmissionGrantState::Redeemable { deadline: 20 }
    );

    assert_eq!(authority.end_assignment(&assignment).unwrap(), EndOutcome::Ended);
    assert_eq!(
        authority.redeem_admission_grant(&grant, &at(&authority, 2)),
        Err(AuthorityError::NotUsable)
    );
    assert_eq!(
        authority
            .admission_grant(&grant, &at(&authority, 2))
            .unwrap()
            .state(),
        AdmissionGrantState::AssignmentEnded
    );

    // The host has no successful RunenOnline authorization to map into a new
    // RunenNet admission, so it performs no Session::admit_new operation.
    assert_eq!(session.live_memberships(), 0);
    assert_eq!(session.participant_for_connection(connection), None);
}

fn admission_failure_does_not_roll_back_redemption() {
    let mut authority = new_authority();
    let (_player, _assignment, grant) = create_grant(&mut authority, 300, 1, 20);

    let occupied_connection = ConnectionHandle::new(20);
    let denied_connection = ConnectionHandle::new(21);
    let mut negotiation = negotiation_manager();
    establish_negotiation(&mut negotiation, occupied_connection);
    establish_negotiation(&mut negotiation, denied_connection);

    // A one-membership RunenNet session gives a deterministic admission failure
    // after the first unrelated membership has occupied the partition.
    let mut session = Session::new(SessionId::new(702), session_limits(1));
    let existing_participant = ParticipantId::new(1);
    session
        .admit_new(
            existing_participant,
            negotiation.established(occupied_connection).unwrap(),
        )
        .unwrap();
    assert_eq!(session.live_memberships(), 1);

    assert_eq!(
        authority
            .redeem_admission_grant(&grant, &at(&authority, 2))
            .unwrap(),
        RedemptionOutcome::Redeemed
    );

    let denied_participant = ParticipantId::new(2);
    assert_eq!(
        session.admit_new(
            denied_participant,
            negotiation.established(denied_connection).unwrap(),
        ),
        Err(SessionError::MembershipLimitExceeded)
    );

    // The two frameworks do not share a transaction. RunenNet rejection does
    // not roll the already-committed RunenOnline redemption back to Redeemable.
    assert_eq!(session.live_memberships(), 1);
    assert_eq!(session.participant_for_connection(denied_connection), None);
    assert_eq!(
        authority
            .admission_grant(&grant, &at(&authority, 3))
            .unwrap()
            .state(),
        AdmissionGrantState::Redeemed
    );
    assert_eq!(
        authority
            .redeem_admission_grant(&grant, &at(&authority, 3))
            .unwrap(),
        RedemptionOutcome::AlreadyRedeemed
    );
}

fn run_proof() {
    successful_handoff_and_rebind();
    failed_authorization_does_not_admit();
    admission_failure_does_not_roll_back_redemption();
}

fn main() {
    run_proof();
    println!("RunenOnline + RunenNet composition proof passed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_server_composition_public_surface_proof() {
        run_proof();
    }
}
