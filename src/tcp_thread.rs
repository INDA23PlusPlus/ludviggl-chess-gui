
use std::sync::{ Arc, Mutex, };
use std::thread::JoinHandle;
use std::net::TcpStream;

use serde_json;
use serde::{ Serialize, de::DeserializeOwned, };

pub trait Message: Clone + Copy + Serialize + DeserializeOwned {}

#[derive(Clone)]
pub struct Buffer<T: Message> {

    data: Arc<Mutex<Option<T>>>,
}

impl<T: Message> Buffer<T> {

    fn new() -> Self {

        Self {

            data: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get(&self) -> Option<T> {

        let mut data = self.data.lock().unwrap();
        if let Some(msg) = *data {

            *data = None;
            Some(msg)
        } else {

            None
        }
    }

    pub fn set(&self, msg: T) {

        let mut data = self.data.lock().unwrap();
        *data = Some(msg);
    }
}

pub type ThreadResult = ();

pub struct TcpThread<I, O> 
    where I: Message,
          O: Message
{
    
    pub incoming: Buffer<I>,
    pub outgoing: Buffer<O>,

    handle: JoinHandle<ThreadResult>,
}

impl<I, O> TcpThread<I, O>
    where I: Message,
          O: Message
{
    
    pub fn new(stream: TcpStream, read_first: bool) -> Self {

        // Create buffers
        let incoming = Buffer<I>::new();
        let outgoing = Buffer<O>::new();

        // Clone arcs for thread
        let iclone = incoming.clone();
        let oclone = outgoing.clone();

        // Spawn thread
        let handle  = std::thread::spawn(
            move || Self::run(stream, iclone, oclone, read_first)
        );

        Self { incoming, outgoing, handle, }
    }

    pub fn run(
        stream: TcpStream,
        incoming: Buffer<I>,
        outgoing: Buffer<O>,
        read_first: bool, // Start with reading, otherwise write
    ) -> ThreadResult {
        
        if read_first {

            let msg = read<I>(&stream);
            incoming.set(msg); 
        }

        loop {

            let msg: O = loop {
                match outgoing.get() {
                    None => (),
                    Some(msg) => { break msg; },
                }
            };
            
            write<O>(&stream, &msg);

            let msg = read<I>(&stream);
            incoming.set(msg); 
        }
    }
}

pub fn write<T: Message>(stream: &mut TcpStream, msg: T) {

    serde_json::to_writer(stream, &msg).unwrap();
}

pub fn read<T: Message>(stream: &mut TcpStream) -> T {

    serde_json::from_reader(stream).unwrap()
}
