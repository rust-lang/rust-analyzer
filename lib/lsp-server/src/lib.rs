//! A language server scaffold, exposing a synchronous crossbeam-channel based API.
//! This crate handles protocol handshaking and parsing messages, while you
//! control the message dispatch loop yourself.
//!
//! Run with `RUST_LOG=lsp_server=debug` to see all the messages.

#![warn(rust_2018_idioms, unused_lifetimes, semicolon_in_expressions_from_macros)]

mod msg;
mod stdio;
mod error;
mod socket;
mod req_queue;

use std::{
    io,
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

use crossbeam_channel::{Receiver, Sender};

pub use crate::{
    error::{ExtractError, ProtocolError},
    msg::{ErrorCode, Message, Notification, Request, RequestId, Response, ResponseError},
    req_queue::{Incoming, Outgoing, ReqQueue},
    stdio::IoThreads,
};

/// Connection is just a pair of channels of LSP messages.
pub struct Connection {
    pub sender: Sender<Message>,
    pub receiver: Receiver<Message>,
}

impl Connection {
    /// Create connection over standard in/standard out.
    ///
    /// Use this to create a real language server.
    fn base_stdio<F>(transport: F) -> (Connection, IoThreads)
    where
        F: FnOnce() -> (Sender<Message>, Receiver<Message>, IoThreads),
    {
        let (sender, receiver, io_threads) = transport();
        (Connection { sender, receiver }, io_threads)
    }

    pub fn stdio() -> (Connection, IoThreads) {
        Self::base_stdio(stdio::stdio_transport)
    }

    #[cfg(feature = "bsp")]
    pub fn bsp_stdio() -> (Connection, IoThreads) {
        Self::base_stdio(stdio::bsp_stdio_transport)
    }

    /// Open a connection over tcp.
    /// This call blocks until a connection is established.
    ///
    /// Use this to create a real language server.
    fn base_connect<A, F>(addr: A, transport: F) -> io::Result<(Connection, IoThreads)>
    where
        A: ToSocketAddrs,
        F: FnOnce(TcpStream) -> (Sender<Message>, Receiver<Message>, IoThreads),
    {
        let stream = TcpStream::connect(addr)?;
        let (sender, receiver, io_threads) = transport(stream);
        Ok((Connection { sender, receiver }, io_threads))
    }

    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<(Connection, IoThreads)> {
        Self::base_connect(addr, socket::socket_transport)
    }

    #[cfg(feature = "bsp")]
    pub fn bsp_connect<A: ToSocketAddrs>(addr: A) -> io::Result<(Connection, IoThreads)> {
        Self::base_connect(addr, socket::bsp_socket_transport)
    }

    /// Listen for a connection over tcp.
    /// This call blocks until a connection is established.
    ///
    /// Use this to create a real language server.
    fn base_listen<A, F>(addr: A, transport: F) -> io::Result<(Connection, IoThreads)>
    where
        A: ToSocketAddrs,
        F: FnOnce(TcpStream) -> (Sender<Message>, Receiver<Message>, IoThreads),
    {
        let listener = TcpListener::bind(addr)?;
        let (stream, _) = listener.accept()?;
        let (sender, receiver, io_threads) = transport(stream);
        Ok((Connection { sender, receiver }, io_threads))
    }

    pub fn listen<A: ToSocketAddrs>(addr: A) -> io::Result<(Connection, IoThreads)> {
        Self::base_listen(addr, socket::socket_transport)
    }

    #[cfg(feature = "bsp")]
    pub fn bsp_listen<A: ToSocketAddrs>(addr: A) -> io::Result<(Connection, IoThreads)> {
        Self::base_listen(addr, socket::bsp_socket_transport)
    }

    /// Creates a pair of connected connections.
    ///
    /// Use this for testing.
    pub fn memory() -> (Connection, Connection) {
        let (s1, r1) = crossbeam_channel::unbounded();
        let (s2, r2) = crossbeam_channel::unbounded();
        (Connection { sender: s1, receiver: r2 }, Connection { sender: s2, receiver: r1 })
    }

    /// Starts the initialization process by waiting for an initialize
    /// request from the client. Use this for more advanced customization than
    /// `initialize` can provide.
    ///
    /// Returns the request id and serialized `InitializeParams` from the client.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::error::Error;
    /// use lsp_types::{ClientCapabilities, InitializeParams, ServerCapabilities};
    ///
    /// use lsp_server::{Connection, Message, Request, RequestId, Response};
    ///
    /// fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    ///    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    ///    // also be implemented to use sockets or HTTP.
    ///    let (connection, io_threads) = Connection::stdio();
    ///
    ///    // Run the server
    ///    let (id, params) = connection.initialize_start()?;
    ///
    ///    let init_params: InitializeParams = serde_json::from_value(params).unwrap();
    ///    let client_capabilities: ClientCapabilities = init_params.capabilities;
    ///    let server_capabilities = ServerCapabilities::default();
    ///
    ///    let initialize_data = serde_json::json!({
    ///        "capabilities": server_capabilities,
    ///        "serverInfo": {
    ///            "name": "lsp-server-test",
    ///            "version": "0.1"
    ///        }
    ///    });
    ///
    ///    connection.initialize_finish(id, initialize_data)?;
    ///
    ///    // ... Run main loop ...
    ///
    ///    Ok(())
    /// }
    /// ```
    fn base_initialize_start<InitF, ExitF>(
        &self,
        is_initialize: InitF,
        is_exit: ExitF,
    ) -> Result<(RequestId, serde_json::Value), ProtocolError>
    where
        InitF: Fn(&Request) -> bool,
        ExitF: Fn(&Notification) -> bool,
    {
        loop {
            break match self.receiver.recv() {
                Ok(Message::Request(req)) if is_initialize(&req) => Ok((req.id, req.params)),
                // Respond to non-initialize requests with ServerNotInitialized
                Ok(Message::Request(req)) => {
                    let resp = Response::new_err(
                        req.id.clone(),
                        ErrorCode::ServerNotInitialized as i32,
                        format!("expected initialize request, got {req:?}"),
                    );
                    self.sender.send(resp.into()).unwrap();
                    continue;
                }
                Ok(Message::Notification(n)) if !is_exit(&n) => {
                    continue;
                }
                Ok(msg) => Err(ProtocolError(format!("expected initialize request, got {msg:?}"))),
                Err(e) => {
                    Err(ProtocolError(format!("expected initialize request, got error: {e}")))
                }
            };
        }
    }

    pub fn initialize_start(&self) -> Result<(RequestId, serde_json::Value), ProtocolError> {
        self.base_initialize_start(Request::is_initialize, Notification::is_exit)
    }

    #[cfg(feature = "bsp")]
    pub fn bsp_initialize_start(&self) -> Result<(RequestId, serde_json::Value), ProtocolError> {
        self.base_initialize_start(Request::bsp_is_initialize, Notification::bsp_is_exit)
    }

    /// Finishes the initialization process by sending an `InitializeResult` to the client
    fn base_initialize_finish<F>(
        &self,
        initialize_id: RequestId,
        initialize_result: serde_json::Value,
        is_initialized: F,
    ) -> Result<(), ProtocolError>
    where
        F: FnOnce(&Notification) -> bool,
    {
        let resp = Response::new_ok(initialize_id, initialize_result);
        self.sender.send(resp.into()).unwrap();
        match &self.receiver.recv() {
            Ok(Message::Notification(n)) if is_initialized(&n) => Ok(()),
            Ok(msg) => {
                Err(ProtocolError(format!(r#"expected initialized notification, got: {msg:?}"#)))
            }
            Err(e) => {
                Err(ProtocolError(format!("expected initialized notification, got error: {e}",)))
            }
        }
    }

    pub fn initialize_finish(
        &self,
        initialize_id: RequestId,
        initialize_result: serde_json::Value,
    ) -> Result<(), ProtocolError> {
        self.base_initialize_finish(initialize_id, initialize_result, Notification::is_initialized)
    }

    #[cfg(feature = "bsp")]
    pub fn bsp_initialize_finish(
        &self,
        initialize_id: RequestId,
        initialize_result: serde_json::Value,
    ) -> Result<(), ProtocolError> {
        self.base_initialize_finish(
            initialize_id,
            initialize_result,
            Notification::bsp_is_initialized,
        )
    }

    /// Initialize the connection. Sends the server capabilities
    /// to the client and returns the serialized client capabilities
    /// on success. If more fine-grained initialization is required use
    /// `initialize_start`/`initialize_finish`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::error::Error;
    /// use lsp_types::ServerCapabilities;
    ///
    /// use lsp_server::{Connection, Message, Request, RequestId, Response};
    ///
    /// fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    ///    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    ///    // also be implemented to use sockets or HTTP.
    ///    let (connection, io_threads) = Connection::stdio();
    ///
    ///    // Run the server
    ///    let server_capabilities = serde_json::to_value(&ServerCapabilities::default()).unwrap();
    ///    let initialization_params = connection.initialize(server_capabilities)?;
    ///
    ///    // ... Run main loop ...
    ///
    ///    Ok(())
    /// }
    /// ```
    fn base_initialize<StartF, FinishF>(
        &self,
        server_capabilities: serde_json::Value,
        start: StartF,
        finish: FinishF,
    ) -> Result<serde_json::Value, ProtocolError>
    where
        StartF: FnOnce(&Self) -> Result<(RequestId, serde_json::Value), ProtocolError>,
        FinishF: FnOnce(&Self, RequestId, serde_json::Value) -> Result<(), ProtocolError>,
    {
        let (id, params) = start(self)?;

        let initialize_data = serde_json::json!({
            "capabilities": server_capabilities,
        });

        finish(self, id, initialize_data)?;

        Ok(params)
    }

    pub fn initialize(
        &self,
        server_capabilities: serde_json::Value,
    ) -> Result<serde_json::Value, ProtocolError> {
        self.base_initialize(
            server_capabilities,
            Connection::initialize_start,
            Connection::initialize_finish,
        )
    }

    #[cfg(feature = "bsp")]
    pub fn bsp_initialize(
        &self,
        server_capabilities: serde_json::Value,
    ) -> Result<serde_json::Value, ProtocolError> {
        self.base_initialize(
            server_capabilities,
            Connection::bsp_initialize_start,
            Connection::bsp_initialize_finish,
        )
    }

    /// If `req` is `Shutdown`, respond to it and return `true`, otherwise return `false`
    fn base_handle_shutdown<ShutdownF, ExitF>(
        &self,
        req: &Request,
        is_shutdown: ShutdownF,
        is_exit: ExitF,
    ) -> Result<bool, ProtocolError>
    where
        ShutdownF: FnOnce(&Request) -> bool,
        ExitF: FnOnce(&Notification) -> bool,
    {
        if !is_shutdown(req) {
            return Ok(false);
        }
        let resp = Response::new_ok(req.id.clone(), ());
        let _ = self.sender.send(resp.into());
        match &self.receiver.recv_timeout(std::time::Duration::from_secs(30)) {
            Ok(Message::Notification(n)) if is_exit(n) => (),
            Ok(msg) => {
                return Err(ProtocolError(format!("unexpected message during shutdown: {msg:?}")))
            }
            Err(e) => return Err(ProtocolError(format!("unexpected error during shutdown: {e}"))),
        }
        Ok(true)
    }

    pub fn handle_shutdown(&self, req: &Request) -> Result<bool, ProtocolError> {
        self.base_handle_shutdown(req, Request::is_shutdown, Notification::is_exit)
    }
    #[cfg(feature = "bsp")]
    pub fn bsp_handle_shutdown(&self, req: &Request) -> Result<bool, ProtocolError> {
        self.base_handle_shutdown(req, Request::bsp_is_shutdown, Notification::bsp_is_exit)
    }
}

#[cfg(test)]
mod tests {
    use lsp_types::notification::{Exit, Initialized, Notification};
    use lsp_types::request::{Initialize, RegisterCapability, Request};
    use lsp_types::{InitializeParams, InitializedParams, Registration, RegistrationParams};
    use serde_json::to_value;

    use crate::{Connection, ErrorCode, Message, ProtocolError, RequestId};

    struct TestCase {
        test_messages: Vec<Message>,
        expected_resp: Result<(RequestId, serde_json::Value), ProtocolError>,
        expected_send: Vec<Message>,
    }

    fn initialize_start_test(test_case: TestCase) {
        let (client, server) = Connection::memory();

        for msg in test_case.test_messages {
            assert!(client.sender.send(msg).is_ok());
        }

        let resp = server.initialize_start();
        assert_eq!(test_case.expected_resp, resp);

        for msg in test_case.expected_send {
            assert_eq!(
                msg,
                client.receiver.recv_timeout(std::time::Duration::from_secs(1)).unwrap()
            );
        }
        assert!(client.receiver.recv_timeout(std::time::Duration::from_secs(1)).is_err());
    }

    #[test]
    fn wrong_req() {
        let params = RegistrationParams {
            registrations: vec![Registration {
                id: "foo".to_string(),
                method: "bar".to_string(),
                register_options: None,
            }],
        };
        let wrong_req_id = RequestId::from(123);
        let wrong_request = crate::Request {
            id: wrong_req_id.clone(),
            method: RegisterCapability::METHOD.to_string(),
            params: to_value(params).unwrap(),
        };

        let params_as_value = to_value(InitializeParams::default()).unwrap();
        let init_req_id = RequestId::from(234);
        let init_request = crate::Request {
            id: init_req_id.clone(),
            method: Initialize::METHOD.into(),
            params: params_as_value.clone(),
        };

        initialize_start_test(TestCase {
            test_messages: vec![wrong_request.clone().into(), init_request.into()],
            expected_resp: Ok((init_req_id, params_as_value)),
            expected_send: vec![crate::Response::new_err(
                wrong_req_id,
                ErrorCode::ServerNotInitialized as i32,
                format!("expected initialize request, got {:?}", wrong_request),
            )
            .into()],
        });
    }

    #[test]
    fn not_exit_notification() {
        let notification = crate::Notification {
            method: Initialized::METHOD.to_string(),
            params: to_value(InitializedParams {}).unwrap(),
        };

        let params_as_value = to_value(InitializeParams::default()).unwrap();
        let req_id = RequestId::from(234);
        let request = crate::Request {
            id: req_id.clone(),
            method: Initialize::METHOD.into(),
            params: params_as_value.clone(),
        };

        initialize_start_test(TestCase {
            test_messages: vec![notification.into(), request.into()],
            expected_resp: Ok((req_id, params_as_value)),
            expected_send: vec![],
        });
    }

    #[test]
    fn exit_notification() {
        let notification =
            crate::Notification { method: Exit::METHOD.into(), params: to_value(()).unwrap() };
        let notification_msg = Message::from(notification);

        initialize_start_test(TestCase {
            test_messages: vec![notification_msg.clone()],
            expected_resp: Err(ProtocolError(format!(
                "expected initialize request, got {:?}",
                notification_msg
            ))),
            expected_send: vec![],
        });
    }
}
