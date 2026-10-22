//! Regenerate the shared header and footer inside static pages.
//!
//! The landing and the dashboard are wasm bundles, so they call the chrome
//! builders directly. The static pages (`/download/`, `404.html`) cannot, so
//! they carry the generated markup between markers and this binary refreshes
//! it:
//!
//! ```text
//! cargo run -p chrome --bin chrome-static -- --write ../src/root-files
//! cargo run -p chrome --bin chrome-static -- --check ../src/root-files
//! ```
//!
//! `--check` is what CI runs: it fails when a page is stale, so a header edit
//! cannot ship on some pages and not others.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use chrome::{
    Counts, FOOTER_START, FooterLabels, NAV_START, NavLabels, footer_html, inject_page, nav_html,
};

fn main() -> ExitCode {
    let mut write = false;
    let mut check = false;
    let mut roots: Vec<PathBuf> = Vec::new();

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--write" => write = true,
            "--check" => check = true,
            "-h" | "--help" => {
                println!("usage: chrome-static --write|--check <file-or-dir>...");
                return ExitCode::SUCCESS;
            }
            other => roots.push(PathBuf::from(other)),
        }
    }

    if write == check {
        eprintln!("chrome-static: pass exactly one of --write or --check");
        return ExitCode::from(2);
    }
    if roots.is_empty() {
        eprintln!("chrome-static: no paths given");
        return ExitCode::from(2);
    }

    let nav = nav_html(&NavLabels::english(), &Counts::default(), None);
    let footer = footer_html(&FooterLabels::english(), &Counts::default());

    let mut pages = Vec::new();
    for root in &roots {
        collect(root, &mut pages);
    }
    pages.sort();

    let mut stale: Vec<PathBuf> = Vec::new();
    for page in pages {
        let source = match std::fs::read_to_string(&page) {
            Ok(source) => source,
            Err(err) => {
                eprintln!("chrome-static: {}: {err}", page.display());
                return ExitCode::FAILURE;
            }
        };
        if !source.contains(NAV_START) && !source.contains(FOOTER_START) {
            continue;
        }
        let rendered = match inject_page(&source, &nav, &footer) {
            Ok(rendered) => rendered,
            Err(err) => {
                eprintln!("chrome-static: {}: {err}", page.display());
                return ExitCode::FAILURE;
            }
        };
        if rendered == source {
            continue;
        }
        if write {
            if let Err(err) = std::fs::write(&page, &rendered) {
                eprintln!("chrome-static: {}: {err}", page.display());
                return ExitCode::FAILURE;
            }
            println!("updated {}", page.display());
        } else {
            stale.push(page);
        }
    }

    if !stale.is_empty() {
        for page in &stale {
            eprintln!("stale chrome: {}", page.display());
        }
        eprintln!(
            "chrome-static: {} page(s) need regenerating, run --write",
            stale.len()
        );
        return ExitCode::FAILURE;
    }

    if check {
        println!("chrome-static: every page carries the current header and footer");
    }
    ExitCode::SUCCESS
}

fn collect(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_dir() {
        let Ok(entries) = std::fs::read_dir(path) else {
            return;
        };
        for entry in entries.flatten() {
            collect(&entry.path(), out);
        }
    } else if path.extension().is_some_and(|ext| ext == "html") {
        out.push(path.to_path_buf());
    }
}
