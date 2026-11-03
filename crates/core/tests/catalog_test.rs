use repot_core::catalog::{ENTRIES, describe, id};
use std::collections::HashSet;

// The UI looks translations up by these ids, so they must stay unique and well-formed.
#[test]
fn ids_are_unique_and_stable() {
    let mut seen = HashSet::new();
    for (key, _) in ENTRIES {
        assert!(seen.insert(*key), "duplicate catalog id {key}");
        let (kind, name) = key.split_once(':').expect("id has a kind prefix");
        assert!(["runtime", "file", "plugin"].contains(&kind), "{key}");
        assert!(
            !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "{key}"
        );
    }
}

#[test]
fn every_id_has_a_one_sentence_description() {
    for (key, text) in ENTRIES {
        assert_eq!(describe(key), Some(*text));
        assert_eq!(id(key).as_deref(), Some(*key));
        assert!(
            text.ends_with('.') && !text.trim_end_matches('.').contains(". "),
            "{key}"
        );
    }
}

#[test]
fn unknown_keys_have_no_id() {
    assert_eq!(id("file:unknown"), None);
    assert_eq!(describe("file:unknown"), None);
}
