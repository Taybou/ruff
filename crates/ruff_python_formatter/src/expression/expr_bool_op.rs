use crate::comments::{leading_comments, Comments};
use crate::expression::parentheses::{
    default_expression_needs_parentheses, NeedsParentheses, Parentheses, Parenthesize,
};
use crate::prelude::*;
use ruff_formatter::{format_args, write};
use rustpython_parser::ast::{Boolop, ExprBoolOp};

#[derive(Default)]
pub struct FormatExprBoolOp;

impl FormatNodeRule<ExprBoolOp> for FormatExprBoolOp {
    fn fmt_fields(&self, item: &ExprBoolOp, f: &mut PyFormatter) -> FormatResult<()> {
        let ExprBoolOp {
            range: _,
            op,
            values,
        } = item;

        let operator = match op {
            Boolop::And => "and",
            Boolop::Or => "or",
        };

        let mut values = values.iter();
        let comments = f.context().comments().clone();

        let Some(first) = values.next() else {
            return Ok(())
        };

        write!(f, [group(&first.format())])?;

        for value in values {
            let leading_value_comments = comments.leading_comments(value);
            // Format the expressions leading comments **before** the operator
            if !leading_value_comments.is_empty() {
                write!(
                    f,
                    [hard_line_break(), leading_comments(leading_value_comments)]
                )?;
            } else {
                write!(f, [space()])?;
            }

            write!(f, [text(operator), space(), group(&value.format())])?;
        }

        Ok(())
    }
}

impl NeedsParentheses for ExprBoolOp {
    fn needs_parentheses(
        &self,
        parenthesize: Parenthesize,
        source: &str,
        comments: &Comments,
    ) -> Parentheses {
        default_expression_needs_parentheses(self.into(), parenthesize, source, comments)
    }
}
