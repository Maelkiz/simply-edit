use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;

use crate::cli::BatchArgs;

const RASTER_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "ico"];
const SVG_EXTENSIONS: &[&str] = &["svg"];

pub(crate) struct BatchOptions {
    pub pattern: Option<Regex>,
    pub output_dir: Option<PathBuf>,
    pub recursive: bool,
}

pub(crate) struct BatchResult {
    pub succeeded: usize,
    pub failed: Vec<(PathBuf, String)>,
}

pub(crate) fn to_batch_options(args: &BatchArgs) -> Result<BatchOptions, String> {
    let pattern = match &args.pattern {
        Some(pat) => {
            let re = Regex::new(pat).map_err(|e| format!("invalid --pattern regex '{pat}': {e}"))?;
            Some(re)
        }
        None => None,
    };

    if let Some(dir) = &args.output_dir {
        if !dir.exists() {
            fs::create_dir_all(dir).map_err(|e| {
                format!(
                    "failed to create output directory '{}': {e}",
                    dir.display()
                )
            })?;
        }
    }

    Ok(BatchOptions {
        pattern,
        output_dir: args.output_dir.clone(),
        recursive: args.recursive,
    })
}

pub(crate) fn collect_files(
    dir: &Path,
    recursive: bool,
    pattern: Option<&Regex>,
    extensions: &[&str],
) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    walk_dir(dir, recursive, pattern, extensions, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk_dir(
    dir: &Path,
    recursive: bool,
    pattern: Option<&Regex>,
    extensions: &[&str],
    out: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("failed to read directory '{}': {e}", dir.display()))?;

    for entry in entries {
        let entry =
            entry.map_err(|e| format!("failed to read entry in '{}': {e}", dir.display()))?;
        let path = entry.path();

        if path.is_dir() {
            if recursive {
                walk_dir(&path, recursive, pattern, extensions, out)?;
            }
            continue;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        let ext = match ext {
            Some(e) => e,
            None => continue,
        };

        if !extensions.iter().any(|&supported| supported == ext) {
            continue;
        }

        if let Some(re) = pattern {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if !re.is_match(filename) {
                continue;
            }
        }

        out.push(path);
    }

    Ok(())
}

pub(crate) fn run_batch<F>(
    dir: &Path,
    options: &BatchOptions,
    process_file: F,
) -> Result<BatchResult, String>
where
    F: Fn(&Path) -> Result<String, String>,
{
    run_batch_with_extensions(dir, options, RASTER_EXTENSIONS, process_file)
}

pub(crate) fn run_batch_svg<F>(
    dir: &Path,
    options: &BatchOptions,
    process_file: F,
) -> Result<BatchResult, String>
where
    F: Fn(&Path) -> Result<String, String>,
{
    run_batch_with_extensions(dir, options, SVG_EXTENSIONS, process_file)
}

fn run_batch_with_extensions<F>(
    dir: &Path,
    options: &BatchOptions,
    extensions: &[&str],
    process_file: F,
) -> Result<BatchResult, String>
where
    F: Fn(&Path) -> Result<String, String>,
{
    if !dir.is_dir() {
        return Err(format!("'{}' is not a directory", dir.display()));
    }

    let files = collect_files(dir, options.recursive, options.pattern.as_ref(), extensions)?;

    if files.is_empty() {
        return Ok(BatchResult {
            succeeded: 0,
            failed: vec![],
        });
    }

    let pb = create_progress_bar(files.len() as u64);

    let mut succeeded = 0usize;
    let mut failed: Vec<(PathBuf, String)> = Vec::new();

    for file in &files {
        match process_file(file) {
            Ok(_) => succeeded += 1,
            Err(e) => failed.push((file.clone(), e)),
        }
        pb.inc(1);
    }

    pb.finish_and_clear();

    Ok(BatchResult { succeeded, failed })
}

fn create_progress_bar(total: u64) -> ProgressBar {
    if !std::io::stderr().is_terminal() {
        return ProgressBar::hidden();
    }

    let pb = ProgressBar::new(total);
    let style = ProgressStyle::with_template("[{bar:30}] {pos}/{len}")
        .unwrap_or_else(|_| ProgressStyle::default_bar());
    pb.set_style(style);
    pb
}

pub(crate) fn resolve_output_path(
    input: &Path,
    suffix: &str,
    options: &BatchOptions,
) -> Result<PathBuf, String> {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    let filename = format!("{stem}_{suffix}.{ext}");

    let dir = match &options.output_dir {
        Some(d) => d.clone(),
        None => input.parent().unwrap_or_else(|| Path::new(".")).to_path_buf(),
    };

    Ok(dir.join(filename))
}

pub(crate) fn resolve_output_path_with_ext(
    input: &Path,
    ext: &str,
    options: &BatchOptions,
) -> Result<PathBuf, String> {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let filename = format!("{stem}.{ext}");

    let dir = match &options.output_dir {
        Some(d) => d.clone(),
        None => input.parent().unwrap_or_else(|| Path::new(".")).to_path_buf(),
    };

    Ok(dir.join(filename))
}

pub(crate) fn print_summary(result: &BatchResult) {
    let total = result.succeeded + result.failed.len();

    if total == 0 {
        println!("No matching files found");
        return;
    }

    println!(
        "Batch complete: {} succeeded, {} failed",
        result.succeeded,
        result.failed.len()
    );

    for (path, err) in &result.failed {
        eprintln!("  ✗ {}: {err}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "simply-batch-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before unix epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&dir).expect("failed to create temp dir");
        dir
    }

    fn touch(dir: &Path, name: &str) {
        fs::write(dir.join(name), b"").expect("failed to write file");
    }

    #[test]
    fn test_collect_files_finds_supported_images() {
        let dir = temp_dir("collect-basic");
        touch(&dir, "a.png");
        touch(&dir, "b.jpg");
        touch(&dir, "c.txt");

        let files = collect_files(&dir, false, None, RASTER_EXTENSIONS).unwrap();
        assert_eq!(files.len(), 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_collect_files_empty_dir() {
        let dir = temp_dir("collect-empty");
        let files = collect_files(&dir, false, None, RASTER_EXTENSIONS).unwrap();
        assert!(files.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_collect_files_regex_filter() {
        let dir = temp_dir("collect-regex");
        touch(&dir, "photo_01.jpg");
        touch(&dir, "photo_02.jpg");
        touch(&dir, "screenshot.png");

        let re = Regex::new(r"^photo_").unwrap();
        let files = collect_files(&dir, false, Some(&re), RASTER_EXTENSIONS).unwrap();
        assert_eq!(files.len(), 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_collect_files_recursive() {
        let dir = temp_dir("collect-recursive");
        let sub = dir.join("sub");
        fs::create_dir_all(&sub).unwrap();
        touch(&dir, "top.png");
        touch(&sub, "nested.png");

        let files_flat = collect_files(&dir, false, None, RASTER_EXTENSIONS).unwrap();
        assert_eq!(files_flat.len(), 1);

        let files_recursive = collect_files(&dir, true, None, RASTER_EXTENSIONS).unwrap();
        assert_eq!(files_recursive.len(), 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_collect_files_svg_extensions() {
        let dir = temp_dir("collect-svg");
        touch(&dir, "icon.svg");
        touch(&dir, "photo.png");

        let files = collect_files(&dir, false, None, SVG_EXTENSIONS).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].extension().unwrap() == "svg");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_collect_files_sorted() {
        let dir = temp_dir("collect-sorted");
        touch(&dir, "c.png");
        touch(&dir, "a.png");
        touch(&dir, "b.png");

        let files = collect_files(&dir, false, None, RASTER_EXTENSIONS).unwrap();
        let names: Vec<_> = files
            .iter()
            .map(|f| f.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["a.png", "b.png", "c.png"]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_output_path_same_dir() {
        let options = BatchOptions {
            pattern: None,
            output_dir: None,
            recursive: false,
        };
        let result = resolve_output_path(Path::new("/photos/img.png"), "invert", &options).unwrap();
        assert_eq!(result, PathBuf::from("/photos/img_invert.png"));
    }

    #[test]
    fn test_resolve_output_path_with_output_dir() {
        let options = BatchOptions {
            pattern: None,
            output_dir: Some(PathBuf::from("/out")),
            recursive: false,
        };
        let result = resolve_output_path(Path::new("/photos/img.png"), "invert", &options).unwrap();
        assert_eq!(result, PathBuf::from("/out/img_invert.png"));
    }

    #[test]
    fn test_resolve_output_path_with_ext() {
        let options = BatchOptions {
            pattern: None,
            output_dir: Some(PathBuf::from("/out")),
            recursive: false,
        };
        let result =
            resolve_output_path_with_ext(Path::new("/photos/img.png"), "webp", &options).unwrap();
        assert_eq!(result, PathBuf::from("/out/img.webp"));
    }

    #[test]
    fn test_run_batch_empty_dir() {
        let dir = temp_dir("batch-empty");
        let options = BatchOptions {
            pattern: None,
            output_dir: None,
            recursive: false,
        };
        let result = run_batch(&dir, &options, |_| Ok("done".to_string())).unwrap();
        assert_eq!(result.succeeded, 0);
        assert!(result.failed.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_run_batch_continues_on_error() {
        let dir = temp_dir("batch-errors");
        touch(&dir, "good.png");
        touch(&dir, "bad.png");

        let options = BatchOptions {
            pattern: None,
            output_dir: None,
            recursive: false,
        };
        let result = run_batch(&dir, &options, |file| {
            if file.file_name().unwrap() == "bad.png" {
                Err("simulated failure".to_string())
            } else {
                Ok("ok".to_string())
            }
        })
        .unwrap();

        assert_eq!(result.succeeded, 1);
        assert_eq!(result.failed.len(), 1);
        assert!(result.failed[0].1.contains("simulated failure"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_to_batch_options_invalid_regex() {
        let args = BatchArgs {
            pattern: Some("[invalid".to_string()),
            output_dir: None,
            recursive: false,
        };
        assert!(to_batch_options(&args).is_err());
    }
}
