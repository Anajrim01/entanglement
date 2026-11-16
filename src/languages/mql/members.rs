//! Fields and methods declared inside an MQL class, struct, or interface.

use tree_sitter::Node;

use super::declarations::{add, declaration};
use super::syntax::{declared_name, is_function};
use crate::graph::{DeclarationKind, SemanticVisibility};
use crate::languages::SemanticStore;
use crate::languages::syntax::text;
use crate::model::DefinitionKind;

pub(super) fn collect(nodes: &[Node<'_>], source: &[u8], facts: &mut SemanticStore) {
    for node in nodes.iter().filter(|node| in_class(**node)) {
        add_member(facts, *node, source);
    }
}

fn in_class(node: Node<'_>) -> bool {
    matches!(node.kind(), "field_declaration" | "function_definition")
        && node
            .parent()
            .is_some_and(|parent| parent.kind() == "field_declaration_list")
}

fn add_member(facts: &mut SemanticStore, node: Node<'_>, source: &[u8]) {
    let Some(name) = declared_name(node).map(|name| text(name, source)) else {
        return;
    };
    let Some(body) = node.parent() else { return };
    let Some(class) = body.parent() else { return };
    let Some(owner) = class
        .child_by_field_name("name")
        .map(|owner| text(owner, source))
    else {
        return;
    };
    let method = is_function(node);
    let interface = class.kind() == "interface_specifier";
    let kind = match (method, name == owner) {
        (false, _) => DeclarationKind::Property,
        (true, true) => DeclarationKind::Constructor,
        (true, false) => DeclarationKind::Function,
    };
    let mut declaration = declaration(node, kind, vec![owner]);
    declaration.visibility = visibility(node, class, source);
    declaration.is_interface_member = interface;
    declaration.modifiers.is_abstract =
        method && (interface || node.child_by_field_name("default_value").is_some());
    let definition = if method {
        DefinitionKind::Method
    } else {
        DefinitionKind::Static
    };
    add(
        facts,
        node,
        name,
        definition,
        declaration,
        body.start_byte()..body.end_byte(),
    );
}

/// An access section applies until the next one; only classes start private.
fn visibility(member: Node<'_>, class: Node<'_>, source: &[u8]) -> SemanticVisibility {
    let section = std::iter::successors(member.prev_named_sibling(), |sibling| {
        sibling.prev_named_sibling()
    })
    .find(|sibling| sibling.kind() == "access_specifier");
    match section.map(|section| text(section, source)).as_deref() {
        Some("public") => SemanticVisibility::Public,
        Some("protected") => SemanticVisibility::Protected,
        Some("private") => SemanticVisibility::Private,
        _ if class.kind() == "class_specifier" => SemanticVisibility::Private,
        _ => SemanticVisibility::Public,
    }
}
