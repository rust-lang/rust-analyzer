use crossbeam_channel::{bounded, Receiver, Sender};
use ra_mbe::ProcMacroError;
use ra_tt::Subtree;

use crate::msg::{ErrorCode, Message, Request, Response, ResponseError};
use crate::rpc::{ExpansionResult, ExpansionTask};

use io::{BufRead, BufReader};
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread::spawn,
};

#[derive(Debug, Default)]
pub(crate) struct ProcMacroProcessExpander {
    inner: Option<Handle>,
}

#[derive(Debug)]
struct Handle {
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

struct Process {
    path: PathBuf,
    child: Child,
}

impl Process {
    fn run(process_path: &Path) -> Result<Process, io::Error> {
        let child = Command::new(process_path.clone())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        Ok(Process { path: process_path.into(), child })
    }

    fn restart(&mut self) -> Result<(), io::Error> {
        let _ = self.child.kill();
        self.child =
            Command::new(self.path.clone()).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
        Ok(())
    }

    fn stdio(&mut self) -> Option<(impl Write, impl BufRead)> {
        let stdin = self.child.stdin.take()?;
        let stdout = self.child.stdout.take()?;
        let read = BufReader::new(stdout);

        Some((stdin, read))
    }
}

impl ProcMacroProcessExpander {
    pub fn run(process_path: &Path) -> Result<ProcMacroProcessExpander, io::Error> {
        let process = Process::run(process_path)?;

        let (task_tx, task_rx) = bounded(0);
        let (res_tx, res_rx) = bounded(0);

        let _ = spawn(move || {
            client_loop(task_rx, res_tx, process);
        });
        Ok(ProcMacroProcessExpander { inner: Some(Handle { sender: task_tx, receiver: res_rx }) })
    }

    pub fn custom_derive(
        &self,
        dylib_path: &Path,
        subtree: &Subtree,
        derive_name: &str,
    ) -> Result<Subtree, ProcMacroError> {
        let handle = match &self.inner {
            None => return Err(ProcMacroError::Dummy),
            Some(it) => it,
        };

        let task = ExpansionTask {
            macro_body: subtree.clone(),
            macro_name: derive_name.to_string(),
            attributes: None,
            lib: dylib_path.to_path_buf(),
        };

        // FIXME: use a proper request id
        let id = 0;
        let req = Request {
            id: id.into(),
            method: "".into(),
            params: serde_json::to_value(task).unwrap(),
        };

        handle.sender.send(req.into()).unwrap();
        let response = handle.receiver.recv().unwrap();

        match response {
            Message::Request(_) => {
                return Err(ProcMacroError::Unknown("Return request from ra_proc_srv".into()))
            }
            Message::Response(res) => {
                if let Some(err) = res.error {
                    return Err(ProcMacroError::ExpansionError(err.message));
                }
                match res.result {
                    None => Ok(Subtree::default()),
                    Some(res) => {
                        let result: ExpansionResult = serde_json::from_value(res)
                            .map_err(|err| ProcMacroError::JsonError(err.to_string()))?;
                        Ok(result.expansion)
                    }
                }
            }
        }
    }
}

fn client_loop(task_rx: Receiver<Message>, res_tx: Sender<Message>, mut process: Process) {
    let (mut stdin, mut stdout) = match process.stdio() {
        None => return,
        Some(it) => it,
    };

    loop {
        let msg = match task_rx.recv() {
            Ok(msg) => msg,
            Err(_) => break,
        };

        let res = match send_message(&mut stdin, &mut stdout, msg) {
            Ok(res) => res,
            Err(_err) => {
                let res = Response {
                    id: 0.into(),
                    result: None,
                    error: Some(ResponseError {
                        code: ErrorCode::ServerErrorEnd as i32,
                        message: "Server closed".into(),
                        data: None,
                    }),
                };
                if res_tx.send(res.into()).is_err() {
                    break;
                }
                // Restart the process
                if process.restart().is_err() {
                    break;
                }
                let stdio = match process.stdio() {
                    None => break,
                    Some(it) => it,
                };
                stdin = stdio.0;
                stdout = stdio.1;
                continue;
            }
        };

        if let Some(res) = res {
            if res_tx.send(res).is_err() {
                break;
            }
        }
    }

    let _ = process.child.kill();
}

fn send_message(
    mut writer: &mut impl Write,
    mut reader: &mut impl BufRead,
    msg: Message,
) -> Result<Option<Message>, io::Error> {
    msg.write(&mut writer)?;
    Ok(Message::read(&mut reader)?)
}
