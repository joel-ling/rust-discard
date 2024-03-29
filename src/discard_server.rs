use crate::aliases::Coroutine;
use crate::aliases::TcpConnection;
use std::io::BufRead;
use std::io::BufReader;
use std::io::ErrorKind::WouldBlock;
use std::io::Result;
use std::net::TcpListener;
use std::ops::Coroutine as _;
use std::pin::pin;
use std::pin::Pin;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;

pub struct DiscardServer {
    tcp_listener: TcpListener,
}

impl DiscardServer {
    pub fn new(address: &str) -> Result<Self> {
        let tcp_listener = TcpListener::bind(address)?;

        tcp_listener.set_nonblocking(true)?;

        Ok(DiscardServer {
            tcp_listener: tcp_listener,
        })
    }

    pub fn run(self) {
        let (listener_reader_tx, listener_reader_rx): (
            Sender<TcpConnection>,
            Receiver<TcpConnection>,
        ) = channel();

        let (reader_handler_tx, _reader_handler_rx): (
            Sender<String>,
            Receiver<String>,
        ) = channel();

        let mut listener: Pin<&mut Coroutine> =
            pin!(DiscardServer::listener_coroutine(
                self.tcp_listener,
                listener_reader_tx.clone(),
            ));

        let mut reader: Pin<&mut Coroutine> =
            pin!(DiscardServer::reader_coroutine(
                listener_reader_rx,
                reader_handler_tx.clone(),
                listener_reader_tx.clone(),
            ));

        loop {
            listener.as_mut().resume(());
            reader.as_mut().resume(());
        }
    }

    fn listener_coroutine(
        tcp_listener: TcpListener,
        tx: Sender<TcpConnection>,
    ) -> Coroutine {
        let coroutine = move || loop {
            match tcp_listener.accept() {
                Ok((stream, client)) => {
                    println!("accepted connection from {client}");

                    stream.set_nonblocking(true).unwrap();

                    let buffer = BufReader::new(stream);

                    tx.send((buffer, client)).unwrap();
                }

                Err(ref error) if error.kind() == WouldBlock => {}

                Err(error) => panic!("{error}"),
            }

            yield;
        };

        Box::pin(coroutine)
    }

    fn reader_coroutine(
        rx: Receiver<TcpConnection>,
        tx_string: Sender<String>,
        tx: Sender<TcpConnection>,
    ) -> Coroutine {
        let coroutine = move || loop {
            let (mut buffer, client): TcpConnection;

            loop {
                match rx.try_recv() {
                    Ok(connection) => {
                        (buffer, client) = connection;

                        break;
                    }

                    Err(error) => match error {
                        TryRecvError::Empty => yield,

                        TryRecvError::Disconnected => return,
                    },
                };
            }

            let mut string = String::new();
            // XXX: assumes UTF-8-encoded payload

            match buffer.read_line(&mut string) {
                Ok(bytecount) => {
                    if bytecount != 0 {
                        string = string.trim().to_string();

                        println!(
                            "Received {bytecount} bytes from {client}: {string}"
                        );

                        tx_string.send(string).unwrap();
                    }
                }

                Err(ref error) if error.kind() == WouldBlock => {}

                Err(error) => panic!("{error}"),
            }

            tx.send((buffer, client)).unwrap();

            yield;
        };

        Box::pin(coroutine)
    }
}

#[cfg(test)]
mod tests {
    use super::DiscardServer;
    use std::io::Write;
    use std::net::TcpStream;
    use std::thread;

    #[test]
    fn test_discard_server() {
        const N_CLIENTS: u8 = u8::MAX;

        const ADDRESS: &str = "127.6.8.3:5009";

        let server: DiscardServer = match DiscardServer::new(ADDRESS) {
            Ok(server) => server,

            Err(error) => panic!("{error}"),
        };

        thread::spawn(move || server.run());

        for _ in 0..N_CLIENTS {
            let mut client: TcpStream;

            loop {
                match TcpStream::connect(ADDRESS) {
                    Ok(stream) => {
                        client = stream;

                        break;
                    }

                    Err(_) => {}
                }
            }

            client.write_fmt(format_args!("HELO server\n")).unwrap();

            client.write_fmt(format_args!("QUIT\n")).unwrap();
        }
    }
}
