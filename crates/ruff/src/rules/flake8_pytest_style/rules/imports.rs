use rustpython_parser::ast::{Ranged, Stmt};

use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};

#[violation]
pub struct PytestIncorrectPytestImport;

impl Violation for PytestIncorrectPytestImport {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Found incorrect import of pytest, use simple `import pytest` instead")
    }
}

fn is_pytest_or_subpackage(imported_name: &str) -> bool {
    imported_name == "pytest" || imported_name.starts_with("pytest.")
}

/// PT013
pub(crate) fn import(import_from: &Stmt, name: &str, asname: Option<&str>) -> Option<Diagnostic> {
    if is_pytest_or_subpackage(name) {
        if let Some(alias) = asname {
            if alias != name {
                return Some(Diagnostic::new(
                    PytestIncorrectPytestImport,
                    import_from.range(),
                ));
            }
        }
    }
    None
}

/// PT013
pub(crate) fn import_from(
    import_from: &Stmt,
    module: Option<&str>,
    level: Option<u32>,
) -> Option<Diagnostic> {
    // If level is not zero or module is none, return
    if let Some(level) = level {
        if level != 0 {
            return None;
        }
    };

    if let Some(module) = module {
        if is_pytest_or_subpackage(module) {
            return Some(Diagnostic::new(
                PytestIncorrectPytestImport,
                import_from.range(),
            ));
        }
    };

    None
}
