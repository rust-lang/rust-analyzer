//! New Protocol Message Definitions covers the functionalities of `legacy_protocol` and with several changes:
//!
//! - Using [postcard](https://github.com/jamesmunns/postcard) as serde library, which provides binary, lightweight serialization.
//! - As we change to postcard, no need to use FlatTree, use `tt::TopSubtree` directly
//!
//! One possible communication may look like this:
//! client ------------------------- server
//!        >-------Request--------->
//!        <-------Query-----------<
//!        >-------Reply----------->
//!        <-------Response-------<

use paths::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::ProcMacroKind;
use crate::legacy_protocol::msg::FlatTree;

/// Represents messages send from client to proc-macr-srv
#[derive(Debug, Serialize, Deserialize)]
pub enum C2SMsg {
    Request(Request),
    // NOTE: Reply is left empty as a placeholder
    Reply,
}

/// Represents messages send from client to proc-macr-srv
#[derive(Debug, Serialize, Deserialize)]
pub enum S2CMsg {
    Response(Response),
    // NOTE: Query is left empty as a place holder
    Query,
}

// NOTE: Directly copied from legacy_protocol
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    ListMacros { dylib_path: Utf8PathBuf },
    ExpandMacro(Box<ExpandMacro>),
    ApiVersionCheck {},
    SetConfig(ServerConfig),
}

// NOTE: Directly copied from legacy_protocol
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    ListMacros(Result<Vec<(String, ProcMacroKind)>, String>),
    ExpandMacro(Result<TreeWrapper, PanicMessage>),
    ApiVersionCheck(u32),
    SetConfig(ServerConfig),
    ExpandMacroExtended(Result<ExpandMacroExtended, PanicMessage>),
}

// NOTE: Directly copied from legacy_protocol
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ServerConfig {
    pub span_mode: SpanMode,
}

// NOTE: Directly copied from legacy_protocol,
// except the tree field
#[derive(Debug, Serialize, Deserialize)]
pub struct ExpandMacroExtended {
    /// The expanded syntax tree.
    pub tree: TreeWrapper,
    /// Additional span data mappings.
    pub span_data_table: Vec<u32>,
}

// NOTE: Directly copied from legacy_protocol
#[derive(Debug, Serialize, Deserialize)]
pub struct PanicMessage(pub String);

// NOTE: Directly copied from legacy_protocol
#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub enum SpanMode {
    #[default]
    Id,
    RustAnalyzer,
}

// NOTE: Directly copied from legacy_protocol
#[derive(Debug, Serialize, Deserialize)]
pub struct ExpandMacro {
    pub lib: Utf8PathBuf,

    pub env: Vec<(String, String)>,

    pub current_dir: Option<String>,

    #[serde(flatten)]
    pub data: ExpandMacroData,
}

// TODO: Maybe ideal we want to ser/de TokenTree directly
//
// Something like this should be ideal
// pub struct TreeWrapper<S>(tt::TokenTree<S>);
// currently, just use this wrapper to build the backbones
#[derive(Debug, Serialize, Deserialize)]
pub struct TreeWrapper(FlatTree);

// NOTE: Directly copied from legacy_protocol,
// except the tree field
#[derive(Debug, Serialize, Deserialize)]
pub struct ExpandMacroData {
    pub macro_body: TreeWrapper,

    pub macro_name: String,

    pub attributes: Option<TreeWrapper>,
    #[serde(skip_serializing_if = "ExpnGlobals::skip_serializing_if")]
    #[serde(default)]
    pub has_global_spans: ExpnGlobals,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub span_data_table: Vec<u32>,
}

// NOTE: Directly copied from legacy_protocol,
#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct ExpnGlobals {
    #[serde(skip_serializing)]
    #[serde(default)]
    pub serialize: bool,
    pub def_site: usize,
    pub call_site: usize,
    pub mixed_site: usize,
}

impl ExpnGlobals {
    fn skip_serializing_if(&self) -> bool {
        !self.serialize
    }
}
