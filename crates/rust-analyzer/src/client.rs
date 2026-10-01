//! Multi-client abstraction for `rust-analyzer`.

use crossbeam_channel::Sender;
use vfs::AbsPathBuf;

use crate::{
    global_state::ReqQueue, line_index::PositionEncoding, lsp::capabilities::ClientCapabilities,
};

/// Unique identifier for an attached language client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClientId(pub u32);

impl ClientId {
    /// The default client ID used for standalone / single-client mode.
    pub const DEFAULT: ClientId = ClientId(0);
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "client#{}", self.0)
    }
}

pub(crate) struct Client {
    pub(crate) sender: Sender<lsp_server::Message>,
    pub(crate) req_queue: ReqQueue,
    pub(crate) shutdown_requested: bool,
    pub(crate) position_encoding: PositionEncoding,
    pub(crate) is_initialized: bool,
    pub(crate) caps: ClientCapabilities,
    /// The directory the client works in, if it told us.
    pub(crate) root: Option<AbsPathBuf>,
}

impl Client {
    pub(crate) fn new(
        sender: Sender<lsp_server::Message>,
        position_encoding: PositionEncoding,
        caps: ClientCapabilities,
    ) -> Self {
        Client {
            sender,
            req_queue: ReqQueue::default(),
            shutdown_requested: false,
            position_encoding,
            is_initialized: false,
            caps,
            root: None,
        }
    }
}
