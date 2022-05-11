//! See [`patch_json_for_outdated_configs`]
use serde_json::{json, Value};

/// This function patches the json config to the new expected keys.
/// That is we try to load old known config keys here and convert them to the new ones.
/// See https://github.com/rust-lang/rust-analyzer/pull/12010
pub(super) fn patch_json_for_outdated_configs(json: &mut Value) {
    let copy = json.clone();

    macro_rules! patch {
        ($(
            $($src:ident).+ -> $($dst:ident).+ ;
        )+) => { $(
            match copy.pointer(concat!($("/", stringify!($src)),+)).cloned() {
                Some(Value::Object(_)) | None => (),
                Some(it) => {
                    let mut last = it;
                    for segment in [$(stringify!($dst)),+].into_iter().rev() {
                        last = Value::Object(serde_json::Map::from_iter(std::iter::once((segment.to_string(), last))));
                    }

                    merge(json, last);
                },
            }
        )+ };
    }

    patch! {
        assist.allowMergingIntoGlobImports -> imports.merge.glob;
        assist.exprFillDefault -> assist.expressionFillDefault;
        assist.importEnforceGranularity -> imports.granularity.enforce;
        assist.importGranularity -> imports.granularity.group;
        assist.importMergeBehavior -> imports.granularity.group;
        assist.importMergeBehaviour -> imports.granularity.group;
        assist.importGroup -> imports.group.enabled;
        assist.importPrefix -> imports.prefix;
        cache.warmup -> cachePriming.enabled;
        cargo.loadOutDirsFromCheck -> cargo.buildScripts.enabled;
        cargo.runBuildScripts -> cargo.buildScripts.enabled;
        cargo.runBuildScriptsCommand -> cargo.buildScripts.overrideCommand;
        cargo.useRustcWrapperForBuildScripts -> cargo.buildScripts.useRustcWrapper;
        diagnostics.enableExperimental -> diagnostics.experimental.enabled;
        experimental.procAttrMacros -> procMacro.attributes.enabled;
        highlighting.strings -> semanticHighlighting.strings.enabled;
        highlightRelated.breakPoints -> semanticHighlighting.breakPoints.enabled;
        highlightRelated.exitPoints -> semanticHighlighting.exitPoints.enabled;
        highlightRelated.yieldPoints -> semanticHighlighting.yieldPoints.enabled;
        highlightRelated.references -> semanticHighlighting.references.enabled;
        hover.documentation -> hover.documentation.enabled;
        hover.linksInHover -> hover.links.enabled;
        hoverActions.linksInHover -> hover.links.enabled;
        hoverActions.debug -> hoverActions.debug.enabled;
        hoverActions.enable -> hoverActions.enable.enabled;
        hoverActions.gotoTypeDef -> hoverActions.gotoTypeDef.enabled;
        hoverActions.implementations -> hoverActions.implementations.enabled;
        hoverActions.references -> hoverActions.references.enabled;
        hoverActions.run -> hoverActions.run.enabled;
        inlayHints.chainingHints -> inlayHints.chainingHints.enabled;
        inlayHints.closureReturnTypeHints -> inlayHints.closureReturnTypeHints.enabled;
        inlayHints.hideNamedConstructorHints -> inlayHints.typeHints.hideNamedConstructorHints;
        inlayHints.parameterHints -> inlayHints.parameterHints.enabled;
        inlayHints.reborrowHints -> inlayHints.reborrowHints.enabled;
        inlayHints.typeHints -> inlayHints.typeHints.enabled;
        lruCapacity -> lru.capacity;
        runnables.cargoExtraArgs -> runnables.extraArgs ;
        runnables.overrideCargo -> runnables.command ;
        rustcSource -> rustc.source;
        rustfmt.enableRangeFormatting -> rustfmt.rangeFormatting.enabled;
    }

    // completion.snippets -> completion.snippets.custom;
    if let Some(Value::Object(obj)) = copy.pointer("/completion/snippets").cloned() {
        if obj.len() != 1 || obj.get("custom").is_none() {
            merge(
                json,
                json! {{
                    "completion": {
                        "snippets": {
                            "custom": obj
                        },
                    },
                }},
            );
        }
    }

    // callInfo_full -> signatureInfo_detail, signatureInfo_documentation_enable
    if let Some(Value::Bool(b)) = copy.pointer("/callInfo/full") {
        let sig_info = match b {
            true => json!({ "signatureInfo": {
                "documentation": {"enabled": true}},
                "detail": "full"
            }),
            false => json!({ "signatureInfo": {
                "documentation": {"enabled": false}},
                "detail": "parameters"
            }),
        };
        merge(json, sig_info);
    }

    // cargo_allFeatures, cargo_features -> cargo_features
    if let Some(Value::Bool(true)) = copy.pointer("/cargo/allFeatures") {
        merge(json, json!({ "cargo": { "features": "all" } }));
    }

    // checkOnSave_allFeatures, checkOnSave_features -> checkOnSave_features
    if let Some(Value::Bool(true)) = copy.pointer("/checkOnSave/allFeatures") {
        merge(json, json!({ "checkOnSave": { "features": "all" } }));
    }

    // completion_addCallArgumentSnippets completion_addCallParenthesis -> completion_callable_snippets
    let res = match (
        copy.pointer("/completion/addCallArgumentSnippets"),
        copy.pointer("/completion/addCallParenthesis"),
    ) {
        (Some(Value::Bool(true)), Some(Value::Bool(true))) => json!("fill_arguments"),
        (Some(Value::Bool(true)), _) => json!("add_parentheses"),
        (_, _) => json!(null),
    };
    merge(json, json!({ "completion": { "callable": {"snippets": res }} }));
}

fn merge(dst: &mut Value, src: Value) {
    match (dst, src) {
        (Value::Object(dst), Value::Object(src)) => {
            for (k, v) in src {
                merge(dst.entry(k).or_insert(v.clone()), v)
            }
        }
        (dst, src) => *dst = src,
    }
}
