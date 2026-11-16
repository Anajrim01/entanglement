//! Structural rules for MQL's C++-derived syntax.

use tree_sitter::Node;

use crate::languages::mql::syntax;
use crate::model::SemanticStore;

/// `Some(None)` rejects a specifier that only names a type;
/// `Some(Some(..))` classifies a prototype. `None` leaves the node to other languages.
pub(super) fn classification(node: Node<'_>) -> Option<Option<(&'static str, bool)>> {
    if matches!(
        node.kind(),
        "class_specifier" | "struct_specifier" | "interface_specifier" | "enum_specifier"
    ) && !syntax::is_type_definition(node)
    {
        return Some(None);
    }
    (node.kind() == "field_declaration" && syntax::is_function(node))
        .then_some(Some(("member", false)))
}

/// An in-class prototype whose definition is `Class::name(...) {...}` in this file.
/// Pure virtual prototypes stay: they are requirements, not duplicates.
pub(super) fn is_prototype_defined_elsewhere(
    node: Node<'_>,
    name: &str,
    owner: &[String],
    facts: &SemanticStore,
) -> bool {
    node.kind() == "field_declaration"
        && syntax::is_function(node)
        && facts.definitions.iter().any(|definition| {
            definition.name == name
                && definition.start_byte != node.start_byte()
                && definition
                    .declaration
                    .as_deref()
                    .is_some_and(|declaration| {
                        declaration.owner.last() == owner.last()
                            && !declaration.modifiers.is_abstract
                    })
        })
}
