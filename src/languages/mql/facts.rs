use tree_sitter::Node;

use super::{declarations, references};
use crate::languages::{FileModuleLayout, FileModuleRules, SemanticStore};

pub(super) const MQL_FILE_MODULE_RULES: FileModuleRules = FileModuleRules {
    extensions: &["mqh", "mq5", "mq4"],
    remaps: &[],
    index_stems: &[],
    unresolved_prefixes: &[],
    layout: FileModuleLayout::SlashRelative,
    module_bindings_shadow: false,
    wildcard_imports_shadow: false,
    // `#include` makes every name of the included file visible.
    wildcard_imports_resolve: true,
};

pub(super) fn capture(root: Node<'_>, source: &[u8]) -> SemanticStore {
    let mut nodes = Vec::new();
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        nodes.push(node);
        let mut cursor = node.walk();
        pending.extend(node.named_children(&mut cursor));
    }
    let mut facts = SemanticStore::default();
    declarations::collect(&nodes, root, source, &mut facts);
    references::collect(&nodes, root, source, &mut facts);
    facts
}
