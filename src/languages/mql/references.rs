//! Type uses, calls, and `#include` lines in MQL source.

use tree_sitter::Node;

use super::syntax::is_type_definition;
use crate::languages::SemanticStore;
use crate::languages::syntax::{ancestors, text};
use crate::model::{Import, ModulePath, Reference, ReferenceKind};

pub(super) fn collect(
    nodes: &[Node<'_>],
    root: Node<'_>,
    source: &[u8],
    facts: &mut SemanticStore,
) {
    for node in nodes {
        match node.kind() {
            "type_identifier" if !names_definition(*node) => {
                add_reference(
                    facts,
                    *node,
                    vec![text(*node, source)],
                    ReferenceKind::Type,
                    root,
                );
            }
            "call_expression" => add_call(facts, *node, source, root),
            "preproc_include" => add_include(facts, *node, source, root),
            _ => {}
        }
    }
}

fn names_definition(name: Node<'_>) -> bool {
    name.parent().is_some_and(|parent| {
        is_type_definition(parent)
            && parent
                .child_by_field_name("name")
                .is_some_and(|declared| declared.id() == name.id())
    })
}

fn add_call(facts: &mut SemanticStore, call: Node<'_>, source: &[u8], root: Node<'_>) {
    let Some(callee) = call.child_by_field_name("function") else {
        return;
    };
    let path = match callee.kind() {
        "identifier" => vec![text(callee, source)],
        "qualified_identifier" => {
            let (Some(scope), Some(name)) = (
                callee.child_by_field_name("scope"),
                callee.child_by_field_name("name"),
            ) else {
                return;
            };
            vec![text(scope, source), text(name, source)]
        }
        _ => return,
    };
    add_reference(facts, callee, path, ReferenceKind::Call, root);
    let owner = ancestors(call)
        .find(|node| node.kind() == "function_definition")
        .map(|function| function.start_byte());
    if let Some(reference) = facts.references.last_mut() {
        reference.call_owner = owner;
        reference.execution_owner = owner;
    }
}

fn add_reference(
    facts: &mut SemanticStore,
    node: Node<'_>,
    path: Vec<String>,
    kind: ReferenceKind,
    root: Node<'_>,
) {
    facts.references.push(Reference {
        path,
        module: ModulePath::default(),
        kind,
        start_byte: node.start_byte(),
        end_byte: node.end_byte(),
        scope_start: root.start_byte(),
        scope_end: root.end_byte(),
        call_owner: None,
        execution_owner: None,
        access: None,
        context_id: 0,
    });
}

/// Record an include as a wildcard import. Quoted paths are relative to the
/// including file; angle-bracket paths name the terminal's library folder.
fn add_include(facts: &mut SemanticStore, include: Node<'_>, source: &[u8], root: Node<'_>) {
    let Some(path) = include.child_by_field_name("path") else {
        return;
    };
    let written = text(path, source);
    let written = written.trim_matches(['"', '<', '>']).replace('\\', "/");
    let specifier = if path.kind() == "string_literal" && !written.starts_with('.') {
        format!("./{written}")
    } else {
        written
    };
    facts.imports.push(Import {
        path: Vec::new(),
        alias: None,
        source: Some(specifier),
        imported_name: Some("*".to_owned()),
        namespace: false,
        start_byte: include.start_byte(),
        end_byte: include.end_byte(),
        module: ModulePath::default(),
        scope_start: root.start_byte(),
        scope_end: root.end_byte(),
        is_public: false,
        context_id: 0,
    });
}
