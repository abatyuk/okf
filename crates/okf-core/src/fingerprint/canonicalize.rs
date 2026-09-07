//! Text normalization (LF, trailing-ws, blank-line rules) and hashing.
//!
//! This is the cross-platform-stability guarantee: the same logical region must hash
//! identically on macOS, Linux and CI regardless of checkout EOL settings. Per
//! ARCHITECTURE.md "Fingerprint canonicalization", the steps are:
//!   1. decode UTF-8 (caller's responsibility);
//!   2. normalize `CRLF`/`CR` → `LF`;
//!   3. strip trailing whitespace per line;
//!   4. strip leading/trailing blank lines from the region;
//!   5. join with `\n`, no trailing newline.
use sha2::{Digest, Sha256};

/// Apply the canonical text normalization (steps 2–5 above) to an already-decoded string.
pub fn canonicalize_text(input: &str) -> String {
    // 2. CRLF / CR → LF.
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");

    // 3. strip trailing whitespace per line.
    let mut lines: Vec<&str> = normalized.split('\n').map(str::trim_end).collect();

    // 4. strip leading/trailing blank lines.
    while lines.first().is_some_and(|l| l.is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }

    // 5. join with `\n`, no trailing newline.
    lines.join("\n")
}

/// Lowercase-hex SHA-256 of arbitrary bytes.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

/// Canonicalize `input` then hash it — the shared path for `line-range` / `markdown-heading`.
pub fn canonical_sha256(input: &str) -> String {
    sha256_hex(canonicalize_text(input).as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crlf_cr_and_lf_are_equivalent() {
        let lf = "line one\nline two\n";
        let crlf = "line one\r\nline two\r\n";
        let cr = "line one\rline two\r";
        assert_eq!(canonicalize_text(lf), canonicalize_text(crlf));
        assert_eq!(canonicalize_text(lf), canonicalize_text(cr));
        assert_eq!(canonical_sha256(lf), canonical_sha256(crlf));
    }

    #[test]
    fn trailing_whitespace_is_stripped_per_line() {
        assert_eq!(
            canonicalize_text("a  \nb\t\n"),
            canonicalize_text("a\nb\n")
        );
    }

    #[test]
    fn leading_and_trailing_blank_lines_are_stripped() {
        assert_eq!(
            canonicalize_text("\n\n  \nkeep\nme\n \n\n"),
            "keep\nme"
        );
    }

    #[test]
    fn interior_blank_lines_are_preserved() {
        assert_eq!(canonicalize_text("a\n\nb"), "a\n\nb");
    }

    #[test]
    fn no_trailing_newline_in_output() {
        assert!(!canonicalize_text("a\nb\n").ends_with('\n'));
    }

    #[test]
    fn sha256_is_lowercase_hex_of_known_vector() {
        // SHA-256("abc")
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
