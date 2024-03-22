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
    if func.signature_decreases.is_none() {
        dbg!("no decreases vst");
        return None;
    }


    let mut new_fn = func.clone();
    let param_list = &(*func.param_list?).params;

    let param_names: Option<Vec<String>> = param_list
        .iter()
        .map(|p| {
            let p = p.pat.clone()?;
            if let Pat::IdentPat(pat) = *p {
                pat.name.ident_token
            } else {
                None
            }
        })
        .collect();
    let param_names = param_names?;

    // get the selected argument
    let index = param_list.iter().position(|p| {
        p.cst.as_ref().unwrap().syntax().text_range().contains_range(ctx.selection_trimmed())
    })?;

    let fn_name = func.name.to_string();

    let pty = param_list[index].ty.as_ref().unwrap();
    let mut result = BlockExpr::new(StmtList::new());

    // better way to type check?
    if pty.to_string().trim() == "nat" {
        result = apply_induction_on_nat(ctx, fn_name, param_names, index)?;
    } else {
        let p = param_list[index].pat.as_ref().unwrap().as_ref();
        if let Some(en) = ctx.type_of_pat_enum(p) {
            let bty = format!("Box<{}>", param_list[index].ty.as_ref().unwrap().to_string().trim());
            result = apply_induction_on_enum(ctx, fn_name, param_names, index, &en, bty)?;
        }
    }

    // now check if proof now goes thorugh, and make sure it is fast
    new_fn.body = Some(Box::new(result.clone()));
    let verif_result = ctx.try_verus(&new_fn)?;
    if !verif_result.is_success {
        return None;
    } else {
        if verif_result.time > 10 {
            return None;
        }
    }
    

    dbg!("{}", &result.to_string());
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
    let sub = ctx.vst_expr_from_text(format!("({} - 1) as nat", id.clone()).as_ref())?;  

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
use vstd::prelude::*;

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

proof fn sum_equal($0n: nat, m: nat) by(nonlinear_arith)
    ensures sum(n) == triangle(n),
    decreases n,
{}

fn main() {}
"#,
            r#"
use vstd::prelude::*;

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

proof fn sum_equal(n: nat, m: nat) by(nonlinear_arith)
    ensures sum(n) == triangle(n),
    decreases n,
{
    if n == 0 {
    } else {
        sum_equal((n - 1) as nat, m);
    };
}


fn main() {}
"#,
        );
    }


    #[test]
    // from https://github.com/verus-lang/verus/blob/0088380265ed6e10c5d8034e89ce807a728f98e3/source/rust_verify/example/summer_school/chapter-1-22.rs
    fn apply_induction_on_enum1() {
        check_assist(
            apply_induction,
// before
            r#"
use vstd::prelude::*;

spec fn sequence_is_sorted(s: Seq<int>) -> bool {
    forall|i: int, j: int| 0 <= i < j < s.len() ==> s[i] <= s[j]
}

spec fn sequences_ordered_at_interface(seq1: Seq<int>, seq2: Seq<int>) -> bool {
    if seq1.len() == 0 || seq2.len() == 0 {
        true
    } else {
        seq1.last() <= seq2[0]
    }
}

#[is_variant] #[derive(PartialEq, Eq)] 
enum Tree {
    Nil,
    Node { value: i64, left: Box<Tree>, right: Box<Tree> },
}
impl Tree {
    spec fn view(&self) -> Seq<int>
        decreases self
    {
        match *self {
            Tree::Nil => seq![],
            Tree::Node { value, left, right } => left@.add(seq![value as int]).add(right@),
        }
    }

    spec fn is_sorted(&self) -> bool
        decreases self
    {
        match *self {
            Tree::Nil => true,
            Tree::Node { value, left, right } => {
                &&& sequences_ordered_at_interface(left@, seq![value as int])
                &&& sequences_ordered_at_interface(seq![value as int], right@)
                &&& left.is_sorted()
                &&& right.is_sorted()
            }
        }
    }
}


proof fn sorted_tree_means_sorted_sequence(tr$0ee: Tree)
    requires
        tree.is_sorted(),
    ensures
        sequence_is_sorted(tree@),
    decreases tree
{
}

fn main() {}
"#,

// after
            r#"
use vstd::prelude::*;

spec fn sequence_is_sorted(s: Seq<int>) -> bool {
    forall|i: int, j: int| 0 <= i < j < s.len() ==> s[i] <= s[j]
}

spec fn sequences_ordered_at_interface(seq1: Seq<int>, seq2: Seq<int>) -> bool {
    if seq1.len() == 0 || seq2.len() == 0 {
        true
    } else {
        seq1.last() <= seq2[0]
    }
}

#[is_variant] #[derive(PartialEq, Eq)] 
enum Tree {
    Nil,
    Node { value: i64, left: Box<Tree>, right: Box<Tree> },
}
impl Tree {
    spec fn view(&self) -> Seq<int>
        decreases self
    {
        match *self {
            Tree::Nil => seq![],
            Tree::Node { value, left, right } => left@.add(seq![value as int]).add(right@),
        }
    }

    spec fn is_sorted(&self) -> bool
        decreases self
    {
        match *self {
            Tree::Nil => true,
            Tree::Node { value, left, right } => {
                &&& sequences_ordered_at_interface(left@, seq![value as int])
                &&& sequences_ordered_at_interface(seq![value as int], right@)
                &&& left.is_sorted()
                &&& right.is_sorted()
            }
        }
    }
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


fn main() {}
"#,
        );
    }
}
