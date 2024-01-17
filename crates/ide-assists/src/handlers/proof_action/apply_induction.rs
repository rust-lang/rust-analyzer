use crate::{AssistContext, Assists};
use ide_db::assists::{AssistId, AssistKind};
use itertools::Itertools;
use syntax::{ast::{self, vst::*}, AstNode,};


// TODO: maybe autogen decreases clause. 
// TODO: remove all panics -- verus-analyzer shouldn't panic on proof-action failure
pub(crate) fn apply_induction(acc: &mut Assists, ctx: &AssistContext<'_>) -> Option<()> {
    let func: ast::Fn = ctx.find_node_at_offset::<ast::Fn>()?;
    let body: ast::BlockExpr = func.body()?;
    let func: Fn = Fn::try_from(func).ok()?;
    let param_list = &(*func.param_list?).params;

    let param_names = param_list
        .iter()
        .map(|p| {
            let p = p.pat.as_ref().unwrap().as_ref();
            if let Pat::IdentPat(pat) = p {
                (&pat.as_ref().name.ident_token).as_ref().unwrap().clone()
            } else {
                panic!("not supported yet");
            }
        })
        .collect::<Vec<_>>();

    let index = param_list.iter().position(|p| {
        p.cst.as_ref().unwrap().syntax().text_range().contains_range(ctx.selection_trimmed())
    })?;

    let fn_name = func.name.to_string();

    let pty = param_list[index].ty.as_ref().unwrap();
    let mut result = String::new();

    // better way to type check?
    if pty.to_string().trim() == "nat" {
        result = apply_induction_on_nat(ctx, fn_name, param_names, index)?.to_string();
    } else {
        let p = param_list[index].pat.as_ref().unwrap().as_ref();
        // check_if_inductive_enum(ctx, p);
        if let Some(en) = ctx.type_of_pat_enum(p) {
            let bty = format!("Box<{}>", param_list[index].ty.as_ref().unwrap().to_string().trim());
            result = apply_induction_on_enum(ctx, fn_name, param_names, index, &en, bty)?.to_string();
        }
    }

    dbg!("{}", &result);
    let result = ctx.fmt(body.clone(),result.to_string())?;

    return acc.add(
        AssistId("apply_induction", AssistKind::RefactorRewrite),
        "Apply induction",
        body.syntax().text_range(),
        |edit| edit.replace(body.syntax().text_range(), result),
    );
}

fn apply_induction_on_nat(
    ctx: &AssistContext<'_>,
    fn_name: String,
    param_names: Vec<String>,
    index: usize,
) -> Option<BlockExpr> {
    let id = param_names[index].clone();
    let cond = ctx.vst_expr_from_text(format!("{} == 0", id.clone()).as_ref())?;  
    let sub = ctx.vst_expr_from_text(format!("{} - 1", id.clone()).as_ref())?;  

    // build arguments for recursive call
    let mut args = ArgList::new();
    param_names.iter().enumerate().for_each(|(i, name)| {
        if i == index {
            args.args.push(sub.clone());
        } else {
            args.args.push(Expr::from(Literal::new(name.clone())));
        }
    });

    // build if-else
    let then_branch = BlockExpr::new(StmtList::new());
    let mut ifexpr = IfExpr::new(cond, then_branch);
    let mut else_branch = BlockExpr::new(StmtList::new());
    let call_stmt: Stmt = CallExpr::new(Literal::new(fn_name), args).into();
    else_branch.stmt_list.statements.push(call_stmt);
    ifexpr.set_else_branch(ElseBranch::Block(Box::new(else_branch)));

    // return if-else as a block
    let mut stmtlist = StmtList::new();
    stmtlist.statements.push(ifexpr.into());
    let block = BlockExpr::new(stmtlist);
    Some(block)
}

fn apply_induction_on_enum(
    ctx: &AssistContext<'_>,
    fn_name: String,
    param_names: Vec<String>,
    index: usize,
    en: &Enum,
    bty: String,
) -> Option<BlockExpr> {
    let mut match_arms = vec![];
    for variant in &en.variant_list.variants {
        let fields = variant.field_list.as_ref();

        if fields == None {
            let arm = format!("{}::{} => {{}}", en.name, variant.name);
            match_arms.push(arm);
            continue;
        }

        match &**fields.unwrap() {
            FieldList::RecordFieldList(fields) => {
                let names =
                    fields.as_ref().fields.iter().map(|f| f.name.as_ref().to_string()).join(",");

                let calls = fields
                    .as_ref()
                    .fields
                    .iter()
                    .filter(|f| {
                        // TODO: this is awful
                        let fty = f.ty.as_ref().to_string().replace(" ", "");
                        fty == bty
                    })
                    .map(|f| {
                        let mut args = vec![];
                        param_names.iter().enumerate().for_each(|(i, name)| {
                            if i == index {
                                args.push(format!("*{}", f.name.as_ref().to_string()));
                            } else {
                                args.push(name.clone());
                            }
                        });
                        format!("{}({})", fn_name, args.join(","))
                    })
                    .collect::<Vec<_>>()
                    .join(";");
                let arm = format!("{}::{}{{{}}} => {{{};}}", en.name, variant.name, names, calls);
                match_arms.push(arm);
            }
            FieldList::TupleFieldList(_) => panic!("not supported yet"),
        }
    }
    let m = format!("match {} {{\n{}\n}}", param_names[index], match_arms.join(",\n"));
    let mut stmtlist = StmtList::new();
    stmtlist.statements.push(ctx.vst_expr_from_text(m.as_ref())?.into());
    let block = BlockExpr::new(stmtlist);
    Some(block)
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
{}
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
    if n == 0 {
    } else {
        sum_equal(n - 1, m);
    };
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
#[is_variant] #[derive(PartialEq, Eq)] 
enum Tree {
    Nil,
    Node { value: i64, left: Box<Tree>, right: Box<Tree> },
}

proof fn sorted_tree_means_sorted_sequence(tr$0ee: Tree)
    requires
        tree.is_sorted(),
    ensures
        sequence_is_sorted(tree@),
    decreases tree
{
}
"#,

            r#"
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
    decreases tree
{
    match tree {
        Tree::Nil => {},
        Tree::Node { value, left, right } => {
            sorted_tree_means_sorted_sequence(*left);
            sorted_tree_means_sorted_sequence(*right);
        },
    };
}

"#,
        );
    }
}
