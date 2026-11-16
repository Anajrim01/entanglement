//! Resolve names that a wildcard import (`#include`) makes visible.

use std::collections::BTreeSet;

use crate::graph::FileFactsView;
use crate::model::{Reference, Resolution};

use super::{Index, Target, to_resolution};

/// Whether the file's language lets a wildcard import provide unbound names.
pub(super) fn apply(fact: &FileFactsView<'_>) -> bool {
    fact.file_module_rules
        .is_some_and(|rules| rules.wildcard_imports_resolve)
}

impl Index<'_, '_> {
    pub(super) fn resolve_through_includes(
        &self,
        file: usize,
        fact: &FileFactsView<'_>,
        reference: &Reference,
    ) -> Resolution {
        let targets = self.resolve_included_name(
            file,
            fact,
            &reference.path[0],
            (reference.start_byte, reference.context_id),
            &mut BTreeSet::new(),
        );
        to_resolution(self.resolve_member_path(targets, &reference.path[1..]))
    }

    /// Find `name` in the files this file includes, and in what those include.
    fn resolve_included_name(
        &self,
        file: usize,
        fact: &FileFactsView<'_>,
        name: &str,
        (position, context_id): (usize, usize),
        visiting: &mut BTreeSet<(usize, String)>,
    ) -> Vec<Target> {
        fact.imports()
            .filter(|import| {
                import.imported_name.as_deref() == Some("*")
                    && import.context_id == context_id
                    && import.scope_start <= position
                    && position < import.scope_end
            })
            .filter_map(|import| import.source.as_deref())
            .flat_map(|source| self.resolve_module(file, source))
            .filter_map(|module| match module {
                Target::Namespace(included) => Some(included),
                _ => None,
            })
            .flat_map(|included| {
                let own = self.resolve_export(included, name, visiting);
                if !own.is_empty() || !visiting.insert((included, format!("#include {name}"))) {
                    return own;
                }
                self.resolve_included_name(
                    included,
                    &self.facts[included],
                    name,
                    (0, context_id),
                    visiting,
                )
            })
            .collect()
    }
}
