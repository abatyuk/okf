//! url fingerprints (etag / last-modified / body) via `ports::Net`.
//!
//! The logic only needs the [`Net`] port, so it always compiles; the production `RealNet`
//! that pulls in `ureq` is what lives behind `feature = "url-sources"`.
use super::canonicalize::sha256_hex;
use crate::error::{OkfError, Result};
use crate::model::source::Fingerprint;
use crate::ports::net::Net;

/// Fingerprint a URL: prefer `etag`, else `last_modified`, else `sha256` of the body.
pub fn url_fp(net: Option<&dyn Net>, url: &str) -> Result<Fingerprint> {
    let net = net.ok_or_else(|| {
        OkfError::Environment(
            "url source requires the network port (build with feature `url-sources`)".to_string(),
        )
    })?;
    let resp = net.fetch(url)?;
    if let Some(etag) = resp.etag {
        return Ok(Fingerprint::from_pairs(vec![("etag".to_string(), etag)]));
    }
    if let Some(lm) = resp.last_modified {
        return Ok(Fingerprint::from_pairs(vec![(
            "last_modified".to_string(),
            lm,
        )]));
    }
    Ok(Fingerprint::from_pairs(vec![(
        "sha256".to_string(),
        sha256_hex(&resp.body),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::net::{FakeNet, HttpResponse};

    #[test]
    fn prefers_etag() {
        let net = FakeNet::new().with_response(
            "https://x/y",
            HttpResponse {
                etag: Some("W/\"abc\"".to_string()),
                last_modified: Some("2026-08-01T00:00:00Z".to_string()),
                body: b"hello".to_vec(),
            },
        );
        let fp = url_fp(Some(&net), "https://x/y").unwrap();
        assert_eq!(fp.get("etag"), Some("W/\"abc\""));
    }

    #[test]
    fn falls_back_to_last_modified_then_body() {
        let net = FakeNet::new()
            .with_response(
                "https://a",
                HttpResponse {
                    last_modified: Some("2026-08-01T00:00:00Z".to_string()),
                    ..Default::default()
                },
            )
            .with_response(
                "https://b",
                HttpResponse {
                    body: b"abc".to_vec(),
                    ..Default::default()
                },
            );
        assert_eq!(
            url_fp(Some(&net), "https://a")
                .unwrap()
                .get("last_modified"),
            Some("2026-08-01T00:00:00Z")
        );
        assert_eq!(
            url_fp(Some(&net), "https://b").unwrap().get("sha256"),
            Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        );
    }

    #[test]
    fn missing_net_is_environment_error() {
        assert!(url_fp(None, "https://x").is_err());
    }
}
