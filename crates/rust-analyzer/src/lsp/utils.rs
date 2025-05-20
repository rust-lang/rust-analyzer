//! Utilities for LSP-related boilerplate code.
use std::{mem, ops::Range, panic};

use ide_db::{base_db::DbPanicContext, source_change::SourceChangeBuilder};
use lsp_server::{Notification, RequestId, Response, ResponseError};
use lsp_types::{
    ShowMessageRequestParams,
    request::{Request, ShowMessageRequest},
};
use stdx::thread::ThreadIntent;
use triomphe::Arc;

use crate::{
    global_state::GlobalState,
    line_index::{LineEndings, LineIndex, PositionEncoding},
    lsp::{LspError, from_proto, to_proto},
    lsp_ext,
    main_loop::Task,
};

pub(crate) fn invalid_params_error(message: String) -> LspError {
    LspError { code: lsp_server::ErrorCode::InvalidParams as i32, message }
}

pub(crate) fn notification_is<N: lsp_types::notification::Notification>(
    notification: &Notification,
) -> bool {
    notification.method == N::METHOD
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Progress {
    Begin,
    Report,
    End,
}

impl Progress {
    pub(crate) fn fraction(done: usize, total: usize) -> f64 {
        assert!(done <= total);
        done as f64 / total.max(1) as f64
    }
}

impl GlobalState {
    pub(crate) fn show_message(
        &mut self,
        typ: lsp_types::MessageType,
        message: String,
        show_open_log_button: bool,
    ) {
        match self.config.open_server_logs() && show_open_log_button  {
            true => self.send_request::<lsp_types::request::ShowMessageRequest>(
                lsp_types::ShowMessageRequestParams {
                    typ,
                    message,
                    actions: Some(vec![lsp_types::MessageActionItem {
                        title: "Open server logs".to_owned(),
                        properties: Default::default(),
                    }]),
                },
                |this, resp| {
                    let lsp_server::Response { error: None, result: Some(result), .. } = resp
                    else { return };
                    if let Ok(Some(_item)) = crate::from_json::<
                        <lsp_types::request::ShowMessageRequest as lsp_types::request::Request>::Result,
                    >(
                        lsp_types::request::ShowMessageRequest::METHOD, &result
                    ) {
                        this.send_notification::<lsp_ext::OpenServerLogs>(());
                    }
                },
            ),
            false => self.send_notification::<lsp_types::notification::ShowMessage>(
                lsp_types::ShowMessageParams {
                    typ,
                    message,
                },
            ),
        }
    }

    /// If `additional_info` is [`Some`], appends a note to the notification telling to check the logs.
    /// This will always log `message` + `additional_info` to the server's error log.
    pub(crate) fn show_and_log_error(&mut self, message: String, additional_info: Option<String>) {
        match additional_info {
            Some(additional_info) => {
                tracing::error!("{message}:\n{additional_info}");
                self.show_message(
                    lsp_types::MessageType::ERROR,
                    message,
                    tracing::enabled!(tracing::Level::ERROR),
                );
            }
            None => {
                tracing::error!("{message}");
                self.send_notification::<lsp_types::notification::ShowMessage>(
                    lsp_types::ShowMessageParams { typ: lsp_types::MessageType::ERROR, message },
                );
            }
        }
    }

    /// Ask for user choice by sending ShowMessageRequest
    pub(crate) fn ask_for_choice(&mut self) {
        let params = {
            let mut handler = self.user_choice_handler.lock();
            if handler.is_awaiting() {
                // already sent a request, do nothing
                return;
            }
            let mut is_done_asking = false;
            let params = if let Some(choice_group) = handler.first_mut_choice_group() {
                if let Some((_idx, choice)) = choice_group.get_cur_question() {
                    Some(ShowMessageRequestParams {
                        typ: lsp_types::MessageType::INFO,
                        message: choice.title.clone(),
                        actions: Some(
                            choice
                                .actions
                                .clone()
                                .into_iter()
                                .map(|action| lsp_types::MessageActionItem {
                                    title: action,
                                    properties: Default::default(),
                                })
                                .collect(),
                        ),
                    })
                } else {
                    is_done_asking = choice_group.is_done_asking();
                    None
                }
            } else {
                None
            };

            if is_done_asking {
                let Some(choice_group) = handler.pop_choice_group() else {
                    return;
                };
                let snap = self.snapshot();
                // TODO: handle finished choice
                // spawn a new task to handle the finished choice, in case of panic
                self.task_pool.handle.spawn(ThreadIntent::Worker, move || {
                    let result = panic::catch_unwind(move || {
                        let _pctx = DbPanicContext::enter("ask_for_choice".to_string());
                        let mut source_change_builder =
                            SourceChangeBuilder::new(choice_group.file_id());
                        choice_group.finish(&mut source_change_builder);
                        let source_change = source_change_builder.finish();
                        to_proto::workspace_edit(&snap, source_change)
                    });

                    // it's either this or die horribly
                    let empty_req_id = RequestId::from("".to_string());
                    match result {
                        Ok(Ok(result)) => Task::Response(Response::new_ok(empty_req_id, result)),
                        Ok(Err(_cancelled)) => Task::Response(Response {
                            id: empty_req_id,
                            result: None,
                            error: Some(ResponseError {
                                code: lsp_server::ErrorCode::ContentModified as i32,
                                message: "content modified".to_owned(),
                                data: None,
                            }),
                        }),
                        Err(panic) => {
                            let panic_message = panic
                                .downcast_ref::<String>()
                                .map(String::as_str)
                                .or_else(|| panic.downcast_ref::<&str>().copied());

                            let mut message = "request handler panicked".to_owned();
                            if let Some(panic_message) = panic_message {
                                message.push_str(": ");
                                message.push_str(panic_message)
                            } else if let Ok(_cancelled) =
                                panic.downcast::<ide_db::base_db::salsa::Cancelled>()
                            {
                                tracing::error!(
                                    "Cancellation propagated out of salsa! This is a bug"
                                );
                            }
                            Task::Response(Response::new_err(
                                empty_req_id,
                                lsp_server::ErrorCode::InternalError as i32,
                                message,
                            ))
                        }
                    }
                });
            }

            if params.is_some() {
                handler.set_awaiting(true);
            }
            params
        };

        // send ShowMessageRequest to the client, and handle the response
        if let Some(params) = params {
            self.send_request::<ShowMessageRequest>(params, |state, response| {
                let lsp_server::Response { error: None, result: Some(result), .. } = response
                else {
                    return;
                };
                let choice = match crate::from_json::<
                    <lsp_types::request::ShowMessageRequest as lsp_types::request::Request>::Result,
                >(
                    lsp_types::request::ShowMessageRequest::METHOD, &result
                ) {
                    Ok(Some(item)) => Some(item.title.clone()),
                    Err(err) => {
                        tracing::error!("Failed to deserialize ShowMessageRequest result: {err}");
                        None
                    }
                    // user made no choice
                    Ok(None) => None,
                };
                let mut do_pop = false;
                let mut handler = state.user_choice_handler.lock();
                match (handler.first_mut_choice_group(), choice) {
                    (Some(choice_group), Some(choice)) => {
                        let Some((question_idx, user_choices)) = choice_group.get_cur_question()
                        else {
                            tracing::error!("No question found for user choice");
                            return;
                        };
                        let choice_idx = user_choices
                            .actions
                            .iter()
                            .position(|it| *it == choice)
                            .unwrap_or(user_choices.actions.len());
                        if let Err(err) = choice_group.make_choice(question_idx, choice_idx) {
                            tracing::error!("Failed to make choice: {err}");
                        }
                    }
                    (None, Some(choice)) => {
                        tracing::error!("No ongoing choice group found for user choice: {choice}");
                    }
                    (Some(_), None) => {
                        // user made no choice, pop&drop current choice group
                        do_pop = true;
                    }
                    _ => (),
                }

                if do_pop {
                    let group = handler.pop_choice_group();
                    tracing::error!(
                        "User made no choice, dropping current choice group: {group:?}"
                    );
                }
                handler.set_awaiting(false);
                drop(handler);

                // recursively call handle_choice to handle the next question
                state.ask_for_choice();
            });
        }
    }

    /// rust-analyzer is resilient -- if it fails, this doesn't usually affect
    /// the user experience. Part of that is that we deliberately hide panics
    /// from the user.
    ///
    /// We do however want to pester rust-analyzer developers with panics and
    /// other "you really gotta fix that" messages. The current strategy is to
    /// be noisy for "from source" builds or when profiling is enabled.
    ///
    /// It's unclear if making from source `cargo xtask install` builds more
    /// panicky is a good idea, let's see if we can keep our awesome bleeding
    /// edge users from being upset!
    pub(crate) fn poke_rust_analyzer_developer(&mut self, message: String) {
        let from_source_build = option_env!("POKE_RA_DEVS").is_some();
        let profiling_enabled = std::env::var("RA_PROFILE").is_ok();
        if from_source_build || profiling_enabled {
            self.show_and_log_error(message, None);
        }
    }

    pub(crate) fn report_progress(
        &mut self,
        title: &str,
        state: Progress,
        message: Option<String>,
        fraction: Option<f64>,
        cancel_token: Option<String>,
    ) {
        if !self.config.work_done_progress() {
            return;
        }
        let percentage = fraction.map(|f| {
            assert!((0.0..=1.0).contains(&f));
            (f * 100.0) as u32
        });
        let cancellable = Some(cancel_token.is_some());
        let token = lsp_types::ProgressToken::String(
            cancel_token.unwrap_or_else(|| format!("rustAnalyzer/{title}")),
        );
        tracing::debug!(?token, ?state, "report_progress {message:?}");
        let work_done_progress = match state {
            Progress::Begin => {
                self.send_request::<lsp_types::request::WorkDoneProgressCreate>(
                    lsp_types::WorkDoneProgressCreateParams { token: token.clone() },
                    |_, _| (),
                );

                lsp_types::WorkDoneProgress::Begin(lsp_types::WorkDoneProgressBegin {
                    title: title.into(),
                    cancellable,
                    message,
                    percentage,
                })
            }
            Progress::Report => {
                lsp_types::WorkDoneProgress::Report(lsp_types::WorkDoneProgressReport {
                    cancellable,
                    message,
                    percentage,
                })
            }
            Progress::End => {
                lsp_types::WorkDoneProgress::End(lsp_types::WorkDoneProgressEnd { message })
            }
        };
        self.send_notification::<lsp_types::notification::Progress>(lsp_types::ProgressParams {
            token,
            value: lsp_types::ProgressParamsValue::WorkDone(work_done_progress),
        });
    }
}

pub(crate) fn apply_document_changes(
    encoding: PositionEncoding,
    file_contents: &str,
    mut content_changes: Vec<lsp_types::TextDocumentContentChangeEvent>,
) -> String {
    // If at least one of the changes is a full document change, use the last
    // of them as the starting point and ignore all previous changes.
    let (mut text, content_changes) =
        match content_changes.iter().rposition(|change| change.range.is_none()) {
            Some(idx) => {
                let text = mem::take(&mut content_changes[idx].text);
                (text, &content_changes[idx + 1..])
            }
            None => (file_contents.to_owned(), &content_changes[..]),
        };
    if content_changes.is_empty() {
        return text;
    }

    let mut line_index = LineIndex {
        // the index will be overwritten in the bottom loop's first iteration
        index: Arc::new(ide::LineIndex::new(&text)),
        // We don't care about line endings here.
        endings: LineEndings::Unix,
        encoding,
    };

    // The changes we got must be applied sequentially, but can cross lines so we
    // have to keep our line index updated.
    // Some clients (e.g. Code) sort the ranges in reverse. As an optimization, we
    // remember the last valid line in the index and only rebuild it if needed.
    // The VFS will normalize the end of lines to `\n`.
    let mut index_valid = !0u32;
    for change in content_changes {
        // The None case can't happen as we have handled it above already
        if let Some(range) = change.range {
            if index_valid <= range.end.line {
                *Arc::make_mut(&mut line_index.index) = ide::LineIndex::new(&text);
            }
            index_valid = range.start.line;
            if let Ok(range) = from_proto::text_range(&line_index, range) {
                text.replace_range(Range::<usize>::from(range), &change.text);
            }
        }
    }
    text
}

/// Checks that the edits inside the completion and the additional edits do not overlap.
/// LSP explicitly forbids the additional edits to overlap both with the main edit and themselves.
pub(crate) fn all_edits_are_disjoint(
    completion: &lsp_types::CompletionItem,
    additional_edits: &[lsp_types::TextEdit],
) -> bool {
    let mut edit_ranges = Vec::new();
    match completion.text_edit.as_ref() {
        Some(lsp_types::CompletionTextEdit::Edit(edit)) => {
            edit_ranges.push(edit.range);
        }
        Some(lsp_types::CompletionTextEdit::InsertAndReplace(edit)) => {
            let replace = edit.replace;
            let insert = edit.insert;
            if replace.start != insert.start
                || insert.start > insert.end
                || insert.end > replace.end
            {
                // insert has to be a prefix of replace but it is not
                return false;
            }
            edit_ranges.push(replace);
        }
        None => {}
    }
    if let Some(additional_changes) = completion.additional_text_edits.as_ref() {
        edit_ranges.extend(additional_changes.iter().map(|edit| edit.range));
    };
    edit_ranges.extend(additional_edits.iter().map(|edit| edit.range));
    edit_ranges.sort_by_key(|range| (range.start, range.end));
    edit_ranges
        .iter()
        .zip(edit_ranges.iter().skip(1))
        .all(|(previous, next)| previous.end <= next.start)
}

#[cfg(test)]
mod tests {
    use ide_db::line_index::WideEncoding;
    use lsp_types::{
        CompletionItem, CompletionTextEdit, InsertReplaceEdit, Position, Range,
        TextDocumentContentChangeEvent,
    };

    use super::*;

    #[test]
    fn test_apply_document_changes() {
        macro_rules! c {
            [$($sl:expr, $sc:expr; $el:expr, $ec:expr => $text:expr),+] => {
                vec![$(TextDocumentContentChangeEvent {
                    range: Some(Range {
                        start: Position { line: $sl, character: $sc },
                        end: Position { line: $el, character: $ec },
                    }),
                    range_length: None,
                    text: String::from($text),
                }),+]
            };
        }

        let encoding = PositionEncoding::Wide(WideEncoding::Utf16);
        let text = apply_document_changes(encoding, "", vec![]);
        assert_eq!(text, "");
        let text = apply_document_changes(
            encoding,
            &text,
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: String::from("the"),
            }],
        );
        assert_eq!(text, "the");
        let text = apply_document_changes(encoding, &text, c![0, 3; 0, 3 => " quick"]);
        assert_eq!(text, "the quick");
        let text =
            apply_document_changes(encoding, &text, c![0, 0; 0, 4 => "", 0, 5; 0, 5 => " foxes"]);
        assert_eq!(text, "quick foxes");
        let text = apply_document_changes(encoding, &text, c![0, 11; 0, 11 => "\ndream"]);
        assert_eq!(text, "quick foxes\ndream");
        let text = apply_document_changes(encoding, &text, c![1, 0; 1, 0 => "have "]);
        assert_eq!(text, "quick foxes\nhave dream");
        let text = apply_document_changes(
            encoding,
            &text,
            c![0, 0; 0, 0 => "the ", 1, 4; 1, 4 => " quiet", 1, 16; 1, 16 => "s\n"],
        );
        assert_eq!(text, "the quick foxes\nhave quiet dreams\n");
        let text =
            apply_document_changes(encoding, &text, c![0, 15; 0, 15 => "\n", 2, 17; 2, 17 => "\n"]);
        assert_eq!(text, "the quick foxes\n\nhave quiet dreams\n\n");
        let text = apply_document_changes(
            encoding,
            &text,
            c![1, 0; 1, 0 => "DREAM", 2, 0; 2, 0 => "they ", 3, 0; 3, 0 => "DON'T THEY?"],
        );
        assert_eq!(text, "the quick foxes\nDREAM\nthey have quiet dreams\nDON'T THEY?\n");
        let text =
            apply_document_changes(encoding, &text, c![0, 10; 1, 5 => "", 2, 0; 2, 12 => ""]);
        assert_eq!(text, "the quick \nthey have quiet dreams\n");

        let text = String::from("❤️");
        let text = apply_document_changes(encoding, &text, c![0, 0; 0, 0 => "a"]);
        assert_eq!(text, "a❤️");

        let text = String::from("a\nb");
        let text =
            apply_document_changes(encoding, &text, c![0, 1; 1, 0 => "\nțc", 0, 1; 1, 1 => "d"]);
        assert_eq!(text, "adcb");

        let text = String::from("a\nb");
        let text =
            apply_document_changes(encoding, &text, c![0, 1; 1, 0 => "ț\nc", 0, 2; 0, 2 => "c"]);
        assert_eq!(text, "ațc\ncb");
    }

    #[test]
    fn empty_completion_disjoint_tests() {
        let empty_completion = CompletionItem::new_simple("label".to_owned(), "detail".to_owned());

        let disjoint_edit_1 = lsp_types::TextEdit::new(
            Range::new(Position::new(2, 2), Position::new(3, 3)),
            "new_text".to_owned(),
        );
        let disjoint_edit_2 = lsp_types::TextEdit::new(
            Range::new(Position::new(3, 3), Position::new(4, 4)),
            "new_text".to_owned(),
        );

        let joint_edit = lsp_types::TextEdit::new(
            Range::new(Position::new(1, 1), Position::new(5, 5)),
            "new_text".to_owned(),
        );

        assert!(
            all_edits_are_disjoint(&empty_completion, &[]),
            "Empty completion has all its edits disjoint"
        );
        assert!(
            all_edits_are_disjoint(
                &empty_completion,
                &[disjoint_edit_1.clone(), disjoint_edit_2.clone()]
            ),
            "Empty completion is disjoint to whatever disjoint extra edits added"
        );

        assert!(
            !all_edits_are_disjoint(
                &empty_completion,
                &[disjoint_edit_1, disjoint_edit_2, joint_edit]
            ),
            "Empty completion does not prevent joint extra edits from failing the validation"
        );
    }

    #[test]
    fn completion_with_joint_edits_disjoint_tests() {
        let disjoint_edit = lsp_types::TextEdit::new(
            Range::new(Position::new(1, 1), Position::new(2, 2)),
            "new_text".to_owned(),
        );
        let disjoint_edit_2 = lsp_types::TextEdit::new(
            Range::new(Position::new(2, 2), Position::new(3, 3)),
            "new_text".to_owned(),
        );
        let joint_edit = lsp_types::TextEdit::new(
            Range::new(Position::new(1, 1), Position::new(5, 5)),
            "new_text".to_owned(),
        );

        let mut completion_with_joint_edits =
            CompletionItem::new_simple("label".to_owned(), "detail".to_owned());
        completion_with_joint_edits.additional_text_edits =
            Some(vec![disjoint_edit.clone(), joint_edit.clone()]);
        assert!(
            !all_edits_are_disjoint(&completion_with_joint_edits, &[]),
            "Completion with disjoint edits fails the validation even with empty extra edits"
        );

        completion_with_joint_edits.text_edit =
            Some(CompletionTextEdit::Edit(disjoint_edit.clone()));
        completion_with_joint_edits.additional_text_edits = Some(vec![joint_edit.clone()]);
        assert!(
            !all_edits_are_disjoint(&completion_with_joint_edits, &[]),
            "Completion with disjoint edits fails the validation even with empty extra edits"
        );

        completion_with_joint_edits.text_edit =
            Some(CompletionTextEdit::InsertAndReplace(InsertReplaceEdit {
                new_text: "new_text".to_owned(),
                insert: disjoint_edit.range,
                replace: disjoint_edit_2.range,
            }));
        completion_with_joint_edits.additional_text_edits = Some(vec![joint_edit]);
        assert!(
            !all_edits_are_disjoint(&completion_with_joint_edits, &[]),
            "Completion with disjoint edits fails the validation even with empty extra edits"
        );
    }

    #[test]
    fn completion_with_disjoint_edits_disjoint_tests() {
        let disjoint_edit = lsp_types::TextEdit::new(
            Range::new(Position::new(1, 1), Position::new(2, 2)),
            "new_text".to_owned(),
        );
        let disjoint_edit_2 = lsp_types::TextEdit::new(
            Range::new(Position::new(2, 2), Position::new(3, 3)),
            "new_text".to_owned(),
        );
        let joint_edit = lsp_types::TextEdit::new(
            Range::new(Position::new(1, 1), Position::new(5, 5)),
            "new_text".to_owned(),
        );

        let mut completion_with_disjoint_edits =
            CompletionItem::new_simple("label".to_owned(), "detail".to_owned());
        completion_with_disjoint_edits.text_edit = Some(CompletionTextEdit::Edit(disjoint_edit));
        let completion_with_disjoint_edits = completion_with_disjoint_edits;

        assert!(
            all_edits_are_disjoint(&completion_with_disjoint_edits, &[]),
            "Completion with disjoint edits is valid"
        );
        assert!(
            !all_edits_are_disjoint(&completion_with_disjoint_edits, &[joint_edit]),
            "Completion with disjoint edits and joint extra edit is invalid"
        );
        assert!(
            all_edits_are_disjoint(&completion_with_disjoint_edits, &[disjoint_edit_2]),
            "Completion with disjoint edits and joint extra edit is valid"
        );
    }
}
