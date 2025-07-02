//! Postcard binary protocol task executor for new protocol support

use std::io;

use proc_macro_api::new_protocol::{
    msg::{C2SMsg, PanicMessage, Request, Response, S2CMsg},
    postcard::ProtoPostcard,
};
use proc_macro_srv::EnvSnapshot;

use crate::task_executor::TaskExecutor;

pub(crate) struct PostcardTaskExecutor;

impl TaskExecutor for PostcardTaskExecutor {
    fn run(&self) -> io::Result<()> {
        fn macro_kind_to_api(kind: proc_macro_srv::ProcMacroKind) -> proc_macro_api::ProcMacroKind {
            match kind {
                proc_macro_srv::ProcMacroKind::CustomDerive => {
                    proc_macro_api::ProcMacroKind::CustomDerive
                }
                proc_macro_srv::ProcMacroKind::Bang => proc_macro_api::ProcMacroKind::Bang,
                proc_macro_srv::ProcMacroKind::Attr => proc_macro_api::ProcMacroKind::Attr,
            }
        }

        let env = EnvSnapshot::default();
        let srv = proc_macro_srv::ProcMacroSrv::new(&env);

        let mut stdin = io::stdin().lock();
        let mut stdout = io::stdout().lock();

        // Main communication loop
        loop {
            // Receive C2SMsg from client
            let client_msg = match C2SMsg::receive_proto(&mut stdin) {
                Ok(msg) => msg,
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    // Client disconnected, exit gracefully
                    break;
                }
                Err(e) => return Err(e),
            };

            match client_msg {
                C2SMsg::Request(request) => {
                    // Process the request and send response
                    let response = match request {
                        Request::ListMacros { dylib_path } => {
                            Response::ListMacros(srv.list_macros(&dylib_path).map(|macros| {
                                macros
                                    .into_iter()
                                    .map(|(name, kind)| (name, macro_kind_to_api(kind)))
                                    .collect()
                            }))
                        }
                        Request::ExpandMacro(_task) => {
                            // TODO: Implement macro expansion for new protocol
                            // This would need to convert TreeWrapper to appropriate types
                            // and handle the expansion similar to JsonTaskExecutor
                            Response::ExpandMacro(Err(PanicMessage("Not Yet Implemented".into())))
                        }
                        Request::ApiVersionCheck {} => Response::ApiVersionCheck(
                            proc_macro_api::legacy_protocol::msg::CURRENT_API_VERSION,
                        ),
                        Request::SetConfig(config) => Response::SetConfig(config),
                    };

                    // Send the response back to client
                    let server_msg = S2CMsg::Response(response);
                    server_msg.send_proto(&mut stdout)?;
                }
                C2SMsg::Reply => {
                    // Handle reply from client (in response to our query)
                    // For now, this is a placeholder as we don't send queries yet
                    // In future, this would handle bidirectional communication
                }
            }
        }

        Ok(())
    }
}
