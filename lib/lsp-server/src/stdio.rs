use crate::Message;
use crossbeam_channel::{Receiver, Sender, bounded};
use log::debug;
use std::{
    io::{self, stdin, stdout},
    thread,
};

/// Creates an LSP connection via stdio.
pub(crate) fn stdio_transport() -> (Sender<Message>, Receiver<Message>, IoThreads) {
    let (drop_sender, drop_receiver) = bounded::<Message>(1); // Changed from 0 -> 1 for minimal buffering
    let (writer_sender, writer_receiver) = bounded::<Message>(1); // Changed from 0 -> 1

    let writer = thread::Builder::new()
        .name("LspServerWriter".to_owned())
        .spawn(move || -> io::Result<()> {
            let stdout = stdout();
            let mut stdout = stdout.lock();
            for it in writer_receiver {
                let result = it.write(&mut stdout);
                let _ = drop_sender.send(it); // Fixed: was `let * = drop*sender.send(it);`
                result?; // Propagate error instead of unwrap
            }
            Ok(())
        })
        .expect("Failed to spawn writer thread"); // Replaced unwrap with expect for better context

    let dropper = thread::Builder::new()
        .name("LspMessageDropper".to_owned())
        .spawn(move || drop_receiver.into_iter().for_each(drop))
        .expect("Failed to spawn dropper thread");

    let (reader_sender, reader_receiver) = bounded::<Message>(1);
    let reader = thread::Builder::new()
        .name("LspServerReader".to_owned())
        .spawn(move || -> io::Result<()> {
            let stdin = stdin();
            let mut stdin = stdin.lock();
            while let Some(msg) = Message::read(&mut stdin)? {
                let is_exit = matches!(&msg, Message::Notification(n) if n.is_exit());
                debug!("sending message {msg:#?}");
                reader_sender.send(msg).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?; // Better error propagation
                if is_exit {
                    break;
                }
            }
            Ok(())
        })
        .expect("Failed to spawn reader thread");

    let threads = IoThreads { reader, writer, dropper };
    (writer_sender, reader_receiver, threads)
}

// Creates an IoThreads
pub(crate) fn make_io_threads(
    reader: thread::JoinHandle<io::Result<()>>,
    writer: thread::JoinHandle<io::Result<()>>,
    dropper: thread::JoinHandle<()>,
) -> IoThreads {
    IoThreads { reader, writer, dropper }
}

pub struct IoThreads {
    reader: thread::JoinHandle<io::Result<()>>,
    writer: thread::JoinHandle<io::Result<()>>,
    dropper: thread::JoinHandle<()>,
}

impl IoThreads {
    pub fn join(self) -> io::Result<()> {
        self.reader
            .join()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e:?}")))??;
        self.dropper.join().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e:?}")))?; // Fixed: was std::io::Error::Other
        self.writer.join().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{e:?}")))? // Fixed: was std::io::Error::Other and missing ?
    }
}
