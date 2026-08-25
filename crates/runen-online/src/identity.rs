use crate::{AuthorityError, IdKind};

/// Local handle identifying one in-process RunenOnline authority domain.
///
/// This is deliberately not a standardized realm, provider, deployment, wire,
/// or storage identifier.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct AuthorityDomainHandle(u64);

impl AuthorityDomainHandle {
    pub const fn new(local_value: u64) -> Self {
        Self(local_value)
    }

    pub const fn local_value(self) -> u64 {
        self.0
    }
}

macro_rules! semantic_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
        pub struct $name {
            domain: AuthorityDomainHandle,
            local_value: u64,
        }

        impl $name {
            pub const fn domain(self) -> AuthorityDomainHandle {
                self.domain
            }

            /// Returns the local implementation incarnation value.
            ///
            /// This value has meaning only together with `domain()` and does
            /// not establish a wire or storage representation.
            pub const fn local_value(self) -> u64 {
                self.local_value
            }

            pub(crate) const fn from_parts(
                domain: AuthorityDomainHandle,
                local_value: u64,
            ) -> Self {
                Self {
                    domain,
                    local_value,
                }
            }
        }
    };
}

semantic_id!(PlayerId);
semantic_id!(AssignmentId);
semantic_id!(AdmissionGrantId);
semantic_id!(MatchRequestId);
semantic_id!(MatchId);

/// Already-verified external-principal evidence accepted through the trusted
/// host boundary of one authority domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedExternalPrincipal {
    domain: AuthorityDomainHandle,
    authority: Box<[u8]>,
    subject: Box<[u8]>,
}

impl VerifiedExternalPrincipal {
    pub const fn domain(&self) -> AuthorityDomainHandle {
        self.domain
    }

    pub fn authority(&self) -> &[u8] {
        &self.authority
    }

    pub fn subject(&self) -> &[u8] {
        &self.subject
    }

    pub(crate) fn new(
        domain: AuthorityDomainHandle,
        authority: &[u8],
        subject: &[u8],
    ) -> Self {
        Self {
            domain,
            authority: authority.into(),
            subject: subject.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct PrincipalKey {
    pub authority: Box<[u8]>,
    pub subject: Box<[u8]>,
}

impl From<&VerifiedExternalPrincipal> for PrincipalKey {
    fn from(value: &VerifiedExternalPrincipal) -> Self {
        Self {
            authority: value.authority.clone(),
            subject: value.subject.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct IdAllocator {
    kind: IdKind,
    next: u64,
}

impl IdAllocator {
    pub(crate) const fn new(kind: IdKind) -> Self {
        Self { kind, next: 1 }
    }

    pub(crate) fn allocate(&mut self) -> Result<u64, AuthorityError> {
        if self.next == u64::MAX {
            return Err(AuthorityError::IdExhausted(self.kind));
        }

        let allocated = self.next;
        self.next += 1;
        Ok(allocated)
    }

    #[cfg(test)]
    pub(crate) fn set_next_for_test(&mut self, next: u64) {
        self.next = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_local_value_in_different_domains_is_not_equal() {
        let left = PlayerId::from_parts(AuthorityDomainHandle::new(1), 7);
        let right = PlayerId::from_parts(AuthorityDomainHandle::new(2), 7);

        assert_ne!(left, right);
        assert_eq!(left.local_value(), right.local_value());
    }

    #[test]
    fn allocator_fails_before_wrap_or_reuse() {
        let mut allocator = IdAllocator::new(IdKind::Player);
        allocator.set_next_for_test(u64::MAX - 1);

        assert_eq!(allocator.allocate().unwrap(), u64::MAX - 1);
        assert_eq!(
            allocator.allocate(),
            Err(AuthorityError::IdExhausted(IdKind::Player))
        );
    }
}
