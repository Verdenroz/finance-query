//! Asking the crate what it can do, from outside it.
//!
//! The capability matrix in `docs/library/providers/index.md` is generated
//! from these calls, so anything that table shows has to be reachable here.

use finance_query::{Capability, Provider};

#[test]
fn every_capability_is_enumerable_and_named() {
    let all: Vec<Capability> = Capability::all().collect();
    assert_eq!(all.len(), 15);

    for capability in &all {
        assert_ne!(capability.name(), "unknown", "{capability:?} has no name");
        assert!(capability.contains(*capability));
        assert!(!Capability::NONE.contains(*capability));
        assert!(capability.contains(Capability::NONE));
    }

    let names: Vec<&str> = all.iter().map(|c| c.name()).collect();
    assert!(names.contains(&"quote"));
    assert!(names.contains(&"discovery"));
    assert!(names.contains(&"filings"));
}

#[test]
fn a_provider_reports_what_it_serves() {
    let yahoo = Provider::Yahoo.capabilities();
    assert!(yahoo.contains(Capability::QUOTE));
    assert!(yahoo.contains(Capability::CHART));
    assert!(!yahoo.contains(Capability::ECONOMIC));

    assert_eq!(
        Provider::custom("introspection").capabilities(),
        Capability::NONE
    );
}

#[test]
fn a_capability_reports_who_serves_it() {
    let quote = Capability::QUOTE.candidate_providers();
    assert!(quote.contains(&Provider::Yahoo));

    for provider in Capability::FILINGS.candidate_providers() {
        assert!(
            provider.capabilities().contains(Capability::FILINGS),
            "{provider} was listed for FILINGS but does not declare it"
        );
    }
}

#[test]
fn union_composes_in_a_const() {
    const BOTH: Capability = Capability::QUOTE.union(Capability::CHART);
    assert!(BOTH.contains(Capability::QUOTE));
    assert!(BOTH.contains(Capability::CHART));
    assert!(!BOTH.contains(Capability::OPTIONS));
    assert_eq!(BOTH, Capability::QUOTE | Capability::CHART);
}

#[test]
fn the_capability_matrix_is_derivable() {
    let mut rows = 0;
    for provider in Provider::all() {
        let served: Vec<&str> = Capability::all()
            .filter(|c| provider.capabilities().contains(*c))
            .map(|c| c.name())
            .collect();
        // Every built-in serves something; only a custom provider is empty.
        assert!(!served.is_empty(), "{provider} declares no capability");
        rows += 1;
    }
    assert_eq!(rows, Provider::all().len());
    assert!(rows >= 4, "expected at least the always-compiled providers");
}
