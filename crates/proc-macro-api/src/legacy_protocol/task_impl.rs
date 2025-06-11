//! Implement how to send and receive legacy protocol message on stdio stream(protocol layer)
//!
//! The implementation follows:
//! 1. Send Request
//! 2. Receive Response

use std::{
    io::{self, BufRead, Write},
    sync::Arc,
};

use crate::{
    ServerError,
    legacy_protocol::{
        json::{read_json, write_json},
        msg::{Request, Response},
    },
    task::TaskClient,
};

/// Sends a request to the server and reads the response.
fn send_request(
    mut writer: &mut dyn Write,
    mut reader: &mut dyn BufRead,
    req: Request,
    buf: &mut String,
) -> Result<Option<Response>, ServerError> {
    use crate::legacy_protocol::msg::Message;

    req.write(write_json, &mut writer).map_err(|err| ServerError {
        message: "failed to write request".into(),
        io: Some(Arc::new(err)),
    })?;
    let res = Response::read(read_json, &mut reader, buf).map_err(|err| ServerError {
        message: "failed to read response".into(),
        io: Some(Arc::new(err)),
    })?;
    Ok(res)
}

pub struct JsonTaskClient<'a> {
    pub writer: &'a mut dyn Write,
    pub reader: &'a mut dyn BufRead,
    pub buf: &'a mut String,
}

impl TaskClient for JsonTaskClient<'_> {
    type Task = Request;
    type TaskResult = Response;

    // Implement send_task for Json Legacy Protocol
    // Basically what we have done in process.rs
    fn send_task(&mut self, task: Self::Task) -> Result<Self::TaskResult, ServerError> {
        send_request(self.writer, self.reader, task, self.buf).and_then(|res| {
            res.ok_or_else(|| {
                let message = "proc-macro server did not respond with data".to_owned();
                ServerError {
                    io: Some(Arc::new(io::Error::new(io::ErrorKind::BrokenPipe, message.clone()))),
                    message,
                }
            })
        })
    }
}
