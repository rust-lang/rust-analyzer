// use ide_db::syntax_helpers::node_ext::is_pattern_cond;
use crate::{
    assist_context::{AssistContext, Assists},
    // utils::invert_boolean_expression,
    AssistId,
    AssistKind,
};
use syntax::ast::{self, vst::*, AstNode, LogicOp};

pub(crate) fn split_imply_ensures(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
   // setup basic variables
    let func: ast::Fn = ctx.find_node_at_offset::<ast::Fn>()?;
    let ensures: ast::EnsuresClause = func.ensures_clause()?;

    // trigger on "ensures"
    // check if cursor is on "ensures" keyword
    let ensures_keyword = ensures.ensures_token()?;
    let cursor_in_range = ensures_keyword.text_range().contains_range(ctx.selection_trimmed());
    if !cursor_in_range {
        return None;
    }

    let v_func = Fn::try_from(func.clone()).ok()?;
    let result = vst_rewriter_split_imply_ensures(v_func)?;
    let result = ctx.fmt(func.clone(),result.to_string())?;

    acc.add(
        AssistId("split_imply_ensures", AssistKind::RefactorRewrite),
        "Split implication in ensures into requires and ensures",
        func.syntax().text_range(),
        |edit: &mut ide_db::source_change::SourceChangeBuilder| {
            edit.replace(func.syntax().text_range(), result);
        },
    )
}

pub(crate) fn vst_rewriter_split_imply_ensures(mut func: Fn) -> Option<String> {
    let ensures = *func.ensures_clause.clone()?;
    // check if the number of ensures clause if 1
    if ensures.exprs.len() != 1 {
      return None;
    }
    let ensures_expr = ensures.exprs[0].clone();

    // if assertion's expression's top level is not implication, return None
    
    let (new_req, new_ens) = match ensures_expr {
        Expr::BinExpr(b) => {
            if b.op != BinaryOp::LogicOp(LogicOp::Imply) {
                dbg!("not an implication");
                return None;
            }
            (*b.lhs, *b.rhs)
        }
        _ => {dbg!("not a binexpr"); return None;},
    };
    func.ensures_clause.as_mut()?.exprs = vec![new_ens];
    let mut req = RequiresClause::new();
    req.exprs.push(new_req);
    func.requires_clause = Some(Box::new(req));
    Some(func.to_string())    
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::check_assist;

    #[test]
    fn test_split_imply_ensures_1() {
        check_assist(
          split_imply_ensures,
"
fn test_split_imply_ensures(b: bool) -> (ret: u32)
    ens$0ures 
      b ==> ret == 2
{
    let mut ret: u32 = 1;
    if b {
        ret = ret + 1;
    }
    ret
}  
",
"
fn test_split_imply_ensures(b: bool) -> (ret: u32)
    requires
        b,
    ensures
        ret == 2,
{
    let mut ret: u32 = 1;
    if b {
        ret = ret + 1;
    }
    ret
}
  
",

        )
    }
}

// let formatter = "/home/chanhee/.cargo/bin/rustfmt";
// let formatted_string = Command::new("echo")
//     .arg(string.clone())
//     .arg("|")
//     .arg(formatter)
//     .spawn()
//     .expect("echo command failed to start").stdout.unwrap();
// dbg!(formatted_string);
