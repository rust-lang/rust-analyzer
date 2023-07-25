use text_edit::TextRange;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum VerusError {
    Pre(PreFailure),
    Post(PostFailure),
    Assert(AssertFailure),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PreFailure {
    pub failing_pre: TextRange,
    pub callsite: TextRange,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PostFailure {
    pub failing_post: TextRange,
    pub func_body: TextRange,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct AssertFailure {
    pub range: TextRange,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct VerusQuantifier {
    pub exprs: Vec<TextRange>,
}

pub fn filter_pre_failuires(verus_errors: &Vec<VerusError>) -> Vec<PreFailure> {
    let mut pre_errs = vec![];
    for verr in verus_errors {
        if let VerusError::Pre(p) = verr {
            pre_errs.push(p.clone());
        }
    }
    pre_errs
}

pub fn filter_post_failuires(verus_errors: &Vec<VerusError>) -> Vec<PostFailure> {
    let mut post_errs = vec![];
    for verr in verus_errors {
        if let VerusError::Post(p) = verr {
            post_errs.push(p.clone());
        }
    }
    post_errs
}
