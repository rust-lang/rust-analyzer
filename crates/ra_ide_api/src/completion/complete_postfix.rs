use crate::{
    completion::{
        completion_item::{
            Completions,
            Builder,
            CompletionKind,
            InsertTextFormat
        },
        completion_context::CompletionContext,
    },
    CompletionItem
};
use ra_syntax::{
    ast::AstNode,
    TextRange
};
use ra_text_edit::{
    TextEditBuilder,
    AtomTextEdit
};

fn postfix_snippet<'a>(
    ctx: &'a CompletionContext<'a>,
    range: TextRange,
    label: &str,
    snippet: &str,
) -> Builder<'a> {
    let text_edit = AtomTextEdit::insert(ctx.offset, snippet.to_string());
    let mut builder = TextEditBuilder::default();
    builder.delete(range);
    CompletionItem::new(CompletionKind::Postfix, ctx, label)
        .text_edit(text_edit)
        .insert_text_format(InsertTextFormat::Snippet)
        .additional_text_edits(builder.finish())
}

pub(super) fn complete_postfix(acc: &mut Completions, ctx: &CompletionContext) {
    if let Some(dot_receiver) = ctx.dot_receiver {
        let receiver_text = dot_receiver.syntax().text().to_string();
        let receiver_range = dot_receiver.syntax().range();
        let range = TextRange::from_to(receiver_range.start(), ctx.offset);
        postfix_snippet(ctx, range, "not", "!not").add_to(acc);
        postfix_snippet(ctx, range, "if", &format!("if {} {{$0}}", receiver_text)).add_to(acc);
        postfix_snippet(
            ctx,
            range,
            "match",
            &format!("match {} {{\n${{1:_}} => {{$0\\}},\n}}", receiver_text),
        )
            .add_to(acc);
        postfix_snippet(
            ctx,
            range,
            "while",
            &format!("while {} {{\n$0\n}}", receiver_text),
        )
            .add_to(acc);
    }
}

#[cfg(test)]
mod tests {
    use crate::completion::completion_item::CompletionKind;
    use crate::completion::completion_item::check_completion;

    fn check_snippet_completion(code: &str, expected_completions: &str) {
        check_completion(code, expected_completions, CompletionKind::Postfix);
    }

    #[test]
    fn test_complete_postfix() {
        check_snippet_completion(
            "completion_postfix",
            r#"
            fn main() {
                let bar = "a";
                bar.if<|>
            }
            "#,
        );
    }
}
