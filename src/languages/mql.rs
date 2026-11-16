//! MQL4 and MQL5 metrics from the C++-derived upstream grammar.

use tree_sitter::{Language, Tree};

use super::query::QueryAnalyzer;
use super::{CapturedTree, LanguageHandler, LanguageSpec, ResolutionFamily};
use crate::Result;

pub(super) const LANGUAGE_SPEC: LanguageSpec = LanguageSpec {
    scope: "source.mql5",
    resolution_family: ResolutionFamily::None,
    project_manifests: &[],
    file_module_rules: None,
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
        Ok(self.0.capture(tree, source, include_tokens).into())
    }
}
