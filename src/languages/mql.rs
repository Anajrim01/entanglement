//! MQL4 and MQL5 metrics and semantic facts from the C++-derived upstream grammar.

mod declarations;
mod facts;
mod members;
mod references;
pub(crate) mod syntax;

use tree_sitter::{Language, Tree};

use super::query::QueryAnalyzer;
use super::{CapturedTree, LanguageHandler, LanguageSpec, ResolutionFamily};
use crate::Result;

pub(super) const LANGUAGE_SPEC: LanguageSpec = LanguageSpec {
    scope: "source.mql5",
    resolution_family: ResolutionFamily::FileModules,
    project_manifests: &[],
    file_module_rules: Some(&facts::MQL_FILE_MODULE_RULES),
    build,
};

fn build(_: &str, language: &Language) -> Result<Box<dyn LanguageHandler>> {
    Ok(Box::new(Analyzer(QueryAnalyzer::new(
        language,
        include_str!("../queries/mql5.scm"),
    )?)))
}

struct Analyzer(QueryAnalyzer);

impl LanguageHandler for Analyzer {
    fn capture(&self, tree: &Tree, source: &[u8], include_tokens: bool) -> Result<CapturedTree> {
        let shared = self.0.capture(tree, source, include_tokens);
        Ok(CapturedTree::from(shared).with_semantics(facts::capture(tree.root_node(), source)))
    }
}
