use std::{
    io::{self, BufReader},
    net::TcpStream,
    thread,
};

use crossbeam_channel::{bounded, Receiver, Sender};

use crate::{
    stdio::{make_io_threads, IoThreads},
    Message, Notification,
};

fn base_socket_transport<ExitF>(
    stream: TcpStream,
    is_exit: ExitF,
) -> (Sender<Message>, Receiver<Message>, IoThreads)
where
    ExitF: Fn(&Notification) -> bool + Send + 'static,
{
    let (reader_receiver, reader) = make_reader(stream.try_clone().unwrap(), is_exit);
    let (writer_sender, writer) = make_write(stream);
    let io_threads = make_io_threads(reader, writer);
    (writer_sender, reader_receiver, io_threads)
}

pub(crate) fn socket_transport(
    stream: TcpStream,
) -> (Sender<Message>, Receiver<Message>, IoThreads) {
    base_socket_transport(stream, Notification::is_exit)
}

#[cfg(feature = "bsp")]
pub(crate) fn bsp_socket_transport(
    stream: TcpStream,
) -> (Sender<Message>, Receiver<Message>, IoThreads) {
    base_socket_transport(stream, Notification::bsp_is_exit)
}

fn make_reader<ExitF>(
    stream: TcpStream,
    is_exit: ExitF,
) -> (Receiver<Message>, thread::JoinHandle<io::Result<()>>)
where
    ExitF: Fn(&Notification) -> bool + Send + 'static,
{
    let (reader_sender, reader_receiver) = bounded::<Message>(0);
    let reader = thread::spawn(move || {
        let mut buf_read = BufReader::new(stream);
        while let Some(msg) = Message::read(&mut buf_read).unwrap() {
            let is_exit_notif = matches!(&msg, Message::Notification(n) if is_exit(&n));
            reader_sender.send(msg).unwrap();
            if is_exit_notif {
                break;
            }
        }
        Ok(())
    });
    (reader_receiver, reader)
}

fn make_write(mut stream: TcpStream) -> (Sender<Message>, thread::JoinHandle<io::Result<()>>) {
    let (writer_sender, writer_receiver) = bounded::<Message>(0);
    let writer = thread::spawn(move || {
        writer_receiver.into_iter().try_for_each(|it| it.write(&mut stream)).unwrap();
        Ok(())
    });
    (writer_sender, writer)
}
