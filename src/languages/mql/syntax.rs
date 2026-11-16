//! Declarator helpers shared by MQL semantic facts and structure capture.

use tree_sitter::Node;

fn declarators(first: Node<'_>) -> impl Iterator<Item = Node<'_>> {
    std::iter::successors(Some(first), |node| node.child_by_field_name("declarator"))
}

/// Name one declarator, looking through pointers, references, arrays, and the
/// parameter list. `None` for shapes such as function pointers.
fn name_of(declarator: Node<'_>) -> Option<Node<'_>> {
    let last = declarators(declarator).last()?;
    let name = match last.kind() {
        "qualified_identifier" => last.child_by_field_name("name")?,
        _ => last,
    };
    matches!(
        name.kind(),
        "identifier" | "field_identifier" | "destructor_name" | "operator_name"
    )
    .then_some(name)
}

/// Name a field, function, or prototype by its first declarator.
pub(crate) fn declared_name(declaration: Node<'_>) -> Option<Node<'_>> {
    name_of(declaration.child_by_field_name("declarator")?)
}

/// Names after the first in `int low, high;`.
pub(crate) fn extra_names(declaration: Node<'_>) -> Vec<Node<'_>> {
    let mut cursor = declaration.walk();
    declaration
        .children_by_field_name("declarator", &mut cursor)
        .skip(1)
        .filter_map(name_of)
        .collect()
}

/// The class named before `::` in an out-of-class method definition.
pub(crate) fn qualifier(declaration: Node<'_>) -> Option<Node<'_>> {
    declarators(declaration.child_by_field_name("declarator")?)
        .last()
        .filter(|last| last.kind() == "qualified_identifier")?
        .child_by_field_name("scope")
}

pub(crate) fn is_function(declaration: Node<'_>) -> bool {
    declaration
        .child_by_field_name("declarator")
        .is_some_and(|first| declarators(first).any(|node| node.kind() == "function_declarator"))
}

/// Whether the specifier defines a type here instead of only naming one.
pub(crate) fn is_type_definition(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "class_specifier" | "struct_specifier" | "interface_specifier" | "enum_specifier"
    ) && node.child_by_field_name("body").is_some()
}
