use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::{AuthorityError, IdKind};

#[derive(Debug)]
struct DomainMarker {
    _identity_byte: u8,
}

/// Process-local identity token whose equality is allocation identity rather
/// than caller-chosen numeric data.
///
/// Every semantic ID keeps an `Arc` to this marker, so a stale ID keeps its
/// domain token alive and the allocator cannot recycle that token's address
/// while the stale ID still exists.
#[derive(Clone)]
struct DomainIdentity(Arc<DomainMarker>);

impl DomainIdentity {
    fn fresh() -> Self {
        Self(Arc::new(DomainMarker { _identity_byte: 0 }))
    }

    fn address(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
    }
}

impl fmt::Debug for DomainIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("DomainIdentity")
            .field(&self.address())
            .finish()
    }
}

impl PartialEq for DomainIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for DomainIdentity {}

impl PartialOrd for DomainIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DomainIdentity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.address().cmp(&other.address())
    }
}

impl Hash for DomainIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address().hash(state);
    }
}

/// Linear construction capability for one in-process RunenOnline authority
/// domain.
///
/// `AuthorityDomainHandle` is deliberately not `Clone` or `Copy`. Constructing
/// an `Authority` consumes the handle, which prevents safe code from creating
/// two independent authority aggregates that mint colliding semantic IDs for
/// the same domain token. Create a fresh handle for a distinct authority
/// domain.
///
/// This is an implementation-local capability. It is not a standardized realm,
/// provider, deployment, wire, or storage identifier.
#[derive(Debug)]
pub struct AuthorityDomainHandle {
    identity: DomainIdentity,
}

impl AuthorityDomainHandle {
    pub fn new() -> Self {
        Self {
            identity: DomainIdentity::fresh(),
        }
    }

    pub fn id(&self) -> AuthorityDomainId {
        AuthorityDomainId(self.identity.clone())
    }

    pub(crate) fn into_id(self) -> AuthorityDomainId {
        AuthorityDomainId(self.identity)
    }
}

impl Default for AuthorityDomainHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// Comparable process-local identity of an established authority domain.
///
/// This value can be cloned and carried by semantic IDs, but it cannot be used
/// to construct another `Authority`; construction requires the linear
/// `AuthorityDomainHandle` capability above.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AuthorityDomainId(DomainIdentity);

macro_rules! semantic_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name {
            domain: AuthorityDomainId,
            local_value: u64,
        }

        impl $name {
            pub fn domain(&self) -> &AuthorityDomainId {
                &self.domain
            }

            /// Returns the local implementation incarnation value.
            ///
            /// This value has meaning only together with `domain()` and does
            /// not establish a wire or storage representation.
            pub const fn local_value(&self) -> u64 {
                self.local_value
            }

            pub(crate) fn from_parts(domain: AuthorityDomainId, local_value: u64) -> Self {
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
    domain: AuthorityDomainId,
    authority: Box<[u8]>,
    subject: Box<[u8]>,
}

impl VerifiedExternalPrincipal {
    pub fn domain(&self) -> &AuthorityDomainId {
        &self.domain
    }

    pub fn authority(&self) -> &[u8] {
        &self.authority
    }

    pub fn subject(&self) -> &[u8] {
        &self.subject
    }

    pub(crate) fn new(domain: AuthorityDomainId, authority: &[u8], subject: &[u8]) -> Self {
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
    fn independently_constructed_domains_cannot_compare_equal() {
        let left_domain = AuthorityDomainHandle::new().into_id();
        let right_domain = AuthorityDomainHandle::new().into_id();
        let left = PlayerId::from_parts(left_domain, 7);
        let right = PlayerId::from_parts(right_domain, 7);

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