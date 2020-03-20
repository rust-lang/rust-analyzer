use ra_proc_macro::{msg, ExpansionResult, ExpansionTask};
use std::io;

fn read_task() -> Result<Option<ExpansionTask>, io::Error> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let msg = msg::Message::read(&mut stdin)?;

    let msg = match msg {
        None => return Ok(None),
        Some(msg) => msg,
    };

    let req = match msg {
        msg::Message::Request(req) => req,
        msg::Message::Response(_) => {
            // Ignore response here.
            return Ok(None);
        }
    };

    Ok(serde_json::from_value(req.params)?)
}

fn write_result(id: u64, res: Result<ExpansionResult, String>) -> Result<(), io::Error> {
    let msg: msg::Message = match res {
        Ok(result) => msg::Response {
            id: id.into(),
            result: Some(serde_json::to_value(result)?),
            error: None,
        },
        Err(err) => {
            let err = msg::ResponseError {
                code: msg::ErrorCode::ExpansionError as i32,
                message: err,
                data: None,
            };
            msg::Response { id: id.into(), result: None, error: Some(err) }
        }
    }
    .into();

    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    msg.write(&mut stdout)
}
fn main() {
    let mut response_id = 0;
    loop {
        let task = read_task();
        match task {
            Err(err) => {
                eprintln!("Read message error on ra_proc_macro_srv: {}", err.to_string());
            }
            Ok(None) => (),
            Ok(Some(task)) => {
                if let Err(err) = write_result(response_id, ra_proc_macro_srv::expand_task(&task)) {
                    eprintln!("Write message error on ra_proc_macro_srv: {}", err);
                }
                response_id += 1;
            }
        }
    }
}
