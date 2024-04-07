use crate::{
    assist_context::{AssistContext, Assists},
    AssistId,
    AssistKind,
};
use syntax::ast::{self, vst::*, AstNode, LogicOp};

pub(crate) fn seq_index_inbound(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    // if the function name is not inside an assertForallExpr, return None
    let assert_forall_expr: ast::AssertForallExpr = ctx.find_node_at_offset()?;
    // trigger on the seq variable
    let seq_path : ast::PathExpr = ctx.find_node_at_offset()?; 

    // now convert to vst nodes
    let v_assert_forall_expr = AssertForallExpr::try_from(assert_forall_expr.clone()).ok()?;
    // dbg!("{}", &v_assert_forall_expr.to_string());
    let v_seq_path = PathExpr::try_from(seq_path.clone()).ok()?;

    let result = vst_rewriter_seq_index_inbound(ctx, v_assert_forall_expr.clone(), v_seq_path.clone())?; // TODO: verusfmt
    let result = ctx.fmt(assert_forall_expr.clone(),result.to_string())?;

    acc.add(
        AssistId("seq_index_inbound", AssistKind::RefactorRewrite),
        "Insert In-bound predicate for selected seq",
        assert_forall_expr.syntax().text_range(),
        |edit| {
            edit.insert(assert_forall_expr.syntax().text_range().start(), format!("{}\n    ",result));
        },
    )
}

pub(crate) fn vst_rewriter_seq_index_inbound(
    ctx: &AssistContext<'_>, 
    mut assert_forall: AssertForallExpr, 
    seq_path: PathExpr,
) -> Option<AssertForallExpr> {
    let assert_forall_cp = assert_forall.clone();
    // if assertion's expression's top level is not implication, return None
    if assert_forall.implies_token {
        return None; // already with assumption
    }
    assert_forall.implies_token = true;

    // assume seq for now
    // let struck = ctx.type_of_expr_struct(&seq_path.clone().into())?;
    // dbg!(&struck);
    // if struck.name.to_string().trim() != "Seq" {
    //   return None;
    // }

    let quantified_variable = assert_forall.closure_expr.param_list.as_ref()?.params.first()?.pat.as_ref()?;

    // now add in bound predicate as assumption
    // e,g., 0 <= i < s2.len()
    // "s2"
    let seq_as_expr:Expr = seq_path.clone().into();
    // "s2.len()"
    let method_call = MethodCallExpr::new(seq_as_expr, ctx.vst_nameref_from_text("len")?, ArgList::new());
    // 0 <= i
    let first_binexpr = BinExpr::new(
      Literal::new(String::from("0")),
      BinaryOp::CmpOp(ast::CmpOp::Ord { ordering: ast::Ordering::Less, strict: false }),
      Literal::new(quantified_variable.to_string().trim().to_owned()),
    );
    // i < s2.len()
    let second_binexpr = BinExpr::new(
      Literal::new(quantified_variable.to_string().trim().to_owned()),
      BinaryOp::CmpOp(ast::CmpOp::Ord { ordering: ast::Ordering::Less, strict: true }),
      method_call,
    );
    //  0 <= i < s2.len()
    let binexpr = BinExpr::new(
      first_binexpr,
      BinaryOp::LogicOp(LogicOp::And),
      second_binexpr,
    );
    assert_forall.expr = Some(assert_forall_cp.closure_expr.body);
    assert_forall.closure_expr.body = Box::new(binexpr.into());
    
    Some(assert_forall) 
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::check_assist;

    #[test]
    fn test_seq_index_inbound1() {
        check_assist(
          seq_index_inbound,
          
"
use vstd::seq::*;
use vstd::seq_lib::*;
proof fn test_seq_in_bound() {
    let s1 = Seq::new(6, |i: int|
      if i == 0 { 10 }
      else if i == 1 { 20 }
      else if i == 2 { 30 }
      else if i == 3 { 45 }
      else if i == 4 { 55 }
      else { 70 }
    );
    let s2 = s1.filter(|x: int| x < 40);
    assert forall|i: int| $0s2[i] < 40 by{};
}
",
"
use vstd::seq::*;
use vstd::seq_lib::*;
proof fn test_seq_in_bound() {
    let s1 = Seq::new(6, |i: int|
      if i == 0 { 10 }
      else if i == 1 { 20 }
      else if i == 2 { 30 }
      else if i == 3 { 45 }
      else if i == 4 { 55 }
      else { 70 }
    );
    let s2 = s1.filter(|x: int| x < 40);
    assert forall|i: int| 0 <= i && i < s2.len() implies s2[i] < 40 by {}
    assert forall|i: int| s2[i] < 40 by{};
}
",

        )
    }
}

