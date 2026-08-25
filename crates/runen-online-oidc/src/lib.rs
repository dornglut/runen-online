#![forbid(unsafe_code)]

//! Bounded OIDC Core ID-token verification adapter for RunenOnline.
//!
//! The host is responsible for obtaining ID tokens and for supplying and
//! refreshing the static JWKS used by this verifier. This crate performs no
//! discovery, HTTP, login-flow, persistence, service, or runtime work. A raw
//! ID token is provider credential input; it is not a standardized RunenOnline
//! wire credential.
//!
//! Successful verification maps only the exact OIDC `iss` + `sub` pair through
//! [`runen_online::Authority::accept_verified_external_principal`]. The owning
//! RunenOnline authority therefore remains responsible for deciding whether the
//! issuer is trusted and whether the mapped representations fit its bounds.

use std::collections::BTreeMap;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::jwk::{AlgorithmParameters, JwkSet, KeyAlgorithm, KeyOperations, PublicKeyUse};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use runen_online::{Authority, AuthorityError, VerifiedExternalPrincipal};
use serde::Deserialize;

/// Finite host policy applied before parsing or retaining remotely influenced
/// OIDC material.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifierLimits {
    pub max_id_token_bytes: usize,
    pub max_jwks_bytes: usize,
    pub max_jwk_count: usize,
}

/// Trusted host configuration for one verifier instance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifierConfig<'a> {
    pub expected_issuer: &'a str,
    pub expected_client_id: &'a str,
    pub limits: VerifierLimits,
}

/// Host-owned nonce policy for one ID-token verification operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NonceExpectation<'a> {
    /// Reject any token that carries a nonce.
    Absent,
    /// Require the token nonce to equal this value exactly.
    Exact(&'a str),
}

/// Input category rejected at an explicit byte bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundedInput {
    IdToken,
    Jwks,
}

/// Adapter-local diagnostics. These variants are not portable RunenOnline
/// authentication semantics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationError {
    InvalidConfiguration,
    InputTooLarge(BoundedInput),
    MalformedJwks,
    UnsupportedJwk,
    MalformedToken,
    UnsupportedTokenProfile,
    VerificationFailed,
    PrincipalRejected(AuthorityError),
}

impl fmt::Display for VerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfiguration => {
                formatter.write_str("invalid OIDC verifier configuration")
            }
            Self::InputTooLarge(BoundedInput::IdToken) => {
                formatter.write_str("ID token exceeds configured byte bound")
            }
            Self::InputTooLarge(BoundedInput::Jwks) => {
                formatter.write_str("JWKS exceeds configured byte bound")
            }
            Self::MalformedJwks => formatter.write_str("malformed JWKS"),
            Self::UnsupportedJwk => formatter.write_str("unsupported JWK profile"),
            Self::MalformedToken => formatter.write_str("malformed ID token"),
            Self::UnsupportedTokenProfile => formatter.write_str("unsupported ID-token profile"),
            Self::VerificationFailed => formatter.write_str("ID-token verification failed"),
            Self::PrincipalRejected(_) => {
                formatter.write_str("RunenOnline rejected the verified external principal")
            }
        }
    }
}

impl std::error::Error for VerificationError {}

/// Static-JWKS verifier for the bounded first RO4 OIDC profile.
pub struct OidcVerifier {
    expected_issuer: Box<str>,
    expected_client_id: Box<str>,
    max_id_token_bytes: usize,
    keys_by_id: BTreeMap<Box<str>, DecodingKey>,
}

impl OidcVerifier {
    /// Constructs a verifier from trusted host configuration and a bounded,
    /// already-obtained JWKS document.
    pub fn new(config: VerifierConfig<'_>, jwks_json: &[u8]) -> Result<Self, VerificationError> {
        if config.expected_issuer.is_empty()
            || config.expected_client_id.is_empty()
            || config.limits.max_id_token_bytes == 0
            || config.limits.max_jwks_bytes == 0
            || config.limits.max_jwk_count == 0
        {
            return Err(VerificationError::InvalidConfiguration);
        }
        if jwks_json.len() > config.limits.max_jwks_bytes {
            return Err(VerificationError::InputTooLarge(BoundedInput::Jwks));
        }

        let jwks: JwkSet =
            serde_json::from_slice(jwks_json).map_err(|_| VerificationError::MalformedJwks)?;
        if jwks.keys.is_empty() || jwks.keys.len() > config.limits.max_jwk_count {
            return Err(VerificationError::InvalidConfiguration);
        }

        let mut keys_by_id = BTreeMap::new();
        for jwk in &jwks.keys {
            require_rs256_verification_jwk(jwk)?;
            let key_id = jwk
                .common
                .key_id
                .as_deref()
                .filter(|key_id| !key_id.is_empty())
                .ok_or(VerificationError::UnsupportedJwk)?;
            let decoding_key =
                DecodingKey::from_jwk(jwk).map_err(|_| VerificationError::UnsupportedJwk)?;
            if keys_by_id
                .insert(Box::<str>::from(key_id), decoding_key)
                .is_some()
            {
                return Err(VerificationError::InvalidConfiguration);
            }
        }

        Ok(Self {
            expected_issuer: config.expected_issuer.into(),
            expected_client_id: config.expected_client_id.into(),
            max_id_token_bytes: config.limits.max_id_token_bytes,
            keys_by_id,
        })
    }

    /// Verifies one bounded ID token at an explicit host-supplied authentication
    /// time and hands the verified issuer/subject pair to the owning authority.
    ///
    /// `verification_time` is OIDC realization time only. It is not converted
    /// into RunenOnline `TrustedTime` and carries no Assignment, AdmissionGrant,
    /// or matchmaking deadline authority.
    pub fn verify(
        &self,
        authority: &Authority,
        raw_id_token: &str,
        nonce: NonceExpectation<'_>,
        verification_time: SystemTime,
    ) -> Result<VerifiedExternalPrincipal, VerificationError> {
        if raw_id_token.len() > self.max_id_token_bytes {
            return Err(VerificationError::InputTooLarge(BoundedInput::IdToken));
        }

        let header = decode_header(raw_id_token).map_err(|_| VerificationError::MalformedToken)?;
        if header.alg != Algorithm::RS256 {
            return Err(VerificationError::UnsupportedTokenProfile);
        }
        let key_id = header
            .kid
            .as_deref()
            .filter(|key_id| !key_id.is_empty())
            .ok_or(VerificationError::UnsupportedTokenProfile)?;
        let decoding_key = self
            .keys_by_id
            .get(key_id)
            .ok_or(VerificationError::UnsupportedTokenProfile)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.leeway = 0;
        // jsonwebtoken validates expiration against its own wall clock. RO4B
        // instead requires the explicit trusted host value supplied above, so
        // only that library-internal time check is disabled. Presence and the
        // exact comparison are enforced below after signature verification.
        validation.validate_exp = false;
        validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
        validation.set_issuer(&[self.expected_issuer.as_ref()]);
        validation.set_audience(&[self.expected_client_id.as_ref()]);

        let claims = decode::<IdTokenClaims>(raw_id_token, decoding_key, &validation)
            .map_err(|_| VerificationError::VerificationFailed)?
            .claims;

        if claims.iss != self.expected_issuer.as_ref() || claims.sub.is_empty() {
            return Err(VerificationError::VerificationFailed);
        }
        match &claims.aud {
            Audience::Single(audience) if audience == self.expected_client_id.as_ref() => {}
            Audience::Multiple(audiences)
                if audiences.len() == 1
                    && audiences[0] == self.expected_client_id.as_ref() => {}
            Audience::Multiple(audiences) if audiences.len() > 1 => {
                return Err(VerificationError::UnsupportedTokenProfile);
            }
            Audience::Single(_) | Audience::Multiple(_) => {
                return Err(VerificationError::VerificationFailed);
            }
        }

        let verification_seconds = verification_time
            .duration_since(UNIX_EPOCH)
            .map_err(|_| VerificationError::VerificationFailed)?
            .as_secs();
        if verification_seconds >= claims.exp {
            return Err(VerificationError::VerificationFailed);
        }

        // `iat` is required by the OIDC ID-token profile. Deserializing this
        // concrete field above establishes presence and numeric representation.
        let _issued_at = claims.iat;

        match (nonce, claims.nonce.as_deref()) {
            (NonceExpectation::Absent, None) => {}
            (NonceExpectation::Exact(expected), Some(actual)) if expected == actual => {}
            _ => return Err(VerificationError::VerificationFailed),
        }

        authority
            .accept_verified_external_principal(claims.iss.as_bytes(), claims.sub.as_bytes())
            .map_err(VerificationError::PrincipalRejected)
    }
}

fn require_rs256_verification_jwk(jwk: &jsonwebtoken::jwk::Jwk) -> Result<(), VerificationError> {
    if !matches!(jwk.algorithm, AlgorithmParameters::RSA(_)) {
        return Err(VerificationError::UnsupportedJwk);
    }
    if let Some(algorithm) = jwk.common.key_algorithm
        && algorithm != KeyAlgorithm::RS256
    {
        return Err(VerificationError::UnsupportedJwk);
    }
    if let Some(key_use) = &jwk.common.public_key_use
        && key_use != &PublicKeyUse::Signature
    {
        return Err(VerificationError::UnsupportedJwk);
    }
    if let Some(operations) = &jwk.common.key_operations
        && (operations.is_empty()
            || operations
                .iter()
                .any(|operation| operation != &KeyOperations::Verify))
    {
        return Err(VerificationError::UnsupportedJwk);
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct IdTokenClaims {
    iss: String,
    sub: String,
    aud: Audience,
    exp: u64,
    iat: u64,
    #[serde(default)]
    nonce: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Audience {
    Single(String),
    Multiple(Vec<String>),
}
