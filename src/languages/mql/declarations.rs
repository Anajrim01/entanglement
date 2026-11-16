//! Types, functions, and members declared in MQL source.

use std::ops::Range;

use tree_sitter::Node;

use super::members;
use super::syntax::{declared_name, is_type_definition, qualifier};
use crate::graph::{DeclarationFacts, DeclarationKind, SemanticVisibility};
use crate::languages::SemanticStore;
use crate::languages::syntax::text;
use crate::model::{Definition, DefinitionKind, Export, ModulePath};

pub(super) fn collect(
    nodes: &[Node<'_>],
    root: Node<'_>,
    source: &[u8],
    facts: &mut SemanticStore,
) {
    let whole_file = root.start_byte()..root.end_byte();
    for node in nodes.iter().filter(|node| is_type_definition(**node)) {
        let Some(name) = node.child_by_field_name("name") else {
            continue;
        };
        let kind = match node.kind() {
            "interface_specifier" => DefinitionKind::Trait,
            "enum_specifier" => DefinitionKind::Enum,
            _ => DefinitionKind::Struct,
        };
        let declaration = declaration(*node, DeclarationKind::Class, Vec::new());
        add(
            facts,
            *node,
            text(name, source),
            kind,
            declaration,
            whole_file.clone(),
        );
    }
    members::collect(nodes, source, facts);
    // An out-of-class definition borrows the access of its in-class prototype.
    for node in nodes.iter().filter(|node| {
        node.kind() == "function_definition"
            && node
                .parent()
                .is_some_and(|parent| parent.kind() != "field_declaration_list")
    }) {
        add_function(facts, *node, source, whole_file.clone());
    }
}

fn add_function(facts: &mut SemanticStore, node: Node<'_>, source: &[u8], scope: Range<usize>) {
    let Some(name) = declared_name(node).map(|name| text(name, source)) else {
        return;
    };
    let owner: Vec<String> = qualifier(node)
        .map(|scope| text(scope, source))
        .into_iter()
        .collect();
    let mut declaration = declaration(node, DeclarationKind::Function, owner.clone());
    let kind = if owner.is_empty() {
        DefinitionKind::Function
    } else {
        declaration.visibility = facts
            .definitions
            .iter()
            .filter(|definition| definition.name == name)
            .filter_map(|definition| definition.declaration.as_deref())
            .find(|prototype| prototype.owner == owner)
            .map_or(SemanticVisibility::Public, |prototype| prototype.visibility);
        DefinitionKind::Method
    };
    add(facts, node, name, kind, declaration, scope);
}

pub(super) fn declaration(
    node: Node<'_>,
    kind: DeclarationKind,
    owner: Vec<String>,
) -> DeclarationFacts {
    DeclarationFacts {
        start_byte: node.start_byte(),
        owner,
        visibility: SemanticVisibility::Public,
        kind,
        ..DeclarationFacts::default()
    }
}

pub(super) fn add(
    facts: &mut SemanticStore,
    node: Node<'_>,
    name: String,
    kind: DefinitionKind,
    declaration: DeclarationFacts,
    scope: Range<usize>,
) {
    let top_level = declaration.owner.is_empty();
    let visibility = declaration.visibility;
    facts.definitions.push(Definition {
        declaration: Some(Box::new(declaration)),
        name: name.clone(),
        kind,
        module: ModulePath::default(),
        start_byte: node.start_byte(),
        end_byte: node.end_byte(),
        scope_start: scope.start,
        scope_end: scope.end,
        is_public: visibility == SemanticVisibility::Public,
        inline_module: false,
        external_module: false,
        context_id: 0,
    });
    if top_level {
        facts.exports.push(Export {
            source: None,
            imported_name: None,
            local_name: Some(name.clone()),
            exported_name: name,
            namespace: false,
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            scope_start: scope.start,
            scope_end: scope.end,
            context_id: 0,
        });
    }
}
