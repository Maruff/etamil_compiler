// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
//! Mutual TLS — proving who the *client* is.
//!
//! Ordinary HTTPS proves the server is who it claims to be. A bank wants the
//! other direction as well: before it will discuss an account it wants the
//! caller to present a certificate it has issued. Every Indian bank API, and
//! every UPI payment service provider, works this way — a client certificate
//! is not an option they offer but the door they open.
//!
//! Configured through the environment rather than the language, for the same
//! reason ETAMIL_JWT_SECRET is: a private key is a deployment secret, and a
//! program that names one has put it somewhere it can be read.
//!
//!   ETAMIL_TLS_CERT   the client certificate chain, PEM
//!   ETAMIL_TLS_KEY    its private key, PEM (PKCS#8, SEC1 or PKCS#1)
//!   ETAMIL_TLS_CA     a private CA to trust as well, PEM — optional
//!
//! The two halves are independent. Set none of them and nothing changes:
//! outbound HTTPS works exactly as it did, against the public roots. Set the
//! certificate and its key and every outbound request offers that identity.
//! Set only the CA and a private root is trusted without presenting anything —
//! which is what an internal service over HTTPS needs. Setting one half of the
//! identity without the other is a mistake, and says so rather than sending an
//! anonymous request that the far end will refuse.

use std::sync::Arc;
use std::sync::OnceLock;

use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::{ClientConfig, RootCertStore};

/// The configuration is built once. Parsing a certificate chain per request
/// would be wasted work on every call, and a key read repeatedly is a key
/// exposed repeatedly.
static CONFIGURED: OnceLock<Option<Arc<ClientConfig>>> = OnceLock::new();

/// The client identity to present, if one is configured.
///
/// `None` means no client certificate was asked for, and the caller should use
/// the ordinary HTTPS path rather than a configuration of ours.
pub fn client_config() -> Option<Arc<ClientConfig>> {
    CONFIGURED.get_or_init(build).clone()
}

/// Is mutual TLS configured at all? Cheap enough to ask on every request.
pub fn is_configured() -> bool {
    client_config().is_some()
}

fn build() -> Option<Arc<ClientConfig>> {
    let cert_path = std::env::var("ETAMIL_TLS_CERT").ok();
    let key_path = std::env::var("ETAMIL_TLS_KEY").ok();
    let ca_path = std::env::var("ETAMIL_TLS_CA").ok();

    // Nothing asked for: ordinary HTTPS, exactly as before.
    if cert_path.is_none() && key_path.is_none() && ca_path.is_none() {
        return None;
    }

    // Half an identity is a mistake worth naming. Reading one without the
    // other and carrying on would send an anonymous request that the far end
    // refuses, which looks like the far end's fault.
    let identity = match (&cert_path, &key_path) {
        (Some(cert), Some(key)) => Some((cert.clone(), key.clone())),
        (None, None) => None,
        (Some(_), None) => {
            eprintln!("✗ ETAMIL_TLS_CERT is set but ETAMIL_TLS_KEY is not");
            eprintln!("  a certificate cannot be presented without its key");
            return None;
        }
        (None, Some(_)) => {
            eprintln!("✗ ETAMIL_TLS_KEY is set but ETAMIL_TLS_CERT is not");
            eprintln!("  a key proves nothing without the certificate it belongs to");
            return None;
        }
    };

    match assemble(identity, ca_path.as_deref()) {
        Ok(config) => Some(Arc::new(config)),
        Err(why) => {
            eprintln!("✗ the TLS settings could not be used: {}", why);
            eprintln!("  outbound HTTPS will fall back to the public roots and no identity");
            None
        }
    }
}

/// Build the client configuration.
///
/// The two halves are independent. A private CA is worth trusting whether or
/// not we present a certificate of our own — an internal service over HTTPS
/// needs the first and not the second — so either may be configured alone.
fn assemble(
    identity: Option<(String, String)>,
    ca_path: Option<&str>,
) -> Result<ClientConfig, String> {
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    // A bank that runs its own CA is the normal case, not the exception, and
    // its root will not be among the public ones.
    if let Some(ca_path) = ca_path {
        let extra = read_certificates(ca_path)?;
        if extra.is_empty() {
            return Err(format!("ETAMIL_TLS_CA '{}' holds no certificate", ca_path));
        }
        for certificate in extra {
            roots
                .add(certificate)
                .map_err(|e| format!("ETAMIL_TLS_CA '{}' was not usable: {}", ca_path, e))?;
        }
    }

    // The provider is named rather than taken from the process default: with
    // no default installed, building a config panics, and a panic inside a
    // request handler is a worse way to learn about it than an error here.
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let builder = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| format!("TLS versions could not be selected: {}", e))?
        .with_root_certificates(roots);

    match identity {
        Some((cert_path, key_path)) => {
            let chain = read_certificates(&cert_path)?;
            if chain.is_empty() {
                return Err(format!("'{}' holds no certificate", cert_path));
            }
            let key = read_private_key(&key_path)?;
            builder.with_client_auth_cert(chain, key).map_err(|e| {
                format!("the certificate and key were not accepted together: {}", e)
            })
        }
        None => Ok(builder.with_no_client_auth()),
    }
}

fn read_certificates(path: &str) -> Result<Vec<CertificateDer<'static>>, String> {
    let pem = std::fs::read(path).map_err(|e| format!("'{}' could not be read: {}", path, e))?;
    rustls_pemfile::certs(&mut pem.as_slice())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("'{}' is not a PEM certificate: {}", path, e))
}

/// A private key in whichever of the three PEM shapes it was written in.
///
/// Which one you get depends on what produced it, and an operator handed a key
/// by their bank should not have to know or convert it.
fn read_private_key(path: &str) -> Result<PrivateKeyDer<'static>, String> {
    let pem = std::fs::read(path).map_err(|e| format!("'{}' could not be read: {}", path, e))?;
    rustls_pemfile::private_key(&mut pem.as_slice())
        .map_err(|e| format!("'{}' is not a PEM private key: {}", path, e))?
        .ok_or_else(|| {
            format!(
                "'{}' holds no private key  (expected PKCS#8, SEC1 or PKCS#1 PEM)",
                path
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_file_is_reported_rather_than_ignored() {
        let outcome = assemble(
            Some(("no-such-cert.pem".to_string(), "no-such-key.pem".to_string())),
            None,
        );

        let why = outcome.err().expect("a missing certificate cannot succeed");
        assert!(why.contains("could not be read"), "unexpected: {}", why);
    }

    #[test]
    fn a_private_ca_can_be_trusted_without_presenting_an_identity() {
        // The two halves are independent: an internal service over HTTPS needs
        // its CA trusted and asks for no certificate in return.
        let path = std::env::temp_dir().join("etamil_ca_only.pem");
        std::fs::write(&path, b"not a certificate\n").unwrap();
        let outcome = assemble(None, Some(&path.to_string_lossy()));
        let _ = std::fs::remove_file(&path);

        // It fails because the file is rubbish, not because no identity was
        // given — which is the distinction being asserted.
        let why = outcome.err().expect("rubbish is not a CA");
        assert!(why.contains("ETAMIL_TLS_CA"), "unexpected: {}", why);
    }

    #[test]
    fn a_file_that_is_not_a_certificate_is_refused() {
        // Not "treated as empty and carried on with": a request that goes out
        // without the identity the operator configured looks like the far
        // end's problem when it is refused.
        let path = std::env::temp_dir().join("etamil_not_a_cert.pem");
        std::fs::write(&path, b"this is not a certificate\n").unwrap();

        let outcome = assemble(
            Some((path.to_string_lossy().into(), path.to_string_lossy().into())),
            None,
        );
        let _ = std::fs::remove_file(&path);

        assert!(outcome.is_err(), "a text file is not a certificate chain");
    }
}
