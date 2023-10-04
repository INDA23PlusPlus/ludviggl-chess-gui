
use std::sync::mpsc::{ self, Receiver, Sender, };
use std::net::TcpStream;
use std::thread;
use serde::{ Serialize as Ser, de::DeserializeOwned as Des, };

pub trait Message: Ser + Des + Send + 'static {}
impl<T: Ser + Des + Send + 'static> Message for T {}

pub fn write<T: Message>(stream: &TcpStream, msg: T) {

    serde_json::to_writer(stream, &msg).unwrap();
}

pub fn read<T: Message>(stream: &TcpStream) -> T {

    let mut de = serde_json::Deserializer::from_reader(stream);
    T::deserialize(&mut de).unwrap()
}

type ThreadResult = ();

pub struct TcpHandler<R, W> 
    where R: Message,
          W: Message
{

    receiver:     Receiver<R>,
    sender:       Sender<W>,
    read_handle:  thread::JoinHandle<ThreadResult>,
    write_handle: thread::JoinHandle<ThreadResult>,
}

impl<R, W> TcpHandler<R, W>
    where R: Message,
          W: Message
{

    pub fn new(stream: TcpStream) -> Self {

        let (read_sender, receiver)  = mpsc::channel();
        let (sender, write_receiver) = mpsc::channel();

        let stream2 = stream.try_clone().unwrap();

        let read_handle = thread::spawn(move ||
            Self::read_loop(stream, read_sender)
        );

        let write_handle = thread::spawn(move ||
            Self::write_loop(stream2, write_receiver)
        );

        Self {

            receiver,
            sender,
            read_handle,
            write_handle,
        }
    }

    pub fn read(&self) -> Option<R> {

        match self.receiver.try_recv() {
            Ok(msg) => Some(msg),
            Err(e) => match e {
                mpsc::TryRecvError::Empty => None,
                mpsc::TryRecvError::Disconnected => panic!(),
            },
        }
    }

    pub fn read_blocking(&self) -> R {

        loop {

            if let Some(msg) = self.read() {
                return msg;
            }
        }
    }

    pub fn write(&self, msg: W) {

        self.sender.send(msg).unwrap();
    }

    fn read_loop(
        stream: TcpStream,
        sender: Sender<R>,
    ) -> ThreadResult {

        loop {

            let msg = serde_json::from_reader(&stream).unwrap();
            sender.send(msg).unwrap();
        }
    }

    fn write_loop(
        stream: TcpStream,
        receiver: Receiver<W>,
    ) -> ThreadResult {

        loop {

            match receiver.try_recv() {
                Ok(msg) => serde_json::to_writer(&stream, &msg).unwrap(),
                Err(e) => match e {
                    mpsc::TryRecvError::Empty => (),
                    mpsc::TryRecvError::Disconnected => return (),
                }
            }
        }
    }
}
