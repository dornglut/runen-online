use crate::{AdmissionGrantId, AssignmentId, PlayerId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionGrantState {
    Redeemable { deadline: u64 },
    Redeemed,
    Expired,
    AssignmentEnded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmissionGrantView {
    id: AdmissionGrantId,
    player: PlayerId,
    assignment: AssignmentId,
    state: AdmissionGrantState,
}

impl AdmissionGrantView {
    pub const fn id(self) -> AdmissionGrantId {
        self.id
    }

    pub const fn player(self) -> PlayerId {
        self.player
    }

    pub const fn assignment(self) -> AssignmentId {
        self.assignment
    }

    pub const fn state(self) -> AdmissionGrantState {
        self.state
    }

    pub(crate) const fn new(
        id: AdmissionGrantId,
        player: PlayerId,
        assignment: AssignmentId,
        state: AdmissionGrantState,
    ) -> Self {
        Self {
            id,
            player,
            assignment,
            state,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RedemptionOutcome {
    Redeemed,
    AlreadyRedeemed,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AdmissionGrantRecord {
    pub id: AdmissionGrantId,
    pub player: PlayerId,
    pub assignment: AssignmentId,
    pub state: AdmissionGrantState,
}

impl AdmissionGrantRecord {
    pub(crate) const fn new(
        id: AdmissionGrantId,
        player: PlayerId,
        assignment: AssignmentId,
        deadline: u64,
    ) -> Self {
        Self {
            id,
            player,
            assignment,
            state: AdmissionGrantState::Redeemable { deadline },
        }
    }

    pub(crate) const fn view(self) -> AdmissionGrantView {
        AdmissionGrantView::new(self.id, self.player, self.assignment, self.state)
    }
}
