use std::{fmt, sync::Arc};

use crate::{AuthorityError, ResourceLimit};

#[derive(Debug)]
struct TimeDomainMarker {
    _identity_byte: u8,
}

/// Opaque process-local comparison-domain token for trusted time observations.
///
/// Clones intentionally identify the same comparison domain. Independently
/// constructed handles are never equal merely because a caller reused a
/// numeric label. No epoch, unit, clock source, synchronization mechanism, or
/// wire representation is standardized by this type.
#[derive(Clone)]
pub struct TimeDomainHandle(Arc<TimeDomainMarker>);

impl TimeDomainHandle {
    pub fn new() -> Self {
        Self(Arc::new(TimeDomainMarker { _identity_byte: 0 }))
    }
}

impl Default for TimeDomainHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for TimeDomainHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("TimeDomainHandle")
            .field(&(Arc::as_ptr(&self.0) as usize))
            .finish()
    }
}

impl PartialEq for TimeDomainHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for TimeDomainHandle {}

/// Explicit trusted host observation used for deadline decisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustedTime {
    domain: TimeDomainHandle,
    value: u64,
}

impl TrustedTime {
    /// Constructs a trusted observation supplied by the host boundary.
    ///
    /// RunenOnline does not independently verify the clock source. The host
    /// must create observations from the same `TimeDomainHandle` whose ordering
    /// contract it uses for the authority.
    pub const fn new(domain: TimeDomainHandle, value: u64) -> Self {
        Self { domain, value }
    }

    pub fn domain(&self) -> &TimeDomainHandle {
        &self.domain
    }

    pub const fn value(&self) -> u64 {
        self.value
    }
}

pub(crate) fn require_time_domain(
    observation: &TrustedTime,
    expected: &TimeDomainHandle,
) -> Result<(), AuthorityError> {
    if observation.domain() != expected {
        return Err(AuthorityError::TimeDomainMismatch);
    }
    Ok(())
}

pub(crate) fn validate_deadline(
    observation: &TrustedTime,
    expected_domain: &TimeDomainHandle,
    deadline: u64,
    max_lifetime: u64,
    limit: ResourceLimit,
) -> Result<(), AuthorityError> {
    require_time_domain(observation, expected_domain)?;

    if deadline <= observation.value() {
        return Err(AuthorityError::InvalidDeadline);
    }

    let lifetime = deadline
        .checked_sub(observation.value())
        .ok_or(AuthorityError::InvalidDeadline)?;
    if lifetime > max_lifetime {
        return Err(AuthorityError::ResourceLimit(limit));
    }

    Ok(())
}

pub(crate) fn deadline_reached(
    observation: &TrustedTime,
    expected_domain: &TimeDomainHandle,
    deadline: u64,
) -> Result<bool, AuthorityError> {
    require_time_domain(observation, expected_domain)?;
    Ok(observation.value() >= deadline)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deadline_is_reached_at_exact_boundary() {
        let domain = TimeDomainHandle::new();
        assert!(!deadline_reached(&TrustedTime::new(domain.clone(), 4), &domain, 5).unwrap());
        assert!(deadline_reached(&TrustedTime::new(domain.clone(), 5), &domain, 5).unwrap());
    }

    #[test]
    fn mismatched_comparison_domain_fails_closed() {
        let observation_domain = TimeDomainHandle::new();
        let authority_domain = TimeDomainHandle::new();
        assert_eq!(
            deadline_reached(
                &TrustedTime::new(observation_domain, 5),
                &authority_domain,
                10,
            ),
            Err(AuthorityError::TimeDomainMismatch)
        );
    }

    #[test]
    fn lifetime_limit_uses_overflow_safe_distance() {
        let domain = TimeDomainHandle::new();
        let now = TrustedTime::new(domain.clone(), u64::MAX - 5);

        assert_eq!(
            validate_deadline(
                &now,
                &domain,
                u64::MAX,
                4,
                ResourceLimit::AdmissionGrantLifetime,
            ),
            Err(AuthorityError::ResourceLimit(
                ResourceLimit::AdmissionGrantLifetime
            ))
        );
        assert!(validate_deadline(
            &now,
            &domain,
            u64::MAX,
            5,
            ResourceLimit::AdmissionGrantLifetime,
        )
        .is_ok());
    }
}