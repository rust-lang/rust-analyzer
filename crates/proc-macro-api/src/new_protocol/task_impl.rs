//! Implement how to send and receive new protocol message on stdio stream(protocol layer)
//!
//! Current implementation:
//! 1. Send Request
//! 2. Handle possible query and send reply
//! 3. Receive Response

use std::io::{BufRead, Write};

use crate::{
    ServerError,
    new_protocol::{
        msg::{C2SMsg, S2CMsg},
        postcard::ProtoPostcard,
    },
    task::TaskClient,
};

pub struct PostcardTaskClient<'a> {
    pub writer: &'a mut dyn Write,
    pub reader: &'a mut dyn BufRead,
}

impl TaskClient for PostcardTaskClient<'_> {
    type Task = C2SMsg;
    type TaskResult = S2CMsg;

    // Implement send_task for Postcard Protocol
    // As this new Protocol will allow back and forth communication
    // send_task will abstract these back and forth details
    // Task will be C2SMsg::Request, which is basically legacy_protocol Request
    // TaskResult will be S2CMsg::Response, which is basically legacy_protocol Response
    fn send_task(&mut self, task: Self::Task) -> Result<Self::TaskResult, ServerError> {
        // Send the initial C2SMsg (client-to-server message)
        task.send_proto(self.writer).map_err(|err| ServerError {
            message: "failed to send request".into(),
            io: Some(std::sync::Arc::new(err)),
        })?;

        // Handle bidirectional communication loop
        loop {
            // Receive response from server
            let server_msg = S2CMsg::receive_proto(self.reader).map_err(|err| ServerError {
                message: "failed to read server message".into(),
                io: Some(std::sync::Arc::new(err)),
            })?;

            match server_msg {
                S2CMsg::Response(_) => {
                    // Final response received, return it
                    return Ok(server_msg);
                }
                S2CMsg::Query => {
                    // Server sent a query, we need to send a reply
                    // For now, send an empty Reply as placeholder
                    let reply = C2SMsg::Reply;
                    reply.send_proto(self.writer).map_err(|err| ServerError {
                        message: "failed to send reply".into(),
                        io: Some(std::sync::Arc::new(err)),
                    })?;
                    // Continue the loop to wait for the next server message
                }
            }
        }
    }
}
