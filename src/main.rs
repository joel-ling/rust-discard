#![feature(coroutine_trait)]
#![feature(coroutines)]

use std::boxed::Box;
use std::io::ErrorKind::WouldBlock;
use std::io::Result;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::ops::Coroutine;
use std::pin::pin;
use std::pin::Pin;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

type TcpConnection = (TcpStream, SocketAddr);

type PinnedBoxedCoroutine = Pin<Box<dyn Coroutine<Yield = (), Return = ()>>>;
// WHY: https://doc.rust-lang.org/std/pin/struct.Pin.html#method.resume-1

fn new_tcp_listener_coroutine(
    tx: Sender<TcpConnection>,
    address: &str,
) -> Result<PinnedBoxedCoroutine> {
    let listener: TcpListener = TcpListener::bind(address)?;

    listener.set_nonblocking(true)?;

    let coroutine = move || loop {
        match listener.accept() {
            Ok(connection) => tx.send(connection).unwrap(),
            Err(ref error) if error.kind() == WouldBlock => {}
            Err(error) => panic!("{error}"),
        }

        yield;
    };

    Ok(Box::pin(coroutine))
}

fn main() {
    let (tx, _rx): (Sender<TcpConnection>, Receiver<TcpConnection>) = channel();

    let listener: PinnedBoxedCoroutine =
        new_tcp_listener_coroutine(tx, "localhost:2509").unwrap();

    let mut pinned_listener: Pin<&mut PinnedBoxedCoroutine> = pin!(listener);
    // WHY: https://doc.rust-lang.org/std/pin/macro.pin.html#remarks

    loop {
        pinned_listener.as_mut().resume(());
    }
}
