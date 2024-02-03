use std::io::BufReader;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::pin::Pin;

pub(crate) type Coroutine =
    Pin<Box<dyn std::ops::Coroutine<Yield = (), Return = ()>>>;

pub(crate) type TcpConnection = (BufReader<TcpStream>, SocketAddr);
