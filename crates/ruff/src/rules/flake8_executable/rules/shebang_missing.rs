#![allow(unused_imports)]

use std::path::Path;

use ruff_text_size::TextRange;

use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};

use crate::registry::AsRule;
#[cfg(target_family = "unix")]
use crate::rules::flake8_executable::helpers::is_executable;

/// ## What it does
/// Checks for executable `.py` files that do not have a shebang.
///
/// ## Why is this bad?
/// In Python, a shebang (also known as a hashbang) is the first line of a
/// script, which specifies the interpreter that should be used to run the
/// script.
///
/// If a `.py` file is executable, but does not have a shebang, it may be run
/// with the wrong interpreter, or fail to run at all.
///
/// If the file is meant to be executable, add a shebang; otherwise, remove the
/// executable bit from the file.
///
/// _This rule is only available on Unix-like systems._
///
/// ## References
/// - [Python documentation: Executable Python Scripts](https://docs.python.org/3/tutorial/appendix.html#executable-python-scripts)
#[violation]
pub struct ShebangMissingExecutableFile;

impl Violation for ShebangMissingExecutableFile {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("The file is executable but no shebang is present")
    }
}

/// EXE002
#[cfg(target_family = "unix")]
pub(crate) fn shebang_missing(filepath: &Path) -> Option<Diagnostic> {
    if let Ok(true) = is_executable(filepath) {
        let diagnostic = Diagnostic::new(ShebangMissingExecutableFile, TextRange::default());
        return Some(diagnostic);
    }
    None
}

#[cfg(not(target_family = "unix"))]
pub(crate) fn shebang_missing(_filepath: &Path) -> Option<Diagnostic> {
    None
}
