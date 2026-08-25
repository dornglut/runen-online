use crate::{MatchId, MatchRequestId, PlayerId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchRequestState {
    Pending { deadline: u64 },
    Matched(MatchId),
    Ended,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchRequestView {
    id: MatchRequestId,
    cohort: Box<[PlayerId]>,
    matching_input: Box<[u8]>,
    state: MatchRequestState,
}

impl MatchRequestView {
    pub const fn id(&self) -> MatchRequestId {
        self.id
    }

    pub fn cohort(&self) -> &[PlayerId] {
        &self.cohort
    }

    pub fn matching_input(&self) -> &[u8] {
        &self.matching_input
    }

    pub const fn state(&self) -> MatchRequestState {
        self.state
    }
}

#[derive(Clone, Debug)]
pub(crate) struct MatchRequestRecord {
    pub id: MatchRequestId,
    pub cohort: Box<[PlayerId]>,
    pub matching_input: Box<[u8]>,
    pub state: MatchRequestState,
}

impl MatchRequestRecord {
    pub(crate) fn view(&self) -> MatchRequestView {
        MatchRequestView {
            id: self.id,
            cohort: self.cohort.clone(),
            matching_input: self.matching_input.clone(),
            state: self.state,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchContribution {
    request_id: MatchRequestId,
    cohort: Box<[PlayerId]>,
}

impl MatchContribution {
    pub const fn request_id(&self) -> MatchRequestId {
        self.request_id
    }

    pub fn cohort(&self) -> &[PlayerId] {
        &self.cohort
    }

    pub(crate) fn new(request_id: MatchRequestId, cohort: Box<[PlayerId]>) -> Self {
        Self { request_id, cohort }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchView {
    id: MatchId,
    contributions: Box<[MatchContribution]>,
    roster: Box<[PlayerId]>,
}

impl MatchView {
    pub const fn id(&self) -> MatchId {
        self.id
    }

    pub fn contributions(&self) -> &[MatchContribution] {
        &self.contributions
    }

    pub fn roster(&self) -> &[PlayerId] {
        &self.roster
    }

    pub(crate) fn new(
        id: MatchId,
        contributions: Box<[MatchContribution]>,
        roster: Box<[PlayerId]>,
    ) -> Self {
        Self {
            id,
            contributions,
            roster,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct MatchRecord {
    pub view: MatchView,
}
