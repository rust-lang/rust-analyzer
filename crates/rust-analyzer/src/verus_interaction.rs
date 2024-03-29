use syntax::{TextRange, TextSize};
use ide_assists::proof_plumber_api::verus_error::{AssertFailure, PostFailure, PreFailure, VerusError};

pub(crate) fn diagnostic_to_verus_err(
    diagnostic: &cargo_metadata::diagnostic::Diagnostic,
) -> Option<VerusError> {
    if diagnostic.message.contains("precondition not satisfied") {
        if diagnostic.spans.len() == 2 {
            let range0 = TextRange::new(
                TextSize::from(diagnostic.spans[0].byte_start),
                TextSize::from(diagnostic.spans[0].byte_end),
            );
            let range1 = TextRange::new(
                TextSize::from(diagnostic.spans[1].byte_start),
                TextSize::from(diagnostic.spans[1].byte_end),
            );
            let verr;
            if diagnostic.spans[0].is_primary {
                verr = VerusError::Pre(PreFailure { failing_pre: range1, callsite: range0 });
            } else {
                verr = VerusError::Pre(PreFailure { failing_pre: range0, callsite: range1 });
            }
            Some(verr)
        } else {
            // panic!("pre unexpected num of span");
            None
        }
    } else if diagnostic.message.contains("postcondition not satisfied") {
        if diagnostic.spans.len() == 2 {
            let range0 = TextRange::new(
                TextSize::from(diagnostic.spans[0].byte_start),
                TextSize::from(diagnostic.spans[0].byte_end),
            );
            let range1 = TextRange::new(
                TextSize::from(diagnostic.spans[1].byte_start),
                TextSize::from(diagnostic.spans[1].byte_end),
            );
            let verr;
            if diagnostic.spans[0].is_primary {
                verr =
                    VerusError::Post(PostFailure { failing_post: range1, func_body: range0 });
            } else {
                verr =
                    VerusError::Post(PostFailure { failing_post: range0, func_body: range1 });
            }
            Some(verr)
        } else {
            // panic!("post unexpected num of span");
            None
        }
    } else if diagnostic.message.contains("assertion failed") {
        // only reading first span now
        // dbg!(&diagnostic.spans);
        let range = TextRange::new(
            TextSize::from(diagnostic.spans[0].byte_start),
            TextSize::from(diagnostic.spans[0].byte_end),
        );
        let verr = VerusError::Assert(AssertFailure { range });
        Some(verr)
    } else {
        None
    }
}
