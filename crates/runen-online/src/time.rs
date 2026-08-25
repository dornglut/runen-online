use crate::{AuthorityError, ResourceLimit};

/// Local handle naming the comparison domain for trusted time observations.
///
/// It does not standardize an epoch, unit, clock source, synchronization
/// mechanism, or wire representation.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct TimeDomainHandle(u64);

impl TimeDomainHandle {
    pub const fn new(local_value: u64) -> Self {
        Self(local_value)
    }

    pub const fn local_value(self) -> u64 {
        self.0
    }
}

/// Explicit trusted host observation used for deadline decisions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrustedTime {
    domain: TimeDomainHandle,
    value: u64,
}

impl TrustedTime {
    /// Constructs a trusted observation supplied by the host boundary.
    ///
    /// RunenOnline does not independently verify the clock source.
    pub const fn new(domain: TimeDomainHandle, value: u64) -> Self {
        Self { domain, value }
    }

    pub const fn domain(self) -> TimeDomainHandle {
        self.domain
    }

    pub const fn value(self) -> u64 {
        self.value
    }
}

pub(crate) fn require_time_domain(
    observation: TrustedTime,
    expected: TimeDomainHandle,
) -> Result<(), AuthorityError> {
    if observation.domain() != expected {
        return Err(AuthorityError::TimeDomainMismatch);
    }
    Ok(())
}

pub(crate) fn validate_deadline(
    observation: TrustedTime,
    expected_domain: TimeDomainHandle,
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
    observation: TrustedTime,
    expected_domain: TimeDomainHandle,
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
        let domain = TimeDomainHandle::new(9);
        assert!(!deadline_reached(TrustedTime::new(domain, 4), domain, 5).unwrap());
        assert!(deadline_reached(TrustedTime::new(domain, 5), domain, 5).unwrap());
    }

    #[test]
    fn mismatched_comparison_domain_fails_closed() {
        assert_eq!(
            deadline_reached(
                TrustedTime::new(TimeDomainHandle::new(1), 5),
                TimeDomainHandle::new(2),
                10,
            ),
            Err(AuthorityError::TimeDomainMismatch)
        );
    }

    #[test]
    fn lifetime_limit_uses_overflow_safe_distance() {
        let domain = TimeDomainHandle::new(1);
        let now = TrustedTime::new(domain, u64::MAX - 5);

        assert_eq!(
            validate_deadline(
                now,
                domain,
                u64::MAX,
                4,
                ResourceLimit::AdmissionGrantLifetime,
            ),
            Err(AuthorityError::ResourceLimit(
                ResourceLimit::AdmissionGrantLifetime
            ))
        );
        assert!(validate_deadline(
            now,
            domain,
            u64::MAX,
            5,
            ResourceLimit::AdmissionGrantLifetime,
        )
        .is_ok());
    }
}
