use std::fmt;

use rustpython_parser::ast::{self, Expr, Ranged, Stmt};

use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_semantic::model::SemanticModel;

use crate::checkers::ast::Checker;

use super::helpers;

/// ## What it does
/// Checks for the order of Model's inner classes, methods, and fields as per
/// the [Django Style Guide].
///
/// ## Why is this bad?
/// The [Django Style Guide] specifies that the order of Model inner classes,
/// attributes and methods should be as follows:
///
/// 1. All database fields
/// 2. Custom manager attributes
/// 3. `class Meta`
/// 4. `def __str__()`
/// 5. `def save()`
/// 6. `def get_absolute_url()`
/// 7. Any custom methods
///
/// ## Examples
/// ```python
/// from django.db import models
///
///
/// class StrBeforeFieldModel(models.Model):
///     class Meta:
///         verbose_name = "test"
///         verbose_name_plural = "tests"
///
///     def __str__(self):
///         return "foobar"
///
///     first_name = models.CharField(max_length=32)
///     last_name = models.CharField(max_length=40)
/// ```
///
/// Use instead:
/// ```python
/// from django.db import models
///
///
/// class StrBeforeFieldModel(models.Model):
///     first_name = models.CharField(max_length=32)
///     last_name = models.CharField(max_length=40)
///
///     class Meta:
///         verbose_name = "test"
///         verbose_name_plural = "tests"
///
///     def __str__(self):
///         return "foobar"
/// ```
///
/// [Django Style Guide]: https://docs.djangoproject.com/en/dev/internals/contributing/writing-code/coding-style/#model-style
#[violation]
pub struct DjangoUnorderedBodyContentInModel {
    elem_type: ContentType,
    before: ContentType,
}

impl Violation for DjangoUnorderedBodyContentInModel {
    #[derive_message_formats]
    fn message(&self) -> String {
        let DjangoUnorderedBodyContentInModel { elem_type, before } = self;
        format!("Order of model's inner classes, methods, and fields does not follow the Django Style Guide: {elem_type} should come before {before}")
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub(crate) enum ContentType {
    FieldDeclaration,
    ManagerDeclaration,
    MetaClass,
    StrMethod,
    SaveMethod,
    GetAbsoluteUrlMethod,
    CustomMethod,
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentType::FieldDeclaration => f.write_str("field declaration"),
            ContentType::ManagerDeclaration => f.write_str("manager declaration"),
            ContentType::MetaClass => f.write_str("`Meta` class"),
            ContentType::StrMethod => f.write_str("`__str__` method"),
            ContentType::SaveMethod => f.write_str("`save` method"),
            ContentType::GetAbsoluteUrlMethod => f.write_str("`get_absolute_url` method"),
            ContentType::CustomMethod => f.write_str("custom method"),
        }
    }
}

fn get_element_type(model: &SemanticModel, element: &Stmt) -> Option<ContentType> {
    match element {
        Stmt::Assign(ast::StmtAssign { targets, value, .. }) => {
            if let Expr::Call(ast::ExprCall { func, .. }) = value.as_ref() {
                if helpers::is_model_field(model, func) {
                    return Some(ContentType::FieldDeclaration);
                }
            }
            let Some(expr) = targets.first() else {
                return None;
            };
            let Expr::Name(ast::ExprName { id, .. }) = expr else {
                return None;
            };
            if id == "objects" {
                Some(ContentType::ManagerDeclaration)
            } else {
                None
            }
        }
        Stmt::ClassDef(ast::StmtClassDef { name, .. }) => {
            if name == "Meta" {
                Some(ContentType::MetaClass)
            } else {
                None
            }
        }
        Stmt::FunctionDef(ast::StmtFunctionDef { name, .. }) => match name.as_str() {
            "__str__" => Some(ContentType::StrMethod),
            "save" => Some(ContentType::SaveMethod),
            "get_absolute_url" => Some(ContentType::GetAbsoluteUrlMethod),
            _ => Some(ContentType::CustomMethod),
        },
        _ => None,
    }
}

/// DJ012
pub(crate) fn unordered_body_content_in_model(
    checker: &mut Checker,
    bases: &[Expr],
    body: &[Stmt],
) {
    if !bases
        .iter()
        .any(|base| helpers::is_model(checker.semantic_model(), base))
    {
        return;
    }
    let mut elements_type_found = Vec::new();
    for element in body.iter() {
        let Some(current_element_type) = get_element_type(checker.semantic_model(), element) else {
            continue;
        };
        let Some(&element_type) = elements_type_found
            .iter()
            .find(|&&element_type| element_type > current_element_type) else {
                elements_type_found.push(current_element_type);
                continue;
        };
        let diagnostic = Diagnostic::new(
            DjangoUnorderedBodyContentInModel {
                elem_type: current_element_type,
                before: element_type,
            },
            element.range(),
        );
        checker.diagnostics.push(diagnostic);
    }
}
