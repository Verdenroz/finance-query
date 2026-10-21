//! Every page under `docs/` must be reachable from the site nav.
//!
//! An unlisted page still builds and still passes `docs check`, so it goes
//! live unreachable and nothing says so.
//!
//! One direction only: the nav also names pages that `make docs-pages`
//! generates (the spec HTML and reconciliation status), which are gitignored
//! and absent from a fresh checkout.

use std::path::{Path, PathBuf};

fn repo(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn nav_pages() -> Vec<String> {
    std::fs::read_to_string(repo("soothfast.toml"))
        .expect("soothfast.toml is readable")
        .split('"')
        .filter(|piece| piece.ends_with(".md"))
        // Entries may be written as `Label: path.md`.
        .map(|piece| {
            piece
                .rsplit(": ")
                .next()
                .unwrap_or(piece)
                .trim()
                .to_string()
        })
        .collect()
}

fn markdown_under_docs(dir: &Path, found: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).expect("docs directory is readable") {
        let path = entry.expect("entry is readable").path();
        if path.is_dir() {
            markdown_under_docs(&path, found);
        } else if path.extension().is_some_and(|e| e == "md") {
            let relative = path
                .strip_prefix(repo("docs"))
                .expect("page lives under docs/");
            found.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
}

#[test]
fn every_docs_page_is_reachable_from_the_nav() {
    let nav = nav_pages();
    assert!(!nav.is_empty(), "no nav pages parsed — parser is broken");

    let mut pages = Vec::new();
    markdown_under_docs(&repo("docs"), &mut pages);
    assert!(!pages.is_empty(), "no pages found under docs/");

    let orphaned: Vec<&String> = pages.iter().filter(|p| !nav.contains(p)).collect();
    assert!(
        orphaned.is_empty(),
        "pages absent from soothfast.toml's nav, so unreachable on the site: {orphaned:?}"
    );
}
