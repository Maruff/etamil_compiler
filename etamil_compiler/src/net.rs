// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//! Signing and outbound HTTP: the two host primitives a payment integration
//! cannot be written without.
//!
//! Both are genuinely Layer 0. HMAC needs bytes and a constant-time
//! comparison, neither of which eTamil has; opening a socket is a syscall.
//! Everything above them — which gateway, which fields, what a webhook means —
//! stays in the language.

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

/// Sign a message with HMAC-SHA256, returning lowercase hex.
///
/// Hex rather than base64 because that is what the gateways send: Razorpay and
/// Stripe both put a hex digest in the signature header, so a caller can
/// compare what arrives against this directly without a decoding step the
/// language would have to provide.
pub fn sign(key: &str, message: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(key.as_bytes())
        .expect("HMAC accepts a key of any length");
    mac.update(message.as_bytes());

    mac.finalize()
        .into_bytes()
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect()
}

/// Does `signature` match the message?
///
/// The comparison is constant-time. A byte-by-byte `==` returns as soon as it
/// finds a difference, and the time it took says how many leading characters
/// were right — enough to recover a signature one position at a time. That is
/// the entire reason this is a host primitive rather than `==` in eTamil.
///
/// Case-insensitive on the hex, since gateways differ on which they send.
pub fn verify(key: &str, message: &str, signature: &str) -> bool {
    let expected = sign(key, message);
    let given = signature.trim().to_lowercase();

    // Lengths are compared first and in the clear: the length of a signature
    // is not a secret, and ct_eq needs equal-length inputs to be meaningful.
    if expected.len() != given.len() {
        return false;
    }
    expected.as_bytes().ct_eq(given.as_bytes()).into()
}

/// One HTTP response, as the language will see it.
pub struct Response {
    pub status: u16,
    pub body: String,
    pub headers: Vec<(String, String)>,
}

/// Send a request and read the response.
///
/// A non-2xx status is **not** an error: a gateway rejecting a charge answers
/// 402 with a body explaining why, and turning that into a failure would throw
/// the explanation away. Only a request that never got an answer — DNS, TLS,
/// connection, timeout — is a `தவறு`.
#[cfg(feature = "http-client")]
pub fn request(
    method: &str,
    url: &str,
    body: Option<&str>,
    headers: &[(String, String)],
) -> Result<Response, String> {
    // A request with no timeout can hang a worker thread for as long as the
    // other end feels like holding the socket open.
    let mut builder = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(30));

    // A bank will not discuss an account with a caller it cannot identify, so
    // when a client certificate is configured every request offers it. With
    // none configured this is untouched, and HTTPS works as it always did
    // against the public roots.
    if let Some(identity) = crate::mtls::client_config() {
        builder = builder.tls_config(identity);
    }

    let agent = builder.build();

    let mut request = agent.request(method, url);
    for (name, value) in headers {
        request = request.set(name, value);
    }

    let outcome = match body {
        Some(body) => request.send_string(body),
        None => request.call(),
    };

    // ureq reports a non-2xx as Err(Status(..)); that is an answer, so it is
    // unwrapped back into a response rather than reported as a failure.
    let response = match outcome {
        Ok(response) => response,
        Err(ureq::Error::Status(_, response)) => response,
        Err(ureq::Error::Transport(transport)) => {
            return Err(format!(
                "வலைக் கோரிக்கை தோல்வி  (the request never completed): {}",
                transport
            ));
        }
    };

    let status = response.status();
    let headers = response
        .headers_names()
        .into_iter()
        .filter_map(|name| {
            response
                .header(&name)
                .map(|value| (name.to_lowercase(), value.to_string()))
        })
        .collect();

    // Read the body after the headers: into_string consumes the response.
    let body = response.into_string().map_err(|e| {
        format!("பதிலைப் படிக்க முடியவில்லை  (cannot read the response body): {}", e)
    })?;

    Ok(Response { status, body, headers })
}

#[cfg(not(feature = "http-client"))]
pub fn request(
    _method: &str,
    _url: &str,
    _body: Option<&str>,
    _headers: &[(String, String)],
) -> Result<Response, String> {
    Err("வலை ஆதரவு இல்லாமல் கட்டப்பட்டது  \
         (this build has no HTTP client): rebuild with --features http-client"
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // The published Razorpay example, so the digest is checked against a value
    // this project did not produce.
    #[test]
    fn hmac_matches_a_known_digest() {
        let signature = sign("secret", "hello");
        assert_eq!(
            signature,
            "88aab3ede8d3adf94d26ab90d3bafd4a2083070c3bcce9c014ee04a443847c0b"
        );
        assert_eq!(signature.len(), 64, "SHA-256 hex is 64 characters");
    }

    #[test]
    fn verify_accepts_only_the_right_signature() {
        let good = sign("secret", "order_id=1&amount=500");

        assert!(verify("secret", "order_id=1&amount=500", &good));
        assert!(!verify("wrong-key", "order_id=1&amount=500", &good));
        assert!(!verify("secret", "order_id=1&amount=999", &good));
        assert!(!verify("secret", "order_id=1&amount=500", "deadbeef"));
        assert!(!verify("secret", "order_id=1&amount=500", ""));
    }

    // Gateways disagree about the case of the hex they send.
    #[test]
    fn verify_ignores_the_case_of_the_signature() {
        let signature = sign("secret", "payload");
        assert!(verify("secret", "payload", &signature.to_uppercase()));
        assert!(verify("secret", "payload", &format!("  {}  ", signature)));
    }

    // A key longer than the block size is hashed down rather than rejected.
    #[test]
    fn a_long_key_is_accepted() {
        let key = "k".repeat(200);
        let signature = sign(&key, "payload");
        assert!(verify(&key, "payload", &signature));
    }
}
