//! MQL4 and MQL5 grammar used by Entanglement.

use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_mql5() -> *const ();
}

/// Return the MQL4 and MQL5 grammar.
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_mql5) };
