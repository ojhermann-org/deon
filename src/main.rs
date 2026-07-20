//! `deon-check` — run the leak-detection static check over `.okf.md` norm files.
//!
//! Usage: `deon-check [--quiet] <path>...`
//!   Each path is a `.okf.md` file or a directory searched recursively for them.
//! Exit: 0 = clean, 1 = leaks found, 2 = usage/IO/parse error.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use deon_check::check;

const USAGE: &str = "\
deon-check — leak-detection static check for deon norms (DESIGN §4, check 1)

Usage:
    deon-check [--quiet] <path>...

Arguments:
    <path>       a .okf.md file, or a directory searched recursively for them

Options:
    --quiet      print findings only (suppress the per-file/summary lines)
    -h, --help   show this help

Exit codes:
    0  clean (no leaks)      1  leaks found      2  usage / IO / parse error";

fn main() -> ExitCode {
    let mut quiet = false;
    let mut paths: Vec<String> = Vec::new();
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            "--quiet" => quiet = true,
            _ => paths.push(arg),
        }
    }
    if paths.is_empty() {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }

    let mut files = Vec::new();
    for p in &paths {
        if let Err(e) = collect(Path::new(p), &mut files) {
            eprintln!("error: {p}: {e}");
            return ExitCode::from(2);
        }
    }
    files.sort();
    files.dedup();
    if files.is_empty() {
        eprintln!("error: no .okf.md files found in: {}", paths.join(", "));
        return ExitCode::from(2);
    }

    let mut total = 0usize;
    for file in &files {
        let display = file.display().to_string();
        let source = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {display}: {e}");
                return ExitCode::from(2);
            }
        };
        match check(&display, &source) {
            Ok(findings) => {
                for f in &findings {
                    println!("{f}");
                }
                total += findings.len();
                if !quiet && findings.is_empty() {
                    eprintln!("ok: {display}");
                }
            }
            Err(e) => {
                eprintln!("error: {display}: {e}");
                return ExitCode::from(2);
            }
        }
    }

    if quiet {
        // findings already on stdout; nothing else
    } else if total == 0 {
        eprintln!("clean: 0 leaks in {} file(s)", files.len());
    } else {
        eprintln!("{total} leak(s) in {} file(s)", files.len());
    }

    if total == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

/// Collect `.okf.md` files from a path (a file is taken as-is; a directory is
/// searched recursively).
fn collect(path: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            collect(&entry?.path(), out)?;
        }
    } else if is_okf(path) {
        out.push(path.to_path_buf());
    }
    Ok(())
}

/// True for `*.okf.md`.
fn is_okf(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.ends_with(".okf.md"))
}
