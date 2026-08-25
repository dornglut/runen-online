use crate::AssignmentId;

/// Host-local reference to one logical gameplay destination.
///
/// The value is not a process, endpoint, allocation, provider, or network
/// identity contract.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct LogicalDestinationHandle(u64);

impl LogicalDestinationHandle {
    pub const fn new(local_value: u64) -> Self {
        Self(local_value)
    }

    pub const fn local_value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssignmentState {
    Pending {
        deadline: u64,
    },
    Usable {
        destination: LogicalDestinationHandle,
    },
    Ended,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssignmentView {
    id: AssignmentId,
    state: AssignmentState,
}

impl AssignmentView {
    pub const fn id(&self) -> &AssignmentId {
        &self.id
    }

    pub const fn state(&self) -> AssignmentState {
        self.state
    }

    pub(crate) const fn new(id: AssignmentId, state: AssignmentState) -> Self {
        Self { id, state }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssignmentResolutionOutcome {
    Resolved,
    AlreadyUsable,
}

#[derive(Clone, Debug)]
pub(crate) struct AssignmentRecord {
    pub id: AssignmentId,
    pub state: AssignmentState,
}

impl AssignmentRecord {
    pub(crate) const fn pending(id: AssignmentId, deadline: u64) -> Self {
        Self {
            id,
            state: AssignmentState::Pending { deadline },
        }
    }

    pub(crate) const fn usable(id: AssignmentId, destination: LogicalDestinationHandle) -> Self {
        Self {
            id,
            state: AssignmentState::Usable { destination },
        }
    }

    pub(crate) fn view(&self) -> AssignmentView {
        AssignmentView::new(self.id.clone(), self.state)
    }
}
