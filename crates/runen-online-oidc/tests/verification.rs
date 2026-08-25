use std::time::{Duration, UNIX_EPOCH};

use runen_online::{
    AssociationOutcome, Authority, AuthorityDomainHandle, AuthorityError, AuthorityLimits,
    TimeDomainHandle,
};
use runen_online_oidc::{
    BoundedInput, NonceExpectation, OidcVerifier, VerificationError, VerifierConfig, VerifierLimits,
};
use serde_json::json;

const ISSUER: &str = "https://issuer.example";
const CLIENT_ID: &str = "client-1";
const JWKS: &str = r#"{"keys":[{"kty":"RSA","kid":"key-1","use":"sig","alg":"RS256","n":"qjqgd87WaEko3Te0hSX_SvG7Ivs9OcbOvpQzWHBPDEgGYa6K8BWWkPJXSebfum_V7WZbEd9Dc_H6Dw2PC7XlQ3D831jyMSJqc7ok18_wx-wdokyMqloFrBkvc3IaFletZ_Y2zez9a3yGo5cnkRLeiC3qY4tWYzFJxPmuTdvgtnLU5mdmU_2ilyW8atdCxK4TkxjnpnFWnDodWBWArlvLT3_05eLbMCkL5aec-LzYxSt4Q6TgKVy-Lhu_GuNKbnENdCEhCZgGsBIZ0-RdskazRDAAQEXvb9qrdZYOCiew-kWqnDLN3sM8zQnPwyzk_VXc9ZpfX6edwe-lzXHoE123nQ","e":"AQAB"}]}"#;

const VALID: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6ImtleS0xIn0.eyJpc3MiOiJodHRwczovL2lzc3Vlci5leGFtcGxlIiwic3ViIjoicGxheWVyLTEyMyIsImF1ZCI6ImNsaWVudC0xIiwiZXhwIjoyMDAsImlhdCI6MTAwLCJub25jZSI6Im5vbmNlLTEifQ.lFlF_sPNFG5cvIOZuXKfPWw5Eb5jXes6yyPvMza0kQj6kYRzs5eWE5aqgmEfQZX_LKNKqo6OKlY6SsGZueSnxY4ydO_WSRXYi_uzzo074GbCDK4KuovxbvWAl4HVpbyCy5H2V3BsQEQnEzrJ5-RbNhrWLO-KiMnXlEcrjwenvNG_wHqOpqlnSjIFvHuN-gC4CiuTCBofpL1Vt8n1Bt_G8ApYI-1pFc0o9j2TJ6PBXrZPELCd_hDOI-Tuu8oBFk6VtH3-RT4um99nbFLJ2wjPJuOOOuKef4FUr3QSF2v9sk_WekNwMVFnEUD-Ox0fh-ysbObDkrJN3BUGvG2auz2HhA";
const NO_NONCE: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6ImtleS0xIn0.eyJpc3MiOiJodHRwczovL2lzc3Vlci5leGFtcGxlIiwic3ViIjoicGxheWVyLTEyMyIsImF1ZCI6ImNsaWVudC0xIiwiZXhwIjoyMDAsImlhdCI6MTAwfQ.IdE1yy6b1XU1r0NsoSqJ5jn4ZEifpRilVQEINYQXvfN2D8HqsAwx6fRobAhk4f9Vk1XQ4GsZi69L_FAf8ZbPa5-NtkCxpZ0HLeO_YC0vmBGBCowjmakv008llrshm0orSoAhFvOOshaHMK4G7KC3EJlg7Irtg9eF0it6tySf554Irdygi8Lzb36GoZ5uvPzmHbQZvHy2796exjr4vTBamkdgYiwhckk_vQ5DiuaopdbiwYr-8r7A4kzwIbXYKlTSriWqo-rtJ4xBu9vqMfC4trCtbK4m01Gqq7XDjUNcqhlYEjXMYjsATMaW3gxcF44bmcGz3Y7uELQ73PpfS9PCWw";
const WRONG_ISSUER: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6ImtleS0xIn0.eyJpc3MiOiJodHRwczovL290aGVyLmV4YW1wbGUiLCJzdWIiOiJwbGF5ZXItMTIzIiwiYXVkIjoiY2xpZW50LTEiLCJleHAiOjIwMCwiaWF0IjoxMDAsIm5vbmNlIjoibm9uY2UtMSJ9.OrW3qHNh-OahpWoFBGxGlnNoe8de6YfOpZxBU04OFONRls5_yxW-RZRQF-jQEnln64i3FF4OB1DYlTMUwcj82D3z1HXNWyAWYRV9AW9BkT0j2rtr2xUjE6xonOIBVk4UGiBtGDoj2rixmVwkRrhCuGGBwcH9DsfiSGUvIHI_FLZgDlRoqpHHAeZUr_xCGdJDSbphJ550J3mzQgEwJ9gWZsoPS8yXvm3jBQ-tCGE2lYllWaU-T79BA_Ip_oXKCsfMYQwrLF3tfFZBd-DG29smPVa4mZbg0iTEb51lJVQbasQzwvnxRx4b4UE7mxSMaWNhr9KZCiXjWMOtogO7N8LRnA";
const WRONG_AUDIENCE: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6ImtleS0xIn0.eyJpc3MiOiJodHRwczovL2lzc3Vlci5leGFtcGxlIiwic3ViIjoicGxheWVyLTEyMyIsImF1ZCI6Im90aGVyLWNsaWVudCIsImV4cCI6MjAwLCJpYXQiOjEwMCwibm9uY2UiOiJub25jZS0xIn0.ZtVy-MBN4BjoHgscL32dn8DcZDgni-5yBigH_M6nwXgpofHd-ZS89FeT73vKyLdpTzI7aIK9o4_gM5v_m5Quy9tm5-j31XhlBXS6WNPBNSvkK7PzcMhIPHNpuB9BIOtClNdMs0qtlyzTca9IsyATnkCbpX1pRZn6-S98VqZyt56x_mjAkeSGe2f5LOtOAjO8vrFYAgoftvO-gkOMD19sMOlYKbc-GxzLprAEdAzkVNPPdHzSQhO56WY8yPYrSGhgOi801ORzIKl9Nngn7_3m26QvtUJ7hE0NRrnh4AF-K5OqQ665K2BRy0MK26yyBN8p0oIymLalDMpbEI7RRElBZA";
const MULTIPLE_AUDIENCES: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6ImtleS0xIn0.eyJpc3MiOiJodHRwczovL2lzc3Vlci5leGFtcGxlIiwic3ViIjoicGxheWVyLTEyMyIsImF1ZCI6WyJjbGllbnQtMSIsIm90aGVyLWNsaWVudCJdLCJleHAiOjIwMCwiaWF0IjoxMDAsIm5vbmNlIjoibm9uY2UtMSJ9.ImOZCBlgNYYujH42KEuudClzXORagUvI1IeYRH6eQc05WDq4itmhYas_mJOt6WGVF_KJ5osPpzalcjCuB66ZaTf_CtFf13Iv-8EqvxojoRycWfGH8vBFnafk6tU2Sc5jMZqw3HCLq6dk4SDT_uz34sfpZtXLA1fTbYh8_QR0T7qqFgTvokbReK6u3KsvJgOpIyPMT6bLDcDBmz0uIGiypjqX1P612wawViJwXH-OIUvsb7VErFpaSgULTolHRZmvBNtAG_MNXCUK92fOuBmJEYs3EdsIM72-x7zVGyzAI2nwBkNhP521fFbEZT_5YRfEPKN6xx_6Juqry_PmVJwdwg";
const MISSING_IAT: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6ImtleS0xIn0.eyJpc3MiOiJodHRwczovL2lzc3Vlci5leGFtcGxlIiwic3ViIjoicGxheWVyLTEyMyIsImF1ZCI6ImNsaWVudC0xIiwiZXhwIjoyMDAsIm5vbmNlIjoibm9uY2UtMSJ9.mEqw2XJYQou7v5Mf_QBkuxD_DPanxResmiUWVj39vc2jSso6mNNTGJqCT3Sd1v7o0EMgRcm-LXXdjc3oK3RNHUqZzrcGNMZlXu1PLkLccZ4viTj9MsvuRREeVodUbhTtKguP0uAVT5a10uerEZskK-MJxrGT_eduHB2b0ecmPtFQeaiWgLVYQ0TQfNwmAjSH96CUQ7nrFMossJNt25e630lPkvL0S-C2llB4aeUdlcHdyRUaSTAvlKgDxryjHec7NjCSWGX8GV-G3qXkghoaDowQ6ydOeV4BoVA5vhcyKeU2PKMaGIsFY_IGyB_p79nwuxVQjrPBknCPfeGvM8_r0g";
const EMPTY_SUBJECT: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6ImtleS0xIn0.eyJpc3MiOiJodHRwczovL2lzc3Vlci5leGFtcGxlIiwic3ViIjoiIiwiYXVkIjoiY2xpZW50LTEiLCJleHAiOjIwMCwiaWF0IjoxMDAsIm5vbmNlIjoibm9uY2UtMSJ9.GXqSUpDPXov1wYu69LkTtbpbSh8zmKbvFyi-0U-UbdR5v1a1dmP5J1U1OvT-1HMiCuVrQb1I5OY0sTiWD7C50Qwuovg3wVu1TNi4NehtTPv868fdnAATJOrTyR5u7nIrSH7WssHfiC13tJB3y9hOfSUR29OH5S_FIU7-8iU8j1SCZ6Tdi43aIFzW2l_l_lvh8jazlT6FQ8cGrNbDmHHvPtf_DMYPvqbcUNX8MwoZaagNIhqIX_j0eoi60bp1yTuEhXcjE7uxZZ-cnllvThqEwJeNQ1dC_GHhdcET-25Yo1S3Abw4nxENJhDyU-S5KMPEkYk9nC8qkpkWLCETlBAcmQ";
const UNKNOWN_KID: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Im90aGVyLWtleSJ9.eyJpc3MiOiJodHRwczovL2lzc3Vlci5leGFtcGxlIiwic3ViIjoicGxheWVyLTEyMyIsImF1ZCI6ImNsaWVudC0xIiwiZXhwIjoyMDAsImlhdCI6MTAwLCJub25jZSI6Im5vbmNlLTEifQ.F_bLImlEpebwG7zH06bIvdEAsMXUG-r6rNh4bD7Ufju9TYj5d7SSjWdWca3mOFX2suWZaB9mJMPFFyMulXumTSdCikklkD8_EYnCkxiMR4vHRqKNEQWP6gciSL868Tfk8mc1n3yzqo5SOIADTgYMb9AyhXDFLYqEK04DuNypOoAZIUFc9Qt5qyMo3hEzSQpEnAvPORghfZP_x0Haacb5jJ8r8hCkJitPgsr_146HkZ4TSCkFlY6-dmQ7yJnCCT3f_34ALLr1804mjuf287Se4BkYHRNc3dtM6nwHxWbTzmv11YsfzYXrciMjL7wyAiLxi6iukWq0mnKGgixiyQnfiQ";
const MISSING_KID: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJodHRwczovL2lzc3Vlci5leGFtcGxlIiwic3ViIjoicGxheWVyLTEyMyIsImF1ZCI6ImNsaWVudC0xIiwiZXhwIjoyMDAsImlhdCI6MTAwLCJub25jZSI6Im5vbmNlLTEifQ.WCEIJle8OwnGbOOZaFK8MaNSoukNjDxcOba-gmcaorDVf8VRXIRZpAwPEysIh2933XkDz0Ua0wxvejx7Xs_99FIHeWs6QsuVxxw15k76QfjdFVYPrRdUmg8SI_IENGdOLPZ0bZMtPrsefsvHvnvFneHj1PDT2cl9ukszYlCnYHlhBlenWZ_Q_zQF0PlTYpZS4FhXF3T2QsgIli7S3m7Gmvmk8hx1W_X4MY1qOmXh7wPe58iMR3-xajD19ZM-6SinuAQW8OK56-_xe4foLB_ooqh707fgD12HirAcvYcfNE7MOphhTtxmAVSR4qzHDEqc9WsxXdQ0OaVBRHaMVMs_Ew";
const BAD_SIGNATURE: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6ImtleS0xIn0.eyJpc3MiOiJodHRwczovL2lzc3Vlci5leGFtcGxlIiwic3ViIjoicGxheWVyLTEyMyIsImF1ZCI6ImNsaWVudC0xIiwiZXhwIjoyMDAsImlhdCI6MTAwLCJub25jZSI6Im5vbmNlLTEifQ.lFlF_sPNFG5cvIOZuXKfPWw5Eb5jXes6yyPvMza0kQj6kYRzs5eWE5aqgmEfQZX_LKNKqo6OKlY6SsGZueSnxY4ydO_WSRXYi_uzzo074GbCDK4KuovxbvWAl4HVpbyCy5H2V3BsQEQnEzrJ5-RbNhrWLO-KiMnXlEcrjwenvNG_wHqOpqlnSjIFvHuN-gC4CiuTCBofpL1Vt8n1Bt_G8ApYI-1pFc0o9j2TJ6PBXrZPELCd_hDOI-Tuu8oBFk6VtH3-RT4um99nbFLJ2wjPJuOOOuKef4FUr3QSF2v9sk_WekNwMVFnEUD-Ox0fh-ysbObDkrJN3BUGvG2auz2HhB";

fn verifier() -> OidcVerifier {
    OidcVerifier::new(config(4096, 4096, 4), JWKS.as_bytes()).unwrap()
}

fn config(max_token: usize, max_jwks: usize, max_keys: usize) -> VerifierConfig<'static> {
    VerifierConfig {
        expected_issuer: ISSUER,
        expected_client_id: CLIENT_ID,
        limits: VerifierLimits {
            max_id_token_bytes: max_token,
            max_jwks_bytes: max_jwks,
            max_jwk_count: max_keys,
        },
    }
}

fn authority(trusted_issuers: &[&[u8]]) -> Authority {
    Authority::new(
        AuthorityDomainHandle::new(),
        TimeDomainHandle::new(),
        AuthorityLimits {
            max_trusted_external_authorities: 4,
            max_external_authority_bytes: 128,
            max_external_subject_bytes: 128,
            max_players: 8,
            max_principal_associations: 8,
            max_principal_associations_per_player: 4,
            max_assignments: 0,
            max_pending_assignment_lifetime: 0,
            max_admission_grants: 0,
            max_admission_grant_lifetime: 0,
            max_live_admission_grants_per_player: 0,
            max_live_admission_grants_per_assignment: 0,
            max_match_requests: 0,
            max_match_request_lifetime: 0,
            max_match_request_cohort: 0,
            max_matchmaking_input_bytes: 0,
            max_pending_match_requests_per_player: 0,
            max_match_candidate_requests: 0,
            max_match_roster_players: 0,
            max_matches: 0,
        },
        trusted_issuers.iter().copied(),
    )
    .unwrap()
}

fn at(seconds: u64) -> std::time::SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

#[test]
fn valid_rs256_id_token_maps_exact_issuer_and_subject() {
    let authority = authority(&[ISSUER.as_bytes()]);
    let principal = verifier()
        .verify(
            &authority,
            VALID,
            NonceExpectation::Exact("nonce-1"),
            at(150),
        )
        .unwrap();

    assert_eq!(principal.authority(), ISSUER.as_bytes());
    assert_eq!(principal.subject(), b"player-123");
}

#[test]
fn adapter_verification_does_not_create_or_associate_player_state() {
    let mut authority = authority(&[ISSUER.as_bytes()]);
    let principal = verifier()
        .verify(
            &authority,
            VALID,
            NonceExpectation::Exact("nonce-1"),
            at(150),
        )
        .unwrap();

    let player = authority.create_player().unwrap();
    assert_eq!(player.local_value(), 1);
    assert_eq!(
        authority.associate_principal(&player, &principal).unwrap(),
        AssociationOutcome::Associated
    );
    assert_eq!(authority.resolve_principal(&principal).unwrap(), Some(player));
}

#[test]
fn cryptographically_valid_untrusted_issuer_fails_at_core_handoff() {
    let authority = authority(&[b"https://trusted.example"]);
    assert_eq!(
        verifier().verify(
            &authority,
            VALID,
            NonceExpectation::Exact("nonce-1"),
            at(150),
        ),
        Err(VerificationError::PrincipalRejected(
            AuthorityError::UntrustedExternalAuthority
        ))
    );
}

#[test]
fn issuer_audience_and_subject_profile_fail_closed() {
    let authority = authority(&[ISSUER.as_bytes()]);
    for token in [WRONG_ISSUER, WRONG_AUDIENCE, EMPTY_SUBJECT] {
        assert_eq!(
            verifier().verify(
                &authority,
                token,
                NonceExpectation::Exact("nonce-1"),
                at(150),
            ),
            Err(VerificationError::VerificationFailed)
        );
    }
    assert_eq!(
        verifier().verify(
            &authority,
            MULTIPLE_AUDIENCES,
            NonceExpectation::Exact("nonce-1"),
            at(150),
        ),
        Err(VerificationError::UnsupportedTokenProfile)
    );
}

#[test]
fn required_oidc_claims_and_expiry_use_explicit_host_time() {
    let authority = authority(&[ISSUER.as_bytes()]);
    assert_eq!(
        verifier().verify(
            &authority,
            MISSING_IAT,
            NonceExpectation::Exact("nonce-1"),
            at(150),
        ),
        Err(VerificationError::VerificationFailed)
    );
    for verification_time in [200, 201] {
        assert_eq!(
            verifier().verify(
                &authority,
                VALID,
                NonceExpectation::Exact("nonce-1"),
                at(verification_time),
            ),
            Err(VerificationError::VerificationFailed)
        );
    }
}

#[test]
fn signature_and_key_selection_fail_closed() {
    let authority = authority(&[ISSUER.as_bytes()]);
    assert_eq!(
        verifier().verify(
            &authority,
            BAD_SIGNATURE,
            NonceExpectation::Exact("nonce-1"),
            at(150),
        ),
        Err(VerificationError::VerificationFailed)
    );
    for token in [UNKNOWN_KID, MISSING_KID] {
        assert_eq!(
            verifier().verify(
                &authority,
                token,
                NonceExpectation::Exact("nonce-1"),
                at(150),
            ),
            Err(VerificationError::UnsupportedTokenProfile)
        );
    }
}

#[test]
fn nonce_policy_is_explicit_and_exact() {
    let authority = authority(&[ISSUER.as_bytes()]);
    assert_eq!(
        verifier().verify(&authority, VALID, NonceExpectation::Absent, at(150)),
        Err(VerificationError::VerificationFailed)
    );
    assert_eq!(
        verifier().verify(
            &authority,
            VALID,
            NonceExpectation::Exact("other-nonce"),
            at(150),
        ),
        Err(VerificationError::VerificationFailed)
    );
    assert!(
        verifier()
            .verify(&authority, NO_NONCE, NonceExpectation::Absent, at(150))
            .is_ok()
    );
}

#[test]
fn raw_token_and_jwks_bounds_are_checked_before_parse() {
    let authority = authority(&[ISSUER.as_bytes()]);
    let token_limited =
        OidcVerifier::new(config(VALID.len() - 1, 4096, 4), JWKS.as_bytes()).unwrap();
    assert_eq!(
        token_limited.verify(
            &authority,
            VALID,
            NonceExpectation::Exact("nonce-1"),
            at(150),
        ),
        Err(VerificationError::InputTooLarge(BoundedInput::IdToken))
    );

    assert!(matches!(
        OidcVerifier::new(config(4096, JWKS.len() - 1, 4), JWKS.as_bytes()),
        Err(VerificationError::InputTooLarge(BoundedInput::Jwks))
    ));
}

#[test]
fn jwks_activation_rejects_excessive_and_duplicate_keys() {
    let value: serde_json::Value = serde_json::from_str(JWKS).unwrap();
    let key = value["keys"][0].clone();
    let duplicate = serde_json::to_vec(&json!({ "keys": [key.clone(), key] })).unwrap();

    assert!(matches!(
        OidcVerifier::new(config(4096, 8192, 1), &duplicate),
        Err(VerificationError::InvalidConfiguration)
    ));
    assert!(matches!(
        OidcVerifier::new(config(4096, 8192, 4), &duplicate),
        Err(VerificationError::InvalidConfiguration)
    ));
}

#[test]
fn verifier_rejects_zero_limits_empty_keysets_and_non_rs256_keys() {
    assert!(matches!(
        OidcVerifier::new(config(0, 4096, 4), JWKS.as_bytes()),
        Err(VerificationError::InvalidConfiguration)
    ));
    assert!(matches!(
        OidcVerifier::new(config(4096, 4096, 4), br#"{"keys":[]}"#),
        Err(VerificationError::InvalidConfiguration)
    ));

    let unsupported = JWKS.replace("\"RS256\"", "\"RS512\"");
    assert!(matches!(
        OidcVerifier::new(config(4096, 4096, 4), unsupported.as_bytes()),
        Err(VerificationError::UnsupportedJwk)
    ));
}
