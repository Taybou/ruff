use crate::comments::Comments;
use crate::expression::parentheses::{
    default_expression_needs_parentheses, NeedsParentheses, Parentheses, Parenthesize,
};
use crate::prelude::*;
use crate::FormatNodeRule;
use ruff_formatter::write;
use rustpython_parser::ast::{Constant, Expr, ExprAttribute, ExprConstant};

#[derive(Default)]
pub struct FormatExprAttribute;

impl FormatNodeRule<ExprAttribute> for FormatExprAttribute {
    fn fmt_fields(&self, item: &ExprAttribute, f: &mut PyFormatter) -> FormatResult<()> {
        let ExprAttribute {
            value,
            range: _,
            attr,
            ctx: _,
        } = item;

        let requires_space = matches!(
            value.as_ref(),
            Expr::Constant(ExprConstant {
                value: Constant::Int(_) | Constant::Float(_),
                ..
            })
        );

        write!(
            f,
            [
                item.value.format(),
                requires_space.then_some(space()),
                text("."),
                dynamic_text(&attr, None),
            ]
        )
    }
}

impl NeedsParentheses for ExprAttribute {
    fn needs_parentheses(
        &self,
        parenthesize: Parenthesize,
        source: &str,
        comments: &Comments,
    ) -> Parentheses {
        default_expression_needs_parentheses(self.into(), parenthesize, source, comments)
    }
}
