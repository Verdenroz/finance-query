//! Every `#[tool]` must have a `#[soothfast::route]` marker, and vice versa.
//!
//! `spec gen --check` reconciles markers against `mcp-tools.json` but never
//! against the tools themselves, so a tool with no marker is absent from the
//! manifest and a marker with no tool advertises something that cannot be
//! called. Neither shows up in any other gate.

const TOOLS: &str = include_str!("../src/tools/mod.rs");
const MARKERS: &str = include_str!("../spec/routes.rs");

fn declared_tools() -> Vec<String> {
    let mut names = Vec::new();
    for block in TOOLS.split("#[tool(").skip(1) {
        let Some(start) = block.find("async fn ") else {
            continue;
        };
        let rest = &block[start + "async fn ".len()..];
        let end = rest
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(rest.len());
        names.push(rest[..end].to_string());
    }
    names.sort();
    names
}

fn marker_operations() -> Vec<String> {
    let mut names: Vec<String> = MARKERS
        .split("operation = \"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next().map(str::to_string))
        .collect();
    names.sort();
    names
}

#[test]
fn every_tool_has_a_marker_and_every_marker_has_a_tool() {
    let tools = declared_tools();
    let markers = marker_operations();
    assert!(!tools.is_empty(), "no #[tool] found — parser is broken");

    let unmarked: Vec<&String> = tools.iter().filter(|t| !markers.contains(t)).collect();
    let unimplemented: Vec<&String> = markers.iter().filter(|m| !tools.contains(m)).collect();

    assert!(
        unmarked.is_empty(),
        "tools missing a spec/routes.rs marker, so absent from mcp-tools.json: {unmarked:?}"
    );
    assert!(
        unimplemented.is_empty(),
        "markers naming no #[tool], so advertised but uncallable: {unimplemented:?}"
    );
}
