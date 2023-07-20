use crate::{AssistContext, Assists};
use hir::{db, HasCrate, PathResolution};
use ide_db::{
    assists::{AssistId, AssistKind},
    syntax_helpers::node_ext::walk_expr,
    syntax_helpers::vst_ext::*,
};
use syntax::{
    ast::{self, vst::*},
    AstNode, SyntaxToken, T,
};

pub(crate) fn apply_induction(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let func: ast::Fn = ctx.find_node_at_offset::<ast::Fn>()?;
    let body: ast::BlockExpr = func.body()?;
    let func = Fn::try_from(func).ok()?;
    let param_list = &(*func.param_list?).params;

    // TODO: have a more standard way to find the nat param
    let index = param_list.iter().position(|p| {
        p.ty.as_ref().unwrap().to_string().trim() == "nat"
            && p.cst.as_ref().unwrap().syntax().text_range().contains_range(ctx.selection_trimmed())
    })?;

    let param_names = param_list.iter().map(|p| {
        let p = p.pat.as_ref().unwrap().as_ref();
        if let Pat::IdentPat(pat) = p {
            (&pat.as_ref().name.ident_token).as_ref().unwrap().clone()
        } else {
            panic!("not supported yet");
        }
    }
    ).collect::<Vec<_>>();

    let fn_name = func.name.to_string();

    let result = apply_induction_on_nat(ctx, fn_name, param_names, index)?;
    dbg!(&result);
    return acc.add(
        AssistId("apply_induction", AssistKind::RefactorRewrite),
        "Apply induction",
        body.syntax().text_range(),
        |edit| edit.replace(body.syntax().text_range(), result),
    );
}

fn apply_induction_on_nat(ctx: &AssistContext<'_>, fn_name: String, param_names: Vec<String>, index: usize) -> Option<String> {
    let id = param_names[index].clone();
    let cond = BinExpr::new(
        Literal::new(id.clone()),
        BinaryOp::CmpOp(ast::CmpOp::Eq { negated: false }),
        Literal::new(String::from("0")),
    );

    let mut args = ArgList::new();

    let sub = Expr::from(BinExpr::new(
        Literal::new(id.clone()),
        BinaryOp::ArithOp(ast::ArithOp::Sub),
        Literal::new(String::from("1")),
    ));

    param_names.iter().enumerate().for_each(|(i, name)| {
        if i == index {
            args.args.push(sub.clone());
        } else {
            args.args.push(Expr::from(Literal::new(name.clone())));
        }
    });

    let then_branch = BlockExpr::new(StmtList::new());
    let mut ifexpr = IfExpr::new(cond, then_branch);
    let mut else_branch = BlockExpr::new(StmtList::new());

    let call = ExprStmt::new(CallExpr::new(Literal::new(fn_name), args));
    else_branch.stmt_list.statements.push(Stmt::from(call));
    ifexpr.set_else_branch(ElseBranch::Block(Box::new(else_branch)));
    Some(ifexpr.to_string())
}

#[cfg(test)]
mod tests {
    use crate::tests::check_assist;

    use super::*;

    #[test]
    fn apply_induction_on_nat1() {
        check_assist(
            apply_induction,
            r#"
spec fn sum(n: nat) -> nat
{
    n * (n + 1) / 2
}

spec fn triangle(n: nat) -> nat
    decreases n,
{
    if n == 0 {
        0
    } else {
        n + triangle((n - 1) as nat)
    }
}

#[verifier(nonlinear)]
proof fn sum_equal($0n: nat, m: nat) 
    ensures sum(n) == triangle(n),
    decreases n,
{
}
"#,
            r#"
spec fn sum(n: nat) -> nat
{
    n * (n + 1) / 2
}

spec fn triangle(n: nat) -> nat
    decreases n,
{
    if n == 0 {
        0
    } else {
        n + triangle((n - 1) as nat)
    }
}

#[verifier(nonlinear)]
proof fn sum_equal(n: nat, m: nat)
    ensures sum(n) == triangle(n),
    decreases n,
{
    if n == 0 {}
    else {
        sum_equal(n - 1, m);
    }
}            
"#,
        );
    }

    #[test]
    // https://github.com/verus-lang/verus/blob/0088380265ed6e10c5d8034e89ce807a728f98e3/source/rust_verify/example/summer_school/chapter-1-22.rs
    fn apply_induction_on_enum1() {
        check_assist(
            apply_induction,
            r#"
use core::ops::Deref;

struct Box<T>(T);

impl<T> Deref for Box<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target;
}

struct Foo<T>(T);

impl<T> Foo<T> {
    fn get_inner<'a>(self: &'a Box<Self>) -> &'a T {}

    fn get_self<'a>(self: &'a Box<Self>) -> &'a Self {}

    fn into_inner(self: Box<Self>) -> Self {}
}
            
#[is_variant] #[derive(PartialEq, Eq)] 
enum Tree {
    Nil,
    Node { value: i64, left: Box<Tree>, right: Box<Tree> },
}

proof fn$0 sorted_tree_means_sorted_sequence(tree: Tree)
    requires
        tree.is_sorted(),
    ensures
        sequence_is_sorted(tree@),
{
}
"#,
            r#"
use core::ops::Deref;

struct Box<T>(T);

impl<T> Deref for Box<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target;
}

struct Foo<T>(T);

impl<T> Foo<T> {
    fn get_inner<'a>(self: &'a Box<Self>) -> &'a T {}

    fn get_self<'a>(self: &'a Box<Self>) -> &'a Self {}

    fn into_inner(self: Box<Self>) -> Self {}
}
            
#[is_variant] #[derive(PartialEq, Eq)] 
enum Tree {
    Nil,
    Node { value: i64, left: Box<Tree>, right: Box<Tree> },
}

proof fn sorted_tree_means_sorted_sequence(tree: Tree)
    requires
        tree.is_sorted(),
    ensures
        sequence_is_sorted(tree@),
    decreases tree // guessed by Dafny
{
    if let Tree::Node { left, right, value: _ } = tree {
        sorted_tree_means_sorted_sequence(*left); // guessed by Dafny
        sorted_tree_means_sorted_sequence(*right); // guessed by Dafny
    }
}    
"#,
        );
    }
}
