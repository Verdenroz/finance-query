//! Core code must not reach into provider adapters, and `adapters/common`
//! must not know which providers exist.

use std::path::{Path, PathBuf};

const ADAPTERS_NEUTRAL: &[&str] = &["common", "keys", "singleton", "yahoo"];

/// Everything in `src/providers` that is not a provider bridge.
const PROVIDERS_NEUTRAL: &[&str] = &[
    "adapter",
    "build",
    "build_options",
    "build_providers",
    "config",
    "convert",
    "edgar",
    "health",
    "local_exchanges",
    "market_calendar",
    "mock",
    "operation",
    "page",
    "retry",
    "routes",
    "set",
    "statement_metrics",
    "stock_bars",
    "stock_listing",
    "tests",
    "types",
    "yahoo",
    "yahoo_ttm",
];

const ALLOWED_DIRS: &[&str] = &["src/streaming/polygon/"];

/// `src/lib.rs` may re-export adapters (`pub use`) and wrap them in the
/// `__fuzz_*` and `__bench` items. These public wrappers are the rest.
const LIB_WRAPPERS: &[&str] = &["adapters::openfigi", "adapters::defillama"];

/// These modules stay feature-gated until `adapters/common` becomes
/// `core::support` (`#[doc(hidden)] pub`), which removes their dead-code lints.
const COMMON_PENDING: &[&str] = &["src/adapters/common/keyed.rs", "src/adapters/common/mod.rs"];

/// Files still calling provider adapters directly. Remove entries as they are
/// fixed; never add one.
const PENDING: &[&str] = &[
    "src/finance.rs",
    "src/streaming/economic.rs",
    "src/ticker/core.rs",
    "src/tickers/core/corporate.rs",
];

fn manifest(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("source dir is readable") {
        let path = entry.expect("dir entry is readable").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn files_under(relative_dir: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    rust_files(&manifest(relative_dir), &mut files);
    files
}

fn relative(path: &Path) -> String {
    path.strip_prefix(env!("CARGO_MANIFEST_DIR"))
        .expect("path is under the manifest dir")
        .to_string_lossy()
        .replace('\\', "/")
}

fn is_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn leading_ident(text: &str) -> &str {
    let end = text.find(|c| !is_ident(c)).unwrap_or(text.len());
    &text[..end]
}

/// First segment of every item in a `{a::b, c}` group, nested groups included.
fn group_heads(group: &str) -> Vec<&str> {
    let mut heads = Vec::new();
    let mut depth = 0;
    let mut expecting = false;
    let mut skip_to = 0;
    for (at, c) in group.char_indices() {
        if at < skip_to {
            continue;
        }
        match c {
            '{' => {
                depth += 1;
                expecting = depth == 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            ',' => expecting = depth == 1,
            c if expecting && is_ident(c) => {
                let head = leading_ident(&group[at..]);
                heads.push(head);
                skip_to = at + head.len();
                expecting = false;
            }
            _ => {}
        }
    }
    heads
}

/// `(line, "<module>::<name>")` for each provider adapter or bridge a source
/// file names. Paths into other crates' modules of the same name
/// (`tokio_stream::adapters::`) do not count.
fn provider_refs(source: &str) -> Vec<(usize, String)> {
    let mut found = module_refs(source, "adapters", ADAPTERS_NEUTRAL);
    found.extend(module_refs(source, "providers", PROVIDERS_NEUTRAL));
    found.sort_by_key(|(line, _)| *line);
    found
}

fn module_refs(source: &str, module: &str, neutral: &[&str]) -> Vec<(usize, String)> {
    let marker = format!("{module}::");
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(offset) = source[from..].find(&marker) {
        let at = from + offset;
        from = at + marker.len();
        let before = &source[..at];
        let local =
            !before.ends_with("::") || before.ends_with("crate::") || before.ends_with("super::");
        if !local || before.ends_with(is_ident) {
            continue;
        }
        let rest = &source[from..];
        let names = if rest.starts_with('{') {
            group_heads(rest)
        } else {
            vec![leading_ident(rest)]
        };
        let line = before.matches('\n').count() + 1;
        for name in names {
            let is_module = name.starts_with(|c: char| c.is_ascii_lowercase());
            if is_module && name != "self" && !neutral.contains(&name) {
                found.push((line, format!("{marker}{name}")));
            }
        }
    }
    found
}

fn refs_in(path: &Path) -> Vec<(usize, String)> {
    let source = std::fs::read_to_string(path).expect("source file is readable");
    provider_refs(&source)
}

/// The `__` items must stay last in `src/lib.rs`; everything after the first is exempt.
fn lib_refs() -> Vec<(usize, String)> {
    let source = std::fs::read_to_string(manifest("src/lib.rs")).expect("lib.rs is readable");
    let lines: Vec<&str> = source.lines().collect();
    let hidden_items = lines
        .iter()
        .position(|line| line.starts_with("pub fn __") || line.starts_with("pub mod __"))
        .expect("lib.rs has __ items");
    provider_refs(&source)
        .into_iter()
        .filter(|(line, name)| {
            *line <= hidden_items
                && !lines[line - 1].trim_start().starts_with("pub use")
                && !LIB_WRAPPERS.contains(&name.as_str())
        })
        .collect()
}

/// Lines that name a feature, whether in `cfg`, `cfg_attr`, `any` or `all`.
fn feature_lines(source: &str) -> Vec<(usize, String)> {
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("feature = \""))
        .map(|(index, line)| (index + 1, line.trim().to_string()))
        .collect()
}

fn describe(path: &str, refs: Vec<(usize, String)>) -> Vec<String> {
    refs.into_iter()
        .map(|(line, name)| format!("{path}:{line}: {name}"))
        .collect()
}

#[test]
fn core_code_does_not_call_provider_adapters() {
    let mut offenders = describe("src/lib.rs", lib_refs());
    for path in files_under("src") {
        let rel = relative(&path);
        let exempt = rel == "src/lib.rs"
            || rel.starts_with("src/adapters/")
            || rel.starts_with("src/providers/")
            || ALLOWED_DIRS.iter().any(|dir| rel.starts_with(dir))
            || PENDING.contains(&rel.as_str());
        if !exempt {
            offenders.extend(describe(&rel, refs_in(&path)));
        }
    }
    assert!(
        offenders.is_empty(),
        "core code references provider adapters:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn pending_files_still_call_provider_adapters() {
    let fixed: Vec<&str> = PENDING
        .iter()
        .copied()
        .filter(|rel| refs_in(&manifest(rel)).is_empty())
        .collect();
    assert!(
        fixed.is_empty(),
        "remove from PENDING, no longer coupled:\n{}",
        fixed.join("\n")
    );
}

#[test]
fn adapters_common_is_provider_neutral() {
    let mut offenders = Vec::new();
    for path in files_under("src/adapters/common") {
        let rel = relative(&path);
        let source = std::fs::read_to_string(&path).expect("source file is readable");
        if !COMMON_PENDING.contains(&rel.as_str()) {
            offenders.extend(
                feature_lines(&source)
                    .into_iter()
                    .map(|(line, text)| format!("{rel}:{line}: {text}")),
            );
        }
        offenders.extend(describe(&rel, provider_refs(&source)));
    }
    assert!(
        offenders.is_empty(),
        "adapters/common names a provider:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn common_pending_files_are_still_feature_gated() {
    let fixed: Vec<&str> = COMMON_PENDING
        .iter()
        .copied()
        .filter(|rel| {
            let source = std::fs::read_to_string(manifest(rel)).expect("source file is readable");
            feature_lines(&source).is_empty()
        })
        .collect();
    assert!(
        fixed.is_empty(),
        "remove from COMMON_PENDING, no longer feature-gated:\n{}",
        fixed.join("\n")
    );
}

#[test]
fn lib_wrappers_still_call_their_adapters() {
    let source = std::fs::read_to_string(manifest("src/lib.rs")).expect("lib.rs is readable");
    let called = names(&source);
    let fixed: Vec<&str> = LIB_WRAPPERS
        .iter()
        .copied()
        .filter(|wrapper| !called.iter().any(|name| name == wrapper))
        .collect();
    assert!(
        fixed.is_empty(),
        "remove from LIB_WRAPPERS, no longer called:\n{}",
        fixed.join("\n")
    );
}

fn names(source: &str) -> Vec<String> {
    provider_refs(source)
        .into_iter()
        .map(|(_, name)| name)
        .collect()
}

#[test]
fn scanner_flags_a_path() {
    assert_eq!(
        names("let x = crate::adapters::fmp::quote();"),
        ["adapters::fmp"]
    );
    assert_eq!(
        names("use super::super::adapters::polygon::x;"),
        ["adapters::polygon"]
    );
}

#[test]
fn scanner_flags_every_name_in_a_group() {
    assert_eq!(
        names("use crate::adapters::{fmp, polygon};"),
        ["adapters::fmp", "adapters::polygon"]
    );
    assert_eq!(
        names("use crate::adapters::{\n    fred::{a, b},\n    edgar::x::{y, z},\n};"),
        ["adapters::fred", "adapters::edgar"]
    );
}

#[test]
fn scanner_flags_a_bare_module_import() {
    assert_eq!(
        names("use crate::adapters;\nlet x = adapters::fmp::y();"),
        ["adapters::fmp"]
    );
}

#[test]
fn scanner_passes_neutral_and_foreign_paths() {
    assert!(names("use crate::adapters::yahoo::x;").is_empty());
    assert!(names("use crate::adapters::{common, keys::y};").is_empty());
    assert!(names("use tokio_stream::adapters::ChunksTimeout;").is_empty());
}

#[test]
fn scanner_flags_provider_bridges() {
    assert_eq!(
        names("let p = crate::providers::fmp::FmpProvider::new();"),
        ["providers::fmp"]
    );
    assert_eq!(
        names("use crate::providers::{Capability, bls::BlsProvider, polygon};"),
        ["providers::bls", "providers::polygon"]
    );
    assert_eq!(
        names("use crate::providers;\nlet p = providers::kraken::K;"),
        ["providers::kraken"]
    );
}

#[test]
fn scanner_passes_neutral_provider_modules() {
    assert!(names("use crate::providers::{Capability, Operation, ProviderSet};").is_empty());
    assert!(names("use crate::providers::mock::MockProvider;").is_empty());
    assert!(names("use crate::providers::{set::X, yahoo::Y, build_providers};").is_empty());
}
