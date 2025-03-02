use rustpython_parser::ast::{self, Constant, Expr, Keyword, Ranged};

use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::helpers::SimpleCallArgs;
use ruff_python_semantic::analyze::logging;
use ruff_python_stdlib::logging::LoggingLevel;

use crate::checkers::ast::Checker;
use crate::registry::Rule;
use crate::rules::pyflakes::cformat::CFormatSummary;

/// ## What it does
/// Checks for too few positional arguments for a `logging` format string.
///
/// ## Why is this bad?
/// A `TypeError` will be raised if the statement is run.
///
/// ## Example
/// ```python
/// import logging
///
/// try:
///     function()
/// except Exception as e:
///     logging.error("%s error occurred: %s", e)
///     raise
/// ```
///
/// Use instead:
/// ```python
/// import logging
///
/// try:
///     function()
/// except Exception as e:
///     logging.error("%s error occurred: %s", type(e), e)
///     raise
/// ```
#[violation]
pub struct LoggingTooFewArgs;

impl Violation for LoggingTooFewArgs {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Not enough arguments for `logging` format string")
    }
}

/// ## What it does
/// Checks for too many positional arguments for a `logging` format string.
///
/// ## Why is this bad?
/// A `TypeError` will be raised if the statement is run.
///
/// ## Example
/// ```python
/// import logging
///
/// try:
///     function()
/// except Exception as e:
///     logging.error("Error occurred: %s", type(e), e)
///     raise
/// ```
///
/// Use instead:
/// ```python
/// import logging
///
/// try:
///     function()
/// except Exception as e:
///     logging.error("%s error occurred: %s", type(e), e)
///     raise
/// ```
#[violation]
pub struct LoggingTooManyArgs;

impl Violation for LoggingTooManyArgs {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Too many arguments for `logging` format string")
    }
}

/// PLE1205
/// PLE1206
pub(crate) fn logging_call(
    checker: &mut Checker,
    func: &Expr,
    args: &[Expr],
    keywords: &[Keyword],
) {
    // If there are any starred arguments, abort.
    if args.iter().any(Expr::is_starred_expr) {
        return;
    }

    // If there are any starred keyword arguments, abort.
    if keywords.iter().any(|keyword| keyword.arg.is_none()) {
        return;
    }

    if !logging::is_logger_candidate(func, checker.semantic_model()) {
        return;
    }

    if let Expr::Attribute(ast::ExprAttribute { attr, .. }) = func {
        if LoggingLevel::from_attribute(attr.as_str()).is_some() {
            let call_args = SimpleCallArgs::new(args, keywords);
            if let Some(Expr::Constant(ast::ExprConstant {
                value: Constant::Str(value),
                ..
            })) = call_args.argument("msg", 0)
            {
                if let Ok(summary) = CFormatSummary::try_from(value.as_str()) {
                    if summary.starred {
                        return;
                    }
                    if !summary.keywords.is_empty() {
                        return;
                    }

                    let message_args = call_args.args.len() - 1;

                    if checker.enabled(Rule::LoggingTooManyArgs) {
                        if summary.num_positional < message_args {
                            checker
                                .diagnostics
                                .push(Diagnostic::new(LoggingTooManyArgs, func.range()));
                        }
                    }

                    if checker.enabled(Rule::LoggingTooFewArgs) {
                        if message_args > 0
                            && call_args.kwargs.is_empty()
                            && summary.num_positional > message_args
                        {
                            checker
                                .diagnostics
                                .push(Diagnostic::new(LoggingTooFewArgs, func.range()));
                        }
                    }
                }
            }
        }
    }
}
