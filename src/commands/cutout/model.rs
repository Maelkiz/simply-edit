//! Acquisition and caching of the U²-Net weights used by neural cutout.
//!
//! The weights are ~168 MB, so they are never bundled in the binary and never
//! fetched without the user saying yes.

use std::fs::{self, File};
use std::io::{IsTerminal, Read, Write, stdin};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

/// U²-Net weights from the rembg `v0.0.0` release. The weights are Apache-2.0
/// (rembg itself is MIT); RMBG-1.4/2.0 are deliberately avoided because their
/// BRIA license requires a paid commercial agreement.
const MODEL_URL: &str = "https://github.com/danielgatis/rembg/releases/download/v0.0.0/u2net.onnx";
const MODEL_SHA256: &str = "8d10d2f3bb75ae3b6d527c77944fc5e7dcd94b29809d47a739a7a728a912b491";
const MODEL_BYTES: u64 = 175_997_641;
const MODEL_FILE: &str = "u2net.onnx";

/// Environment variable overriding the cache location. Earns its place twice:
/// air-gapped installs, and letting the test suite exercise the model paths
/// without a 168 MB download.
const MODEL_DIR_ENV: &str = "SIMPLY_MODEL_DIR";

/// Refuse to stream more than this. The server is remote and untrusted; without
/// a cap a bad response could fill the user's disk.
const MAX_DOWNLOAD_BYTES: u64 = MODEL_BYTES * 2;

/// Human-readable download size, for prompts and error messages.
fn model_size_mb() -> u64 {
    MODEL_BYTES / 1_000_000
}

pub(crate) fn model_path() -> Result<PathBuf, String> {
    let dir = match std::env::var_os(MODEL_DIR_ENV) {
        Some(dir) => PathBuf::from(dir),
        None => dirs::cache_dir()
            .ok_or_else(|| {
                format!(
                    "cutout: could not determine a cache directory; set {MODEL_DIR_ENV} to choose where the model is stored"
                )
            })?
            .join("simply-edit")
            .join("models"),
    };
    Ok(dir.join(MODEL_FILE))
}

/// Returns the path to the cached model, downloading it if necessary.
///
/// `consented` is true when the user already asked for the download explicitly
/// (`--download-model`). Otherwise consent is obtained interactively, and a
/// non-interactive run is an error rather than a silent 168 MB fetch: a large
/// network transfer should not be triggerable by piped stdin.
pub(crate) fn ensure_model(consented: bool) -> Result<PathBuf, String> {
    let path = model_path()?;
    if path.is_file() {
        return Ok(path);
    }

    if !consented {
        if !stdin().is_terminal() {
            return Err(format!(
                "cutout: the background removal model is not downloaded. Run 'simply cutout --download-model' to fetch it ({} MB), or use --fast for flat backgrounds",
                model_size_mb()
            ));
        }
        let ok = cliclack::confirm(format!(
            "Download the background removal model ({} MB) to {}?",
            model_size_mb(),
            path.display()
        ))
        .interact()
        .map_err(|e| format!("cutout: failed to read confirmation: {e}"))?;
        if !ok {
            return Err(
                "cutout: model download declined; use --fast for flat backgrounds".to_string(),
            );
        }
    }

    download_model(&path)?;
    Ok(path)
}

/// Streams the model to `{dest}.part`, verifies its checksum, then renames it
/// into place — the same temp-then-rename discipline `io::save_transformed_image`
/// uses, so an interrupted download never leaves a corrupt cache entry.
pub(crate) fn download_model(dest: &Path) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "cutout: failed to create model directory '{}': {e}",
                parent.display()
            )
        })?;
    }

    let part = dest.with_extension("part");
    match stream_to_file(&part) {
        Ok(digest) => install_verified(&part, dest, &digest),
        Err(e) => {
            let _ = fs::remove_file(&part);
            Err(e)
        }
    }
}

/// Moves `part` to `dest` only if `digest` matches the pinned checksum.
///
/// A mismatch means the bytes are not the model we pinned — a truncated
/// transfer, a re-tagged release, or tampering — so the file is destroyed
/// rather than cached.
fn install_verified(part: &Path, dest: &Path, digest: &str) -> Result<(), String> {
    if digest != MODEL_SHA256 {
        let _ = fs::remove_file(part);
        return Err(format!(
            "cutout: downloaded model failed checksum verification (expected {MODEL_SHA256}, got {digest}); the file was discarded"
        ));
    }

    fs::rename(part, dest).map_err(|e| {
        let _ = fs::remove_file(part);
        format!(
            "cutout: failed to move the model into '{}': {e}",
            dest.display()
        )
    })
}

/// Downloads [`MODEL_URL`] into `part`, returning the hex sha256 of the bytes
/// written.
fn stream_to_file(part: &Path) -> Result<String, String> {
    let response = ureq::get(MODEL_URL)
        .call()
        .map_err(|e| format!("cutout: failed to download the model from {MODEL_URL}: {e}"))?;

    let body = response.into_body();
    let total = body.content_length().unwrap_or(MODEL_BYTES);
    let mut reader = body.into_reader();

    let mut file = File::create(part)
        .map_err(|e| format!("cutout: failed to create '{}': {e}", part.display()))?;

    let progress = byte_progress_bar(total);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 64 * 1024];
    let mut written: u64 = 0;

    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|e| format!("cutout: failed while downloading the model: {e}"))?;
        if read == 0 {
            break;
        }
        written += read as u64;
        if written > MAX_DOWNLOAD_BYTES {
            return Err(format!(
                "cutout: the download exceeded {MAX_DOWNLOAD_BYTES} bytes and was aborted"
            ));
        }
        hasher.update(&buffer[..read]);
        file.write_all(&buffer[..read])
            .map_err(|e| format!("cutout: failed to write '{}': {e}", part.display()))?;
        progress.set_position(written.min(total));
    }

    file.flush()
        .map_err(|e| format!("cutout: failed to write '{}': {e}", part.display()))?;
    progress.finish_and_clear();

    Ok(hex(&hasher.finalize()))
}

/// Byte-denominated sibling of [`crate::batch::create_progress_bar`]; hidden
/// when stderr is not a terminal so piped output stays clean.
fn byte_progress_bar(total: u64) -> indicatif::ProgressBar {
    if !std::io::stderr().is_terminal() {
        return indicatif::ProgressBar::hidden();
    }
    let pb = indicatif::ProgressBar::new(total);
    let style = indicatif::ProgressStyle::with_template("[{bar:30}] {bytes}/{total_bytes}")
        .unwrap_or_else(|_| indicatif::ProgressStyle::default_bar());
    pb.set_style(style);
    pb
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Fetches the model on the user's explicit request and reports where it went.
pub(crate) fn run_download_model() -> Result<(), String> {
    let path = model_path()?;
    if path.is_file() {
        println!("Model already present at {}", path.display());
        return Ok(());
    }
    let path = ensure_model(true)?;
    println!("Downloaded model to {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::temp_dir;

    /// `SIMPLY_MODEL_DIR` is process-global, so every test that touches it must
    /// hold this lock.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn with_model_dir<T>(dir: &Path, f: impl FnOnce() -> T) -> T {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: the lock serialises every mutation of this variable in-process.
        unsafe { std::env::set_var(MODEL_DIR_ENV, dir) };
        let result = f();
        unsafe { std::env::remove_var(MODEL_DIR_ENV) };
        result
    }

    #[test]
    fn test_model_path_honors_the_env_override() {
        let dir = temp_dir("simply-model-env");
        let path = with_model_dir(&dir, || model_path().expect("path resolves"));
        assert_eq!(path, dir.join(MODEL_FILE));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_ensure_model_returns_a_cached_file_without_downloading() {
        let dir = temp_dir("simply-model-cached");
        fs::write(dir.join(MODEL_FILE), b"stand-in weights").expect("write stub");

        let path = with_model_dir(&dir, || ensure_model(false).expect("cache hit"));
        assert_eq!(path, dir.join(MODEL_FILE));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_model_constants_are_self_consistent() {
        assert_eq!(MODEL_SHA256.len(), 64);
        assert!(MODEL_SHA256.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(MODEL_URL.starts_with("https://"));
        assert_eq!(model_size_mb(), 175);
    }

    #[test]
    fn test_hex_encodes_lowercase_fixed_width() {
        assert_eq!(hex(&[0x00, 0x0f, 0xff]), "000fff");
    }

    #[test]
    fn test_sha256_of_known_input_matches() {
        let mut hasher = Sha256::new();
        hasher.update(b"abc");
        assert_eq!(
            hex(&hasher.finalize()),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_ensure_model_errors_without_a_tty_instead_of_downloading() {
        let dir = temp_dir("simply-model-nontty");
        // cargo test gives the child no terminal, so this exercises the
        // non-interactive branch without any network access.
        let err = with_model_dir(&dir, || {
            ensure_model(false).expect_err("a cache miss must not download silently")
        });
        assert!(err.contains("--download-model"), "{err}");
        assert!(err.contains("--fast"), "{err}");
        assert!(!dir.join(MODEL_FILE).exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_install_verified_rejects_a_bad_checksum_and_deletes_the_file() {
        let dir = temp_dir("simply-model-badsum");
        let part = dir.join("u2net.part");
        let dest = dir.join(MODEL_FILE);
        fs::write(&part, b"truncated").expect("write part");

        let err = install_verified(&part, &dest, &"0".repeat(64))
            .expect_err("a checksum mismatch must be rejected");
        assert!(err.contains("checksum verification"), "{err}");
        assert!(!part.exists(), "the partial file must be discarded");
        assert!(!dest.exists(), "nothing may be installed");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_install_verified_installs_on_a_matching_checksum() {
        let dir = temp_dir("simply-model-goodsum");
        let part = dir.join("u2net.part");
        let dest = dir.join(MODEL_FILE);
        fs::write(&part, b"weights").expect("write part");

        install_verified(&part, &dest, MODEL_SHA256).expect("matching checksum installs");
        assert!(!part.exists());
        assert!(dest.exists());
        let _ = fs::remove_dir_all(&dir);
    }
}
