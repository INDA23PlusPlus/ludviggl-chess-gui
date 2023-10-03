
use std::net::TcpStream;
use serde::{ Serialize, Deserialize, };
use serde_json;

pub fn send<T: Serialize>(stream: &mut TcpStream, msg: T) -> serde_json::Result<()> {

    serde_json::to_writer(&stream, &msg)
}

pub fn recieve<T: Deserialize>(stream: &mut TcpStream) -> serde_json::Result<T> {

    serde_json::from_reader(&stream)
}
