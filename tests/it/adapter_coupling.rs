//! Core code must not reach into provider adapters, and `adapters/common`
//! must not know which providers exist.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use syn::meta::ParseNestedMeta;
use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::{Attribute, Expr, Item, ItemFn, ItemMod, ItemUse, LitStr, Macro, Token, UseTree};

/// Modules of `src/adapters` that are not provider adapters. Edgar and Yahoo
/// stay in core by design.
const ADAPTERS_NEUTRAL: &[&str] = &["common", "keys", "singleton", "edgar", "yahoo"];

/// Modules of `src/providers` that are not provider bridges. Edgar and Yahoo
/// stay in core by design; the helper functions re-exported at the root are not
/// modules, so they never count.
const PROVIDERS_NEUTRAL: &[&str] = &[
    "adapter",
    "build",
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

/// `src/lib.rs` may re-export adapters (`pub use`) and wrap them in `__` items.
/// These public wrappers, with their reference counts, are the rest.
const LIB_WRAPPERS: &[(&str, usize)] = &[("adapters::openfigi", 6), ("adapters::defillama", 2)];

/// Files in `adapters/common` that still name a provider or provider feature,
/// with their violation counts. These modules stay feature-gated until
/// `adapters/common` becomes `core::support` (`#[doc(hidden)] pub`), which
/// removes their dead-code lints. Lower a count when it shrinks; never raise one.
const COMMON_PENDING: &[(&str, usize)] = &[
    ("src/adapters/common/keyed.rs", 18),
    ("src/adapters/common/mod.rs", 15),
];

/// Files still calling provider adapters directly, with their reference counts.
/// Lower a count when it shrinks; never raise one or add a file.
const PENDING: &[(&str, usize)] = &[];

#[derive(Default)]
struct Names {
    adapters: BTreeSet<String>,
    providers: BTreeSet<String>,
    /// Public `lib.rs` modules that re-export a provider adapter (`crate::fred`).
    reexports: BTreeSet<String>,
    /// Cargo features gating a provider adapter or bridge.
    features: BTreeSet<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Core,
    Common,
    Lib,
}

enum End {
    Plain,
    Glob,
    Rename,
}

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

fn parse(source: &str) -> syn::File {
    syn::parse_file(source).expect("source file parses")
}

fn parse_at(relative: &str) -> syn::File {
    parse(&std::fs::read_to_string(manifest(relative)).expect("source file is readable"))
}

fn collect_features(meta: ParseNestedMeta, out: &mut Vec<String>) -> syn::Result<()> {
    if meta.path.is_ident("feature") {
        out.push(meta.value()?.parse::<LitStr>()?.value());
    } else if meta.input.peek(syn::token::Paren) {
        meta.parse_nested_meta(|inner| collect_features(inner, out))?;
    } else if meta.input.peek(Token![=]) {
        meta.value()?.parse::<Expr>()?;
    }
    Ok(())
}

fn cfg_features(attr: &Attribute, out: &mut Vec<String>) {
    if attr.path().is_ident("cfg") || attr.path().is_ident("cfg_attr") {
        attr.parse_nested_meta(|meta| collect_features(meta, out))
            .expect("cfg attribute parses");
    }
}

fn flatten(tree: &UseTree, prefix: &mut Vec<String>, out: &mut Vec<(Vec<String>, End)>) {
    let mut leaf = |ident: &syn::Ident, end| {
        let mut segments = prefix.clone();
        if ident != "self" {
            segments.push(ident.to_string());
        }
        out.push((segments, end));
    };
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            flatten(&path.tree, prefix, out);
            prefix.pop();
        }
        UseTree::Name(name) => leaf(&name.ident, End::Plain),
        UseTree::Rename(rename) => leaf(&rename.ident, End::Rename),
        UseTree::Glob(_) => out.push((prefix.clone(), End::Glob)),
        UseTree::Group(group) => {
            for item in &group.items {
                flatten(item, prefix, out);
            }
        }
    }
}

fn use_paths(item: &ItemUse) -> Vec<(Vec<String>, End)> {
    let mut out = Vec::new();
    flatten(&item.tree, &mut Vec::new(), &mut out);
    out
}

fn bridges(relative: &str, neutral: &[&str]) -> Vec<(String, Vec<String>)> {
    parse_at(relative)
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Mod(module)
                if module.content.is_none()
                    && !neutral.contains(&module.ident.to_string().as_str()) =>
            {
                let mut features = Vec::new();
                for attr in &module.attrs {
                    cfg_features(attr, &mut features);
                }
                Some((module.ident.to_string(), features))
            }
            _ => None,
        })
        .collect()
}

impl Names {
    fn derive() -> Self {
        let mut names = Names::default();
        for (module, neutral, set) in [
            ("src/adapters/mod.rs", ADAPTERS_NEUTRAL, true),
            ("src/providers/mod.rs", PROVIDERS_NEUTRAL, false),
        ] {
            for (name, features) in bridges(module, neutral) {
                names.features.extend(features);
                if set {
                    names.adapters.insert(name);
                } else {
                    names.providers.insert(name);
                }
            }
        }
        for item in &parse_at("src/lib.rs").items {
            let Item::Mod(module) = item else { continue };
            let Some((_, items)) = &module.content else {
                continue;
            };
            let reexports_adapter = items.iter().any(|inner| match inner {
                Item::Use(item) => use_paths(item).iter().any(|(segments, _)| {
                    matches!(segments.as_slice(), [root, adapters, name, ..]
                        if root == "crate" && adapters == "adapters" && names.adapters.contains(name))
                }),
                _ => false,
            });
            if reexports_adapter {
                names.reexports.insert(module.ident.to_string());
            }
        }
        names
    }

    fn resolve(&self, segments: &[String], end: &End, mode: Mode) -> Option<String> {
        let lead = segments
            .iter()
            .take_while(|segment| matches!(segment.as_str(), "crate" | "self" | "super"))
            .count();
        let (head, rest) = segments.split_at(lead);
        let anchored = head.first().is_some_and(|segment| segment == "crate");
        let from_super = head.iter().any(|segment| segment == "super");
        let first = rest.first()?.as_str();
        match (first, rest.get(1)) {
            ("adapters", Some(name)) if self.adapters.contains(name) => {
                Some(format!("adapters::{name}"))
            }
            ("providers", Some(name)) if self.providers.contains(name) => {
                Some(format!("providers::{name}"))
            }
            ("adapters" | "providers", None) => match end {
                End::Glob => Some(format!("{first}::*")),
                End::Rename => Some(format!("{first} as alias")),
                End::Plain => None,
            },
            _ if anchored && self.reexports.contains(first) => Some(format!("crate::{first}")),
            _ if from_super && mode == Mode::Common && self.adapters.contains(first) => {
                Some(format!("adapters::{first}"))
            }
            _ => None,
        }
    }
}

fn names() -> &'static Names {
    static NAMES: OnceLock<Names> = OnceLock::new();
    NAMES.get_or_init(Names::derive)
}

struct Scanner<'a> {
    names: &'a Names,
    mode: Mode,
    found: Vec<String>,
}

impl Scanner<'_> {
    fn is_exempt(&self, ident: &syn::Ident) -> bool {
        self.mode == Mode::Lib && ident.to_string().starts_with("__")
    }
}

impl<'ast> Visit<'ast> for Scanner<'_> {
    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        if self.mode == Mode::Lib && matches!(item.vis, syn::Visibility::Public(_)) {
            return;
        }
        let mut seen = BTreeSet::new();
        for (segments, end) in use_paths(item) {
            if let Some(name) = self.names.resolve(&segments, &end, self.mode)
                && seen.insert(name.clone())
            {
                self.found.push(name);
            }
        }
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        if !self.is_exempt(&item.sig.ident) {
            syn::visit::visit_item_fn(self, item);
        }
    }

    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        if !self.is_exempt(&item.ident) {
            syn::visit::visit_item_mod(self, item);
        }
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        let segments: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
        if let Some(name) = self.names.resolve(&segments, &End::Plain, self.mode) {
            self.found.push(name);
        }
        syn::visit::visit_path(self, path);
    }

    fn visit_attribute(&mut self, attr: &'ast Attribute) {
        if self.mode == Mode::Common {
            let mut features = Vec::new();
            cfg_features(attr, &mut features);
            self.record_features(features);
        }
    }

    fn visit_macro(&mut self, mac: &'ast Macro) {
        self.visit_path(&mac.path);
        if mac.path.is_ident("cfg") {
            if self.mode == Mode::Common {
                let mut features = Vec::new();
                mac.parse_body_with(syn::meta::parser(|meta| {
                    collect_features(meta, &mut features)
                }))
                .expect("cfg! body parses");
                self.record_features(features);
            }
        } else if let Ok(args) =
            mac.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated)
        {
            for arg in &args {
                self.visit_expr(arg);
            }
        }
    }
}

impl Scanner<'_> {
    fn record_features(&mut self, features: Vec<String>) {
        for feature in features {
            if self.names.features.contains(&feature) {
                self.found.push(format!("feature \"{feature}\""));
            }
        }
    }
}

fn scan(source: &str, names: &Names, mode: Mode) -> Vec<String> {
    let mut scanner = Scanner {
        names,
        mode,
        found: Vec::new(),
    };
    scanner.visit_file(&parse(source));
    scanner.found
}

fn scan_at(relative: &str, mode: Mode) -> Vec<String> {
    let source = std::fs::read_to_string(manifest(relative)).expect("source file is readable");
    scan(&source, names(), mode)
}

fn lookup(list: &[(&str, usize)], key: &str) -> usize {
    list.iter()
        .find(|(name, _)| *name == key)
        .map_or(0, |(_, count)| *count)
}

fn tally(found: &[String]) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for name in found {
        *counts.entry(name.as_str()).or_insert(0) += 1;
    }
    counts
}

fn lowered(list: &[(&str, usize)], actual: impl Fn(&str) -> usize) -> Vec<String> {
    list.iter()
        .filter_map(|(key, allowed)| {
            let now = actual(key);
            (now < *allowed).then(|| format!("{key}: lower the count to {now}"))
        })
        .collect()
}

fn exceeded(rel: &str, found: &[String], allowed: usize) -> Option<String> {
    (found.len() > allowed).then(|| {
        format!(
            "{rel}: {} found, {allowed} allowed: {}",
            found.len(),
            found.join(", ")
        )
    })
}

#[test]
fn core_code_does_not_call_provider_adapters() {
    let mut offenders = Vec::new();
    let lib = scan_at("src/lib.rs", Mode::Lib);
    for (name, count) in tally(&lib) {
        let allowed = lookup(LIB_WRAPPERS, name);
        if count > allowed {
            offenders.push(format!(
                "src/lib.rs: {name} {count} found, {allowed} allowed"
            ));
        }
    }
    for path in files_under("src") {
        let rel = relative(&path);
        let exempt = rel == "src/lib.rs"
            || rel.starts_with("src/adapters/")
            || rel.starts_with("src/providers/")
            || ALLOWED_DIRS.iter().any(|dir| rel.starts_with(dir));
        if !exempt {
            let found = scan_at(&rel, Mode::Core);
            offenders.extend(exceeded(&rel, &found, lookup(PENDING, &rel)));
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
    let fixed = lowered(PENDING, |rel| scan_at(rel, Mode::Core).len());
    assert!(fixed.is_empty(), "PENDING shrank:\n{}", fixed.join("\n"));
}

#[test]
fn adapters_common_is_provider_neutral() {
    let mut offenders = Vec::new();
    for path in files_under("src/adapters/common") {
        let rel = relative(&path);
        let found = scan_at(&rel, Mode::Common);
        offenders.extend(exceeded(&rel, &found, lookup(COMMON_PENDING, &rel)));
    }
    assert!(
        offenders.is_empty(),
        "adapters/common names a provider:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn common_pending_files_are_still_feature_gated() {
    let fixed = lowered(COMMON_PENDING, |rel| scan_at(rel, Mode::Common).len());
    assert!(
        fixed.is_empty(),
        "COMMON_PENDING shrank:\n{}",
        fixed.join("\n")
    );
}

#[test]
fn lib_wrappers_still_call_their_adapters() {
    let lib = scan_at("src/lib.rs", Mode::Lib);
    let counts = tally(&lib);
    let fixed = lowered(LIB_WRAPPERS, |name| counts.get(name).copied().unwrap_or(0));
    assert!(
        fixed.is_empty(),
        "LIB_WRAPPERS shrank:\n{}",
        fixed.join("\n")
    );
}

fn fixture() -> Names {
    let set = |items: &[&str]| items.iter().map(|s| s.to_string()).collect();
    Names {
        adapters: set(&["fmp", "polygon", "fred"]),
        providers: set(&["fmp", "bls", "kraken", "polygon"]),
        reexports: set(&["fred", "crypto"]),
        features: set(&["fmp", "crypto"]),
    }
}

fn found(source: &str) -> Vec<String> {
    scan(source, &fixture(), Mode::Core)
}

fn found_in_common(source: &str) -> Vec<String> {
    scan(source, &fixture(), Mode::Common)
}

#[test]
fn derived_names_cover_providers_and_reexports() {
    let names = names();
    assert!(names.adapters.contains("fmp") && !names.adapters.contains("edgar"));
    assert!(names.providers.contains("fmp") && !names.providers.contains("yahoo"));
    assert!(names.reexports.contains("fred") && !names.reexports.contains("edgar"));
    assert!(names.features.contains("fmp") && !names.features.contains("dataframe"));
}

#[test]
fn scanner_flags_a_path() {
    assert_eq!(
        found("fn f() { crate::adapters::fmp::quote(); }"),
        ["adapters::fmp"]
    );
    assert_eq!(
        found("use super::super::adapters::polygon::x;"),
        ["adapters::polygon"]
    );
}

#[test]
fn scanner_flags_every_name_in_a_group() {
    assert_eq!(
        found("use crate::adapters::{fmp, polygon};"),
        ["adapters::fmp", "adapters::polygon"]
    );
}

#[test]
fn scanner_flags_nested_groups_once_per_module() {
    assert_eq!(
        found("use crate::adapters::{fred::{a, b}, yahoo::x::{y, z}, polygon::q};"),
        ["adapters::fred", "adapters::polygon"]
    );
}

#[test]
fn scanner_flags_a_glob_of_the_module() {
    assert_eq!(found("use crate::adapters::*;"), ["adapters::*"]);
    assert_eq!(found("use crate::providers::*;"), ["providers::*"]);
    assert_eq!(found("use crate::adapters::fmp::*;"), ["adapters::fmp"]);
}

#[test]
fn scanner_flags_an_alias_of_the_module() {
    assert_eq!(found("use crate::adapters as a;"), ["adapters as alias"]);
    assert_eq!(
        found("use crate::providers::{self as p};"),
        ["providers as alias"]
    );
}

#[test]
fn scanner_flags_a_bare_module_import_by_its_use() {
    assert_eq!(
        found("use crate::adapters;\nfn f() { adapters::fmp::y(); }"),
        ["adapters::fmp"]
    );
}

#[test]
fn scanner_ignores_comments_inside_groups() {
    assert_eq!(
        found("use crate::adapters::{\n    // fmp, not this one\n    polygon,\n};"),
        ["adapters::polygon"]
    );
    assert!(found("use crate::adapters::{\n    /* fmp */\n    yahoo,\n};").is_empty());
}

#[test]
fn scanner_ignores_docs_comments_and_strings() {
    assert!(
        found(
            "/// See [`crate::adapters::fmp`].\n// crate::providers::bls\nfn f() -> &'static str { \"crate::adapters::fmp::x\" }"
        )
        .is_empty()
    );
}

#[test]
fn scanner_flags_a_reexport_path() {
    assert_eq!(found("fn f() { crate::fred::series(); }"), ["crate::fred"]);
    assert_eq!(found("use crate::crypto::{a, b};"), ["crate::crypto"]);
}

#[test]
fn scanner_flags_paths_in_macro_arguments() {
    assert_eq!(
        found("fn f() { println!(\"{}\", crate::adapters::fmp::x()); }"),
        ["adapters::fmp"]
    );
}

#[test]
fn scanner_flags_super_provider_only_from_common() {
    assert_eq!(
        found_in_common("use super::fmp::Client;"),
        ["adapters::fmp"]
    );
    assert!(found("use super::fmp::Client;").is_empty());
}

#[test]
fn scanner_flags_provider_features_in_common_only() {
    assert_eq!(
        found_in_common("#[cfg(feature = \"fmp\")]\nfn f() {}"),
        ["feature \"fmp\""]
    );
    assert_eq!(
        found_in_common("#[cfg(any(test, not(feature = \"crypto\")))]\nfn f() {}"),
        ["feature \"crypto\""]
    );
    assert_eq!(
        found_in_common("#[cfg_attr(feature = \"fmp\", derive(Debug))]\nstruct S;"),
        ["feature \"fmp\""]
    );
    assert_eq!(
        found_in_common("fn f() -> bool { cfg!(feature = \"fmp\") }"),
        ["feature \"fmp\""]
    );
    assert!(found_in_common("#[cfg(feature = \"dataframe\")]\nfn f() {}").is_empty());
    assert!(found("#[cfg(feature = \"fmp\")]\nfn f() {}").is_empty());
}

#[test]
fn scanner_passes_neutral_and_foreign_paths() {
    assert!(found("use crate::adapters::yahoo::x;").is_empty());
    assert!(found("use crate::adapters::{common, keys::y, edgar};").is_empty());
    assert!(found("use tokio_stream::adapters::ChunksTimeout;").is_empty());
    assert!(found("use crate::providers::{Capability, Operation, build_providers};").is_empty());
    assert!(found("use crate::providers::{set::X, yahoo::Y, mock::M};").is_empty());
}

#[test]
fn scanner_flags_provider_bridges() {
    assert_eq!(
        found("fn f() { crate::providers::fmp::FmpProvider::new(); }"),
        ["providers::fmp"]
    );
    assert_eq!(
        found("use crate::providers::{Capability, bls::BlsProvider, polygon};"),
        ["providers::bls", "providers::polygon"]
    );
}

#[test]
fn lib_scan_exempts_pub_use_and_hidden_items() {
    let source = "pub mod fmp { pub use crate::adapters::fmp::init; }\n\
                  pub fn __fuzz() { crate::adapters::fmp::x(); }\n\
                  pub fn wrap() { crate::adapters::polygon::x(); }";
    assert_eq!(scan(source, &fixture(), Mode::Lib), ["adapters::polygon"]);
}
