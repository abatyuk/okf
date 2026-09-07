//! Network effect for `url` source fingerprints.
//!
//! The trait is always compiled (the `url` fingerprint logic only needs the port), but the
//! production [`RealNet`] impl — which pulls in `ureq` — is gated behind `feature = "url-sources"`.
use crate::error::Result;

/// The bits of an HTTP response a URL fingerprint cares about: validators first, body last.
#[derive(Debug, Clone, Default)]
pub struct HttpResponse {
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub body: Vec<u8>,
}

pub trait Net {
    /// Fetch the headers/body needed to fingerprint a URL (etag/last-modified/body).
    fn fetch(&self, url: &str) -> Result<HttpResponse>;
}

/// Production network port using `ureq` (feature = "url-sources").
#[cfg(feature = "url-sources")]
pub struct RealNet;

#[cfg(feature = "url-sources")]
impl Net for RealNet {
    fn fetch(&self, url: &str) -> Result<HttpResponse> {
        use crate::error::OkfError;
        let resp = ureq::get(url)
            .call()
            .map_err(|e| OkfError::Io(format!("fetch {url}: {e}")))?;
        let etag = resp.header("etag").map(str::to_string);
        let last_modified = resp.header("last-modified").map(str::to_string);
        let mut body = Vec::new();
        std::io::Read::read_to_end(&mut resp.into_reader(), &mut body)
            .map_err(|e| OkfError::Io(format!("read body {url}: {e}")))?;
        Ok(HttpResponse {
            etag,
            last_modified,
            body,
        })
    }
}

/// In-memory network for hermetic tests.
#[derive(Default, Clone)]
pub struct FakeNet {
    responses: std::collections::HashMap<String, HttpResponse>,
}

impl FakeNet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_response(mut self, url: impl Into<String>, resp: HttpResponse) -> Self {
        self.responses.insert(url.into(), resp);
        self
    }
}

impl Net for FakeNet {
    fn fetch(&self, url: &str) -> Result<HttpResponse> {
        self.responses
            .get(url)
            .cloned()
            .ok_or_else(|| crate::error::OkfError::Io(format!("no fake response for {url}")))
    }
}
