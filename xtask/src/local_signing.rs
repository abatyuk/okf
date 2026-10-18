//! Ad-hoc signing for local macOS builds; release signing belongs to CI.
use serde_json::Value;
use std::collections::BTreeSet;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn enabled() -> bool {
    should_sign(std::env::consts::OS, std::env::var_os("CI").is_some())
}

fn should_sign(os: &str, ci_present: bool) -> bool {
    os == "macos" && !ci_present
}

/// Cargo reports actual executable paths, including custom profiles and target directories.
pub fn build(root: &Path, args: &[&str]) {
    let mut child = Command::new(env!("CARGO"))
        .args(args)
        .arg("--message-format=json-render-diagnostics")
        .current_dir(root)
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn cargo build");
    let mut binaries = BTreeSet::new();
    for line in BufReader::new(child.stdout.take().unwrap()).lines() {
        let line = line.expect("read cargo build output");
        match serde_json::from_str::<Value>(&line) {
            Ok(record) => {
                if let Some(path) = executable(&record) {
                    binaries.insert(path);
                }
            }
            Err(_) => println!("{line}"),
        }
    }
    let status = child.wait().expect("wait for cargo build");
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    for binary in binaries {
        sign(&binary);
    }
}

// The running build helper is tooling, not an application artifact to re-sign.
fn executable(record: &Value) -> Option<PathBuf> {
    if record["reason"] != "compiler-artifact"
        || record["profile"]["test"] == true
        || record["target"]["name"] == "xtask"
    {
        return None;
    }
    record["executable"].as_str().map(PathBuf::from)
}

/// Match Cargo install destination precedence, including legacy Cargo config filenames.
pub fn install_root(root: &Path) -> PathBuf {
    if let Some(path) = std::env::var_os("CARGO_INSTALL_ROOT") {
        return root.join(path);
    }
    let cargo_home = std::env::var_os("CARGO_HOME")
        .map(|path| root.join(path))
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").expect("HOME or CARGO_HOME required"))
                .join(".cargo")
        });
    // With --path, Cargo discovers configuration from the package directory upward.
    let package = root.join("crates/okf-cli");
    let mut dirs: Vec<_> = package.ancestors().map(|dir| dir.join(".cargo")).collect();
    if !dirs.contains(&cargo_home) {
        dirs.push(cargo_home.clone());
    }
    for dir in dirs {
        let legacy = dir.join("config");
        let config = if legacy.is_file() {
            legacy
        } else {
            dir.join("config.toml")
        };
        if let Ok(text) = std::fs::read_to_string(&config) {
            let value: toml::Value = toml::from_str(&text).expect("valid Cargo config");
            if let Some(path) = value
                .get("install")
                .and_then(|v| v.get("root"))
                .and_then(toml::Value::as_str)
            {
                return config.parent().unwrap().parent().unwrap().join(path);
            }
        }
    }
    cargo_home
}

pub fn sign(binary: &Path) {
    // Cross-compilation on a Mac may produce ELF/PE executables. Only sign Mach-O.
    let mut file = std::fs::File::open(binary).expect("open built executable");
    let mut magic = [0; 4];
    use std::io::Read;
    file.read_exact(&mut magic).expect("read executable format");
    if !is_macho(magic) {
        return;
    }
    eprintln!("Ad-hoc signing {}", binary.display());
    for args in [
        &["--force", "--sign", "-"][..],
        &["--verify", "--strict"][..],
    ] {
        let status = Command::new("/usr/bin/codesign")
            .args(args)
            .arg(binary)
            .status()
            .expect("run codesign");
        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
    }
}

fn is_macho(magic: [u8; 4]) -> bool {
    matches!(
        u32::from_be_bytes(magic),
        0xfeedface
            | 0xcefaedfe
            | 0xfeedfacf
            | 0xcffaedfe
            | 0xcafebabe
            | 0xbebafeca
            | 0xcafebabf
            | 0xbfbafeca
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signing_is_exclusively_local_macos() {
        assert!(should_sign("macos", false));
        for os in ["macos", "linux", "windows"] {
            assert!(!should_sign(os, true));
        }
        assert!(!should_sign("linux", false));
        assert!(!should_sign("windows", false));
    }

    #[test]
    fn artifact_paths_do_not_assume_profile_or_target_directory() {
        let record = serde_json::json!({"reason": "compiler-artifact",
            "profile": {"test": false}, "executable": "/tmp/custom/aarch64-apple-darwin/fast/okf"});
        assert_eq!(
            executable(&record),
            Some(PathBuf::from("/tmp/custom/aarch64-apple-darwin/fast/okf"))
        );
        assert!(executable(&serde_json::json!({"reason":"build-finished"})).is_none());
        assert!(executable(&serde_json::json!({"reason":"compiler-artifact", "target":{"name":"xtask"}, "executable":"xtask"})).is_none());
        assert!(executable(&serde_json::json!({"reason":"compiler-artifact", "profile":{"test":true},"executable":"test"})).is_none());
    }

    #[test]
    fn cross_compiled_non_macos_binaries_are_skipped() {
        assert!(is_macho([0xcf, 0xfa, 0xed, 0xfe]));
        assert!(is_macho([0xca, 0xfe, 0xba, 0xbe]));
        assert!(!is_macho(*b"\x7fELF"));
        assert!(!is_macho(*b"MZ\0\0"));
    }
}
