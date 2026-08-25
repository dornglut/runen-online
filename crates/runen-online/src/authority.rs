use std::collections::{BTreeMap, BTreeSet};

use crate::admission::{AdmissionGrantRecord, AdmissionGrantState};
use crate::assignment::{AssignmentRecord, AssignmentState};
use crate::identity::{IdAllocator, PrincipalKey};
use crate::matchmaking::{MatchRecord, MatchRequestRecord, MatchRequestState};
use crate::time::{deadline_reached, require_time_domain, validate_deadline};
use crate::{
    AdmissionGrantId, AdmissionGrantView, AssignmentId, AssignmentResolutionOutcome,
    AssignmentView, AuthorityDomainHandle, AuthorityDomainId, AuthorityError, AuthorityLimits,
    IdKind, InvalidInputKind, LogicalDestinationHandle, MatchContribution, MatchId, MatchRequestId,
    MatchRequestView, MatchView, ObjectKind, PlayerId, RedemptionOutcome, ResourceLimit,
    TimeDomainHandle, TrustedTime, VerifiedExternalPrincipal,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssociationOutcome {
    Associated,
    AlreadyAssociated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndOutcome {
    Ended,
    AlreadyTerminal,
}

#[derive(Clone, Debug, Default)]
struct PlayerRecord {
    associations: BTreeSet<PrincipalKey>,
}

/// Bounded deterministic in-process realization of one RunenOnline authority
/// domain.
///
/// The container is an RO2 implementation mechanism, not a persistence,
/// service, provider, transaction, or distributed-consensus contract.
/// Constructing it consumes one linear `AuthorityDomainHandle`, so safe code
/// cannot create two independent aggregates that mint colliding IDs for the
/// same process-local domain token.
pub struct Authority {
    domain: AuthorityDomainId,
    time_domain: TimeDomainHandle,
    limits: AuthorityLimits,
    trusted_external_authorities: BTreeSet<Box<[u8]>>,

    players: BTreeMap<PlayerId, PlayerRecord>,
    principal_to_player: BTreeMap<PrincipalKey, PlayerId>,

    assignments: BTreeMap<AssignmentId, AssignmentRecord>,

    admission_grants: BTreeMap<AdmissionGrantId, AdmissionGrantRecord>,
    live_grants_by_player: BTreeMap<PlayerId, BTreeSet<AdmissionGrantId>>,
    live_grants_by_assignment: BTreeMap<AssignmentId, BTreeSet<AdmissionGrantId>>,

    match_requests: BTreeMap<MatchRequestId, MatchRequestRecord>,
    pending_match_requests_by_player: BTreeMap<PlayerId, BTreeSet<MatchRequestId>>,
    matches: BTreeMap<MatchId, MatchRecord>,

    player_ids: IdAllocator,
    assignment_ids: IdAllocator,
    admission_grant_ids: IdAllocator,
    match_request_ids: IdAllocator,
    match_ids: IdAllocator,
}

impl Authority {
    pub fn new<I, B>(
        domain: AuthorityDomainHandle,
        time_domain: TimeDomainHandle,
        limits: AuthorityLimits,
        trusted_external_authorities: I,
    ) -> Result<Self, AuthorityError>
    where
        I: IntoIterator<Item = B>,
        B: AsRef<[u8]>,
    {
        let mut trusted = BTreeSet::new();
        let mut supplied_count = 0usize;

        trusted_external_authorities
            .into_iter()
            .try_for_each(|authority| {
                if supplied_count >= limits.max_trusted_external_authorities {
                    return Err(AuthorityError::ResourceLimit(
                        ResourceLimit::TrustedExternalAuthorities,
                    ));
                }
                supplied_count = supplied_count.checked_add(1).ok_or(
                    AuthorityError::ResourceLimit(ResourceLimit::TrustedExternalAuthorities),
                )?;

                let authority = authority.as_ref();
                if authority.len() > limits.max_external_authority_bytes {
                    return Err(AuthorityError::ResourceLimit(
                        ResourceLimit::ExternalAuthorityBytes,
                    ));
                }
                trusted.insert(authority.into());
                Ok(())
            })?;

        Ok(Self {
            domain: domain.into_id(),
            time_domain,
            limits,
            trusted_external_authorities: trusted,
            players: BTreeMap::new(),
            principal_to_player: BTreeMap::new(),
            assignments: BTreeMap::new(),
            admission_grants: BTreeMap::new(),
            live_grants_by_player: BTreeMap::new(),
            live_grants_by_assignment: BTreeMap::new(),
            match_requests: BTreeMap::new(),
            pending_match_requests_by_player: BTreeMap::new(),
            matches: BTreeMap::new(),
            player_ids: IdAllocator::new(IdKind::Player),
            assignment_ids: IdAllocator::new(IdKind::Assignment),
            admission_grant_ids: IdAllocator::new(IdKind::AdmissionGrant),
            match_request_ids: IdAllocator::new(IdKind::MatchRequest),
            match_ids: IdAllocator::new(IdKind::Match),
        })
    }

    pub const fn domain(&self) -> &AuthorityDomainId {
        &self.domain
    }

    pub fn time_domain(&self) -> TimeDomainHandle {
        self.time_domain.clone()
    }

    pub const fn limits(&self) -> &AuthorityLimits {
        &self.limits
    }

    // Identity / trusted external principal ---------------------------------

    pub fn create_player(&mut self) -> Result<PlayerId, AuthorityError> {
        if self.players.len() >= self.limits.max_players {
            return Err(AuthorityError::ResourceLimit(ResourceLimit::Players));
        }

        let local = self.player_ids.allocate()?;
        let id = PlayerId::from_parts(self.domain.clone(), local);
        self.players.insert(id.clone(), PlayerRecord::default());
        Ok(id)
    }

    /// Accepts already-verified external-principal evidence supplied through
    /// the trusted host boundary and validates RunenOnline-local trust and
    /// representation bounds.
    pub fn accept_verified_external_principal(
        &self,
        external_authority: &[u8],
        external_subject: &[u8],
    ) -> Result<VerifiedExternalPrincipal, AuthorityError> {
        if external_authority.len() > self.limits.max_external_authority_bytes {
            return Err(AuthorityError::ResourceLimit(
                ResourceLimit::ExternalAuthorityBytes,
            ));
        }
        if external_subject.len() > self.limits.max_external_subject_bytes {
            return Err(AuthorityError::ResourceLimit(
                ResourceLimit::ExternalSubjectBytes,
            ));
        }
        if !self
            .trusted_external_authorities
            .contains(external_authority)
        {
            return Err(AuthorityError::UntrustedExternalAuthority);
        }

        Ok(VerifiedExternalPrincipal::new(
            self.domain.clone(),
            external_authority,
            external_subject,
        ))
    }

    pub fn associate_principal(
        &mut self,
        player: &PlayerId,
        principal: &VerifiedExternalPrincipal,
    ) -> Result<AssociationOutcome, AuthorityError> {
        self.require_player_domain(player)?;
        if principal.domain() != &self.domain {
            return Err(AuthorityError::AuthorityDomainMismatch);
        }
        if !self.players.contains_key(player) {
            return Err(AuthorityError::Unknown(ObjectKind::Player));
        }

        let key = PrincipalKey::from(principal);
        if let Some(existing) = self.principal_to_player.get(&key) {
            return if existing == player {
                Ok(AssociationOutcome::AlreadyAssociated)
            } else {
                Err(AuthorityError::Conflict)
            };
        }

        let association_count = self
            .players
            .get(player)
            .expect("player existence checked above")
            .associations
            .len();
        if association_count >= self.limits.max_principal_associations_per_player {
            return Err(AuthorityError::ResourceLimit(
                ResourceLimit::PrincipalAssociationsPerPlayer,
            ));
        }
        if self.principal_to_player.len() >= self.limits.max_principal_associations {
            return Err(AuthorityError::ResourceLimit(
                ResourceLimit::PrincipalAssociations,
            ));
        }

        self.players
            .get_mut(player)
            .expect("player existence checked above")
            .associations
            .insert(key.clone());
        self.principal_to_player.insert(key, player.clone());
        Ok(AssociationOutcome::Associated)
    }

    pub fn resolve_principal(
        &self,
        principal: &VerifiedExternalPrincipal,
    ) -> Result<Option<PlayerId>, AuthorityError> {
        if principal.domain() != &self.domain {
            return Err(AuthorityError::AuthorityDomainMismatch);
        }
        Ok(self
            .principal_to_player
            .get(&PrincipalKey::from(principal))
            .cloned())
    }

    // Assignment -------------------------------------------------------------

    pub fn establish_pending_assignment(
        &mut self,
        now: &TrustedTime,
        deadline: u64,
    ) -> Result<AssignmentId, AuthorityError> {
        validate_deadline(
            now,
            &self.time_domain,
            deadline,
            self.limits.max_pending_assignment_lifetime,
            ResourceLimit::PendingAssignmentLifetime,
        )?;
        if self.assignments.len() >= self.limits.max_assignments {
            return Err(AuthorityError::ResourceLimit(ResourceLimit::Assignments));
        }

        let local = self.assignment_ids.allocate()?;
        let id = AssignmentId::from_parts(self.domain.clone(), local);
        self.assignments
            .insert(id.clone(), AssignmentRecord::pending(id.clone(), deadline));
        Ok(id)
    }

    pub fn establish_usable_assignment(
        &mut self,
        destination: LogicalDestinationHandle,
    ) -> Result<AssignmentId, AuthorityError> {
        if self.assignments.len() >= self.limits.max_assignments {
            return Err(AuthorityError::ResourceLimit(ResourceLimit::Assignments));
        }

        let local = self.assignment_ids.allocate()?;
        let id = AssignmentId::from_parts(self.domain.clone(), local);
        self.assignments.insert(
            id.clone(),
            AssignmentRecord::usable(id.clone(), destination),
        );
        Ok(id)
    }

    pub fn resolve_assignment(
        &mut self,
        assignment: &AssignmentId,
        destination: LogicalDestinationHandle,
        now: &TrustedTime,
    ) -> Result<AssignmentResolutionOutcome, AuthorityError> {
        self.require_assignment_domain(assignment)?;
        require_time_domain(now, &self.time_domain)?;

        let state = self
            .assignments
            .get(assignment)
            .ok_or(AuthorityError::Unknown(ObjectKind::Assignment))?
            .state;

        match state {
            AssignmentState::Pending { deadline } => {
                if deadline_reached(now, &self.time_domain, deadline)? {
                    self.assignments
                        .get_mut(assignment)
                        .expect("assignment existence checked above")
                        .state = AssignmentState::Ended;
                    return Err(AuthorityError::Expired);
                }

                self.assignments
                    .get_mut(assignment)
                    .expect("assignment existence checked above")
                    .state = AssignmentState::Usable { destination };
                Ok(AssignmentResolutionOutcome::Resolved)
            }
            AssignmentState::Usable {
                destination: current,
            } if current == destination => Ok(AssignmentResolutionOutcome::AlreadyUsable),
            AssignmentState::Usable { .. } => Err(AuthorityError::Conflict),
            AssignmentState::Ended => Err(AuthorityError::Terminal),
        }
    }

    pub fn end_assignment(
        &mut self,
        assignment: &AssignmentId,
    ) -> Result<EndOutcome, AuthorityError> {
        self.require_assignment_domain(assignment)?;
        let record = self
            .assignments
            .get_mut(assignment)
            .ok_or(AuthorityError::Unknown(ObjectKind::Assignment))?;

        if record.state == AssignmentState::Ended {
            return Ok(EndOutcome::AlreadyTerminal);
        }

        record.state = AssignmentState::Ended;
        self.block_live_grants_for_assignment(assignment);
        Ok(EndOutcome::Ended)
    }

    pub fn assignment(
        &mut self,
        assignment: &AssignmentId,
        now: &TrustedTime,
    ) -> Result<AssignmentView, AuthorityError> {
        self.require_assignment_domain(assignment)?;
        self.refresh_assignment_expiry(assignment, now)?;
        Ok(self
            .assignments
            .get(assignment)
            .ok_or(AuthorityError::Unknown(ObjectKind::Assignment))?
            .view())
    }

    // Admission --------------------------------------------------------------

    pub fn issue_admission_grant(
        &mut self,
        player: &PlayerId,
        assignment: &AssignmentId,
        now: &TrustedTime,
        deadline: u64,
    ) -> Result<AdmissionGrantId, AuthorityError> {
        self.require_player_domain(player)?;
        self.require_assignment_domain(assignment)?;
        validate_deadline(
            now,
            &self.time_domain,
            deadline,
            self.limits.max_admission_grant_lifetime,
            ResourceLimit::AdmissionGrantLifetime,
        )?;

        if !self.players.contains_key(player) {
            return Err(AuthorityError::Unknown(ObjectKind::Player));
        }

        self.refresh_assignment_expiry(assignment, now)?;
        let assignment_state = self
            .assignments
            .get(assignment)
            .ok_or(AuthorityError::Unknown(ObjectKind::Assignment))?
            .state;
        if !matches!(assignment_state, AssignmentState::Usable { .. }) {
            return Err(AuthorityError::NotUsable);
        }

        if self.admission_grants.len() >= self.limits.max_admission_grants {
            return Err(AuthorityError::ResourceLimit(
                ResourceLimit::AdmissionGrants,
            ));
        }

        // These are *live* limits. Reconcile relevant lazy expiry/currentness
        // before counting so semantically dead grants cannot block new work.
        self.refresh_live_grants_for_player(player, now)?;
        self.refresh_live_grants_for_assignment(assignment, now)?;

        if self.live_grant_count_for_player(player)
            >= self.limits.max_live_admission_grants_per_player
        {
            return Err(AuthorityError::ResourceLimit(
                ResourceLimit::LiveAdmissionGrantsPerPlayer,
            ));
        }
        if self.live_grant_count_for_assignment(assignment)
            >= self.limits.max_live_admission_grants_per_assignment
        {
            return Err(AuthorityError::ResourceLimit(
                ResourceLimit::LiveAdmissionGrantsPerAssignment,
            ));
        }

        let local = self.admission_grant_ids.allocate()?;
        let id = AdmissionGrantId::from_parts(self.domain.clone(), local);
        self.admission_grants.insert(
            id.clone(),
            AdmissionGrantRecord::new(id.clone(), player.clone(), assignment.clone(), deadline),
        );
        self.live_grants_by_player
            .entry(player.clone())
            .or_default()
            .insert(id.clone());
        self.live_grants_by_assignment
            .entry(assignment.clone())
            .or_default()
            .insert(id.clone());
        Ok(id)
    }

    pub fn redeem_admission_grant(
        &mut self,
        grant: &AdmissionGrantId,
        now: &TrustedTime,
    ) -> Result<RedemptionOutcome, AuthorityError> {
        self.require_grant_domain(grant)?;
        require_time_domain(now, &self.time_domain)?;

        let record = self
            .admission_grants
            .get(grant)
            .cloned()
            .ok_or(AuthorityError::Unknown(ObjectKind::AdmissionGrant))?;

        match record.state {
            AdmissionGrantState::Redeemed => return Ok(RedemptionOutcome::AlreadyRedeemed),
            AdmissionGrantState::Expired => return Err(AuthorityError::Expired),
            AdmissionGrantState::AssignmentEnded => return Err(AuthorityError::NotUsable),
            AdmissionGrantState::Redeemable { deadline } => {
                if deadline_reached(now, &self.time_domain, deadline)? {
                    self.set_grant_terminal(grant, AdmissionGrantState::Expired);
                    return Err(AuthorityError::Expired);
                }
            }
        }

        let assignment_state = self
            .assignments
            .get(&record.assignment)
            .ok_or(AuthorityError::Unknown(ObjectKind::Assignment))?
            .state;
        if !matches!(assignment_state, AssignmentState::Usable { .. }) {
            self.set_grant_terminal(grant, AdmissionGrantState::AssignmentEnded);
            return Err(AuthorityError::NotUsable);
        }

        self.set_grant_terminal(grant, AdmissionGrantState::Redeemed);
        Ok(RedemptionOutcome::Redeemed)
    }

    pub fn admission_grant(
        &mut self,
        grant: &AdmissionGrantId,
        now: &TrustedTime,
    ) -> Result<AdmissionGrantView, AuthorityError> {
        self.require_grant_domain(grant)?;
        self.refresh_grant_currentness(grant, now)?;
        Ok(self
            .admission_grants
            .get(grant)
            .ok_or(AuthorityError::Unknown(ObjectKind::AdmissionGrant))?
            .view())
    }

    // Matchmaking ------------------------------------------------------------

    pub fn establish_match_request(
        &mut self,
        cohort: &[PlayerId],
        matching_input: &[u8],
        now: &TrustedTime,
        deadline: u64,
    ) -> Result<MatchRequestId, AuthorityError> {
        validate_deadline(
            now,
            &self.time_domain,
            deadline,
            self.limits.max_match_request_lifetime,
            ResourceLimit::MatchRequestLifetime,
        )?;

        if cohort.is_empty() {
            return Err(AuthorityError::InvalidInput(
                InvalidInputKind::EmptyMatchCohort,
            ));
        }
        if cohort.len() > self.limits.max_match_request_cohort {
            return Err(AuthorityError::ResourceLimit(
                ResourceLimit::MatchRequestCohort,
            ));
        }
        if matching_input.len() > self.limits.max_matchmaking_input_bytes {
            return Err(AuthorityError::ResourceLimit(
                ResourceLimit::MatchmakingInputBytes,
            ));
        }
        if self.match_requests.len() >= self.limits.max_match_requests {
            return Err(AuthorityError::ResourceLimit(ResourceLimit::MatchRequests));
        }

        // Validate the requested cohort fully before any lazy-expiry
        // materialization so malformed input does not cause unrelated state
        // evolution.
        let mut unique_players = BTreeSet::new();
        for player in cohort {
            self.require_player_domain(player)?;
            if !self.players.contains_key(player) {
                return Err(AuthorityError::Unknown(ObjectKind::Player));
            }
            if !unique_players.insert(player.clone()) {
                return Err(AuthorityError::InvalidInput(
                    InvalidInputKind::DuplicatePlayer,
                ));
            }
        }

        // The quota is defined over semantically Pending requests. Reconcile
        // lazy deadline expiry for the selected players before applying it.
        for player in &unique_players {
            self.refresh_pending_match_requests_for_player(player, now)?;
        }
        for player in &unique_players {
            if self.pending_match_request_count_for_player(player)
                >= self.limits.max_pending_match_requests_per_player
            {
                return Err(AuthorityError::ResourceLimit(
                    ResourceLimit::PendingMatchRequestsPerPlayer,
                ));
            }
        }

        let local = self.match_request_ids.allocate()?;
        let id = MatchRequestId::from_parts(self.domain.clone(), local);
        self.match_requests.insert(
            id.clone(),
            MatchRequestRecord {
                id: id.clone(),
                cohort: cohort.to_vec().into_boxed_slice(),
                matching_input: matching_input.into(),
                state: MatchRequestState::Pending { deadline },
            },
        );
        for player in cohort {
            self.pending_match_requests_by_player
                .entry(player.clone())
                .or_default()
                .insert(id.clone());
        }
        Ok(id)
    }

    pub fn end_match_request(
        &mut self,
        request: &MatchRequestId,
    ) -> Result<EndOutcome, AuthorityError> {
        self.require_match_request_domain(request)?;
        let state = self
            .match_requests
            .get(request)
            .ok_or(AuthorityError::Unknown(ObjectKind::MatchRequest))?
            .state
            .clone();

        if !matches!(state, MatchRequestState::Pending { .. }) {
            return Ok(EndOutcome::AlreadyTerminal);
        }

        self.set_match_request_ended(request);
        Ok(EndOutcome::Ended)
    }

    pub fn match_request(
        &mut self,
        request: &MatchRequestId,
        now: &TrustedTime,
    ) -> Result<MatchRequestView, AuthorityError> {
        self.require_match_request_domain(request)?;
        self.refresh_match_request_expiry(request, now)?;
        Ok(self
            .match_requests
            .get(request)
            .ok_or(AuthorityError::Unknown(ObjectKind::MatchRequest))?
            .view())
    }

    pub fn commit_match(
        &mut self,
        requests: &[MatchRequestId],
        now: &TrustedTime,
    ) -> Result<MatchId, AuthorityError> {
        require_time_domain(now, &self.time_domain)?;

        if requests.is_empty() {
            return Err(AuthorityError::InvalidInput(
                InvalidInputKind::EmptyMatchCandidate,
            ));
        }
        if requests.len() > self.limits.max_match_candidate_requests {
            return Err(AuthorityError::ResourceLimit(
                ResourceLimit::MatchCandidateRequests,
            ));
        }
        if self.matches.len() >= self.limits.max_matches {
            return Err(AuthorityError::ResourceLimit(ResourceLimit::Matches));
        }

        let mut unique_requests = BTreeSet::new();
        let mut selected = Vec::with_capacity(requests.len());
        for request in requests {
            self.require_match_request_domain(request)?;
            if !unique_requests.insert(request.clone()) {
                return Err(AuthorityError::InvalidInput(
                    InvalidInputKind::DuplicateMatchRequest,
                ));
            }

            let record = self
                .match_requests
                .get(request)
                .ok_or(AuthorityError::Unknown(ObjectKind::MatchRequest))?;
            let deadline = match &record.state {
                MatchRequestState::Pending { deadline } => *deadline,
                MatchRequestState::Matched(_) | MatchRequestState::Ended => {
                    return Err(AuthorityError::Terminal);
                }
            };
            selected.push((request.clone(), record.cohort.clone(), deadline));
        }

        let mut expired = Vec::new();
        for (request, _, deadline) in &selected {
            if deadline_reached(now, &self.time_domain, *deadline)? {
                expired.push(request.clone());
            }
        }
        if !expired.is_empty() {
            for request in &expired {
                self.set_match_request_ended(request);
            }
            return Err(AuthorityError::Expired);
        }

        let mut unique_players = BTreeSet::new();
        let mut roster = Vec::new();
        let mut contributions = Vec::with_capacity(selected.len());
        for (request, cohort, _) in &selected {
            for player in cohort.iter() {
                if !unique_players.insert(player.clone()) {
                    return Err(AuthorityError::InvalidInput(
                        InvalidInputKind::OverlappingPlayer,
                    ));
                }
                if roster.len() >= self.limits.max_match_roster_players {
                    return Err(AuthorityError::ResourceLimit(
                        ResourceLimit::MatchRosterPlayers,
                    ));
                }
                roster.push(player.clone());
            }
            contributions.push(MatchContribution::new(request.clone(), cohort.clone()));
        }

        let local = self.match_ids.allocate()?;
        let id = MatchId::from_parts(self.domain.clone(), local);
        let view = MatchView::new(
            id.clone(),
            contributions.into_boxed_slice(),
            roster.into_boxed_slice(),
        );

        // All fallible semantic validation is complete. Exclusive `&mut`
        // authority access now commits every selected request and the immutable
        // Match as one in-process semantic operation.
        for (request, _, _) in &selected {
            self.set_match_request_matched(request, &id);
        }
        self.matches.insert(id.clone(), MatchRecord { view });
        Ok(id)
    }

    pub fn committed_match(&self, id: &MatchId) -> Result<MatchView, AuthorityError> {
        self.require_match_domain(id)?;
        Ok(self
            .matches
            .get(id)
            .ok_or(AuthorityError::Unknown(ObjectKind::Match))?
            .view
            .clone())
    }

    // Internal invariant helpers --------------------------------------------

    fn require_player_domain(&self, id: &PlayerId) -> Result<(), AuthorityError> {
        if id.domain() != &self.domain {
            return Err(AuthorityError::AuthorityDomainMismatch);
        }
        Ok(())
    }

    fn require_assignment_domain(&self, id: &AssignmentId) -> Result<(), AuthorityError> {
        if id.domain() != &self.domain {
            return Err(AuthorityError::AuthorityDomainMismatch);
        }
        Ok(())
    }

    fn require_grant_domain(&self, id: &AdmissionGrantId) -> Result<(), AuthorityError> {
        if id.domain() != &self.domain {
            return Err(AuthorityError::AuthorityDomainMismatch);
        }
        Ok(())
    }

    fn require_match_request_domain(&self, id: &MatchRequestId) -> Result<(), AuthorityError> {
        if id.domain() != &self.domain {
            return Err(AuthorityError::AuthorityDomainMismatch);
        }
        Ok(())
    }

    fn require_match_domain(&self, id: &MatchId) -> Result<(), AuthorityError> {
        if id.domain() != &self.domain {
            return Err(AuthorityError::AuthorityDomainMismatch);
        }
        Ok(())
    }

    fn refresh_assignment_expiry(
        &mut self,
        assignment: &AssignmentId,
        now: &TrustedTime,
    ) -> Result<(), AuthorityError> {
        require_time_domain(now, &self.time_domain)?;
        let state = self
            .assignments
            .get(assignment)
            .ok_or(AuthorityError::Unknown(ObjectKind::Assignment))?
            .state;
        if let AssignmentState::Pending { deadline } = state
            && deadline_reached(now, &self.time_domain, deadline)?
        {
            self.assignments
                .get_mut(assignment)
                .expect("assignment existence checked above")
                .state = AssignmentState::Ended;
        }
        Ok(())
    }

    fn block_live_grants_for_assignment(&mut self, assignment: &AssignmentId) {
        let grants = self
            .live_grants_by_assignment
            .get(assignment)
            .cloned()
            .unwrap_or_default();
        for grant in grants {
            self.set_grant_terminal(&grant, AdmissionGrantState::AssignmentEnded);
        }
    }

    fn refresh_live_grants_for_player(
        &mut self,
        player: &PlayerId,
        now: &TrustedTime,
    ) -> Result<(), AuthorityError> {
        let grants: Vec<_> = self
            .live_grants_by_player
            .get(player)
            .map(|grants| grants.iter().cloned().collect())
            .unwrap_or_default();
        for grant in grants {
            self.refresh_grant_currentness(&grant, now)?;
        }
        Ok(())
    }

    fn refresh_live_grants_for_assignment(
        &mut self,
        assignment: &AssignmentId,
        now: &TrustedTime,
    ) -> Result<(), AuthorityError> {
        let grants: Vec<_> = self
            .live_grants_by_assignment
            .get(assignment)
            .map(|grants| grants.iter().cloned().collect())
            .unwrap_or_default();
        for grant in grants {
            self.refresh_grant_currentness(&grant, now)?;
        }
        Ok(())
    }

    fn refresh_grant_currentness(
        &mut self,
        grant: &AdmissionGrantId,
        now: &TrustedTime,
    ) -> Result<(), AuthorityError> {
        require_time_domain(now, &self.time_domain)?;
        let record = self
            .admission_grants
            .get(grant)
            .cloned()
            .ok_or(AuthorityError::Unknown(ObjectKind::AdmissionGrant))?;
        let deadline = match record.state {
            AdmissionGrantState::Redeemable { deadline } => deadline,
            _ => return Ok(()),
        };

        if deadline_reached(now, &self.time_domain, deadline)? {
            self.set_grant_terminal(grant, AdmissionGrantState::Expired);
            return Ok(());
        }

        let assignment_state = self
            .assignments
            .get(&record.assignment)
            .ok_or(AuthorityError::Unknown(ObjectKind::Assignment))?
            .state;
        if !matches!(assignment_state, AssignmentState::Usable { .. }) {
            self.set_grant_terminal(grant, AdmissionGrantState::AssignmentEnded);
        }
        Ok(())
    }

    fn set_grant_terminal(&mut self, grant: &AdmissionGrantId, state: AdmissionGrantState) {
        let Some(record) = self.admission_grants.get_mut(grant) else {
            return;
        };
        if !matches!(record.state, AdmissionGrantState::Redeemable { .. }) {
            return;
        }

        let player = record.player.clone();
        let assignment = record.assignment.clone();
        record.state = state;
        self.remove_live_grant_indexes(grant, &player, &assignment);
    }

    fn remove_live_grant_indexes(
        &mut self,
        grant: &AdmissionGrantId,
        player: &PlayerId,
        assignment: &AssignmentId,
    ) {
        let remove_player_entry = if let Some(grants) = self.live_grants_by_player.get_mut(player) {
            grants.remove(grant);
            grants.is_empty()
        } else {
            false
        };
        if remove_player_entry {
            self.live_grants_by_player.remove(player);
        }

        let remove_assignment_entry =
            if let Some(grants) = self.live_grants_by_assignment.get_mut(assignment) {
                grants.remove(grant);
                grants.is_empty()
            } else {
                false
            };
        if remove_assignment_entry {
            self.live_grants_by_assignment.remove(assignment);
        }
    }

    fn live_grant_count_for_player(&self, player: &PlayerId) -> usize {
        self.live_grants_by_player
            .get(player)
            .map_or(0, BTreeSet::len)
    }

    fn live_grant_count_for_assignment(&self, assignment: &AssignmentId) -> usize {
        self.live_grants_by_assignment
            .get(assignment)
            .map_or(0, BTreeSet::len)
    }

    fn refresh_pending_match_requests_for_player(
        &mut self,
        player: &PlayerId,
        now: &TrustedTime,
    ) -> Result<(), AuthorityError> {
        let requests: Vec<_> = self
            .pending_match_requests_by_player
            .get(player)
            .map(|requests| requests.iter().cloned().collect())
            .unwrap_or_default();
        for request in requests {
            self.refresh_match_request_expiry(&request, now)?;
        }
        Ok(())
    }

    fn refresh_match_request_expiry(
        &mut self,
        request: &MatchRequestId,
        now: &TrustedTime,
    ) -> Result<(), AuthorityError> {
        require_time_domain(now, &self.time_domain)?;
        let state = self
            .match_requests
            .get(request)
            .ok_or(AuthorityError::Unknown(ObjectKind::MatchRequest))?
            .state
            .clone();
        if let MatchRequestState::Pending { deadline } = state
            && deadline_reached(now, &self.time_domain, deadline)?
        {
            self.set_match_request_ended(request);
        }
        Ok(())
    }

    fn set_match_request_ended(&mut self, request: &MatchRequestId) {
        let Some(record) = self.match_requests.get_mut(request) else {
            return;
        };
        if !matches!(record.state, MatchRequestState::Pending { .. }) {
            return;
        }
        let cohort = record.cohort.clone();
        record.state = MatchRequestState::Ended;
        self.remove_pending_match_request_indexes(request, &cohort);
    }

    fn set_match_request_matched(&mut self, request: &MatchRequestId, matched: &MatchId) {
        let record = self
            .match_requests
            .get_mut(request)
            .expect("match request was fully prevalidated before commit");
        debug_assert!(matches!(record.state, MatchRequestState::Pending { .. }));
        let cohort = record.cohort.clone();
        record.state = MatchRequestState::Matched(matched.clone());
        self.remove_pending_match_request_indexes(request, &cohort);
    }

    fn remove_pending_match_request_indexes(
        &mut self,
        request: &MatchRequestId,
        cohort: &[PlayerId],
    ) {
        for player in cohort {
            let remove_entry =
                if let Some(requests) = self.pending_match_requests_by_player.get_mut(player) {
                    requests.remove(request);
                    requests.is_empty()
                } else {
                    false
                };
            if remove_entry {
                self.pending_match_requests_by_player.remove(player);
            }
        }
    }

    fn pending_match_request_count_for_player(&self, player: &PlayerId) -> usize {
        self.pending_match_requests_by_player
            .get(player)
            .map_or(0, BTreeSet::len)
    }
}
