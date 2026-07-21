//! `deon-check` — run deon's static checks over `.okf.md` norm files.
//!
//! Usage: `deon-check [--quiet] [--okf <bundle>] <path>...`
//!   Each path is a `.okf.md` file or a directory searched recursively for them.
//!   `--okf` enables GROUND-3 (anchor resolution) against an OKF concept bundle.
//! Exit: 0 = clean, 1 = findings, 2 = usage/IO/parse error.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use deon_check::{check, check_bundle, check_with_okf, Okf};

const USAGE: &str = "\
deon-check — static checks for deon norms (DESIGN §4)

Usage:
    deon-check [--quiet] [--okf <bundle>] <path>...

Arguments:
    <path>          a .okf.md file, or a directory searched recursively for them

Options:
    --okf <bundle>  also run the bundle-backed checks against this OKF concept
                    bundle (a .md file or a directory of them): GROUND-3
                    (resolve grounds.ref anchors) and coverage (check the
                    branches against the subject's declared state space). The
                    bundle's own state declarations are checked too, once.
    --quiet         print findings only (suppress the per-file/summary lines)
    -h, --help      show this help

Checks: leak detection (LEAK-1/2/3), grounding completeness (GROUND-1/2;
GROUND-3 only with --okf), coverage (COVER-1/2/3/4, only with --okf), conditional
conflict (CONFLICT-1/2/3), termination-at-seam (SEAM-1/2/3) and regime hygiene
(REGIME-1/2).

Exit codes:
    0  clean (no findings)   1  findings   2  usage / IO / parse error";

fn main() -> ExitCode {
    let mut quiet = false;
    let mut okf_path: Option<String> = None;
    let mut paths: Vec<String> = Vec::new();

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            "--quiet" => quiet = true,
            "--okf" => match args.next() {
                Some(p) => okf_path = Some(p),
                None => {
                    eprintln!("error: --okf needs a path argument");
                    return ExitCode::from(2);
                }
            },
            _ => paths.push(arg),
        }
    }
    if paths.is_empty() {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }

    let okf = match &okf_path {
        Some(p) => match Okf::load(Path::new(p)) {
            Ok(o) => {
                if !quiet {
                    eprintln!(
                        "okf: {} anchor(s), {} subject state space(s) from {p}",
                        o.len(),
                        o.subjects()
                    );
                }
                Some(o)
            }
            Err(e) => {
                eprintln!("error: --okf {p}: {e}");
                return ExitCode::from(2);
            }
        },
        None => None,
    };

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

    // The bundle is checked once, before the norm files that read it: its state
    // declarations are norm content, and a bundle that does not ground its own
    // claims cannot be trusted to judge coverage in the files below.
    if let Some(okf) = &okf {
        let findings = check_bundle(okf);
        for f in &findings {
            println!("{f}");
        }
        total += findings.len();
    }

    for file in &files {
        let display = file.display().to_string();
        let source = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {display}: {e}");
                return ExitCode::from(2);
            }
        };
        let mut findings = match check(&display, &source) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("error: {display}: {e}");
                return ExitCode::from(2);
            }
        };
        if let Some(okf) = &okf {
            match check_with_okf(&display, &source, okf) {
                Ok(f) => findings.extend(f),
                Err(e) => {
                    eprintln!("error: {display}: {e}");
                    return ExitCode::from(2);
                }
            }
        }
        for f in &findings {
            println!("{f}");
        }
        total += findings.len();
        if !quiet && findings.is_empty() {
            eprintln!("ok: {display}");
        }
    }

    if !quiet {
        let ground3 = if okf.is_some() {
            ""
        } else {
            " (GROUND-3 + coverage skipped — pass --okf for the bundle-backed checks)"
        };
        if total == 0 {
            eprintln!("clean: 0 findings in {} file(s){ground3}", files.len());
        } else {
            eprintln!("{total} finding(s) in {} file(s){ground3}", files.len());
        }
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
