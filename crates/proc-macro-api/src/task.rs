//! task.rs is used to abstract ProcMacroServerProcess::send_task in process.rs
//! The ultimate goal is to encapsulate legacy_protocol and don't directly use legacy_protocol in process.rs
//!
use crate::legacy_protocol::msg::{Request, Response};
use crate::new_protocol::msg::{C2SMsg, S2CMsg};

pub trait TaskClient {
    type Task;
    type TaskResult;

    fn send_task(&self, task: Self::Task) -> Self::TaskResult;
}

pub struct JsonTaskClient;

impl TaskClient for JsonTaskClient {
    type Task = Request;
    type TaskResult = Response;

    // Implement send_task for Json Legacy Protocol
    // Basically what we have done in process.rs
    fn send_task(&self, _task: Self::Task) -> Self::TaskResult {
        todo!("implement JsonTaskClient::send_task");
    }
}

pub struct PostcardTaskClient;

impl TaskClient for PostcardTaskClient {
    type Task = C2SMsg;
    type TaskResult = S2CMsg;

    // Implement send_task for Postcard Protocol
    // As this new Protol will allow back and forth communication
    // send_task will abstract these back and forth details
    // Task will be C2SMsg::Request, which is basically legacy_protocol Request
    // TaskResult will be S2CMsg::Response, which is basically legacy_protocol Response
    fn send_task(&self, _task: Self::Task) -> Self::TaskResult {
        todo!("implement PostcardTaskClient::send_task, back and forth");
    }
}
