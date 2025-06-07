use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    task::Poll,
    thread,
    time::Duration,
};

use mio::{Events, Interest, Token, net::TcpStream};

pub struct AsyncTcpStream {
    poll: Arc<Mutex<mio::Poll>>,
    events: Arc<Mutex<Events>>,
    pub stream: Arc<Mutex<TcpStream>>,
    client_token: Arc<Mutex<Token>>,
}
impl AsyncTcpStream {
    pub fn new(
        poll: Arc<Mutex<mio::Poll>>,
        events: Arc<Mutex<Events>>,
        stream: Arc<Mutex<TcpStream>>,
        client_token: Arc<Mutex<Token>>,
    ) -> Self {
        AsyncTcpStream {
            poll,
            events,
            stream,
            client_token,
        }
    }

    pub fn read(self) {
        self.poll
            .lock()
            .unwrap()
            .registry()
            .register(
                &mut *self.stream.lock().unwrap(),
                *self.client_token.lock().unwrap(),
                Interest::READABLE,
            )
            .unwrap();

        // ReadFuture
    }

    pub fn write(self, response: String) -> WriteFuture {
        self.poll
            .lock()
            .unwrap()
            .registry()
            .register(
                &mut *self.stream.lock().unwrap(),
                *self.client_token.lock().unwrap(),
                // Interest::READABLE.add(Interest::WRITABLE),
                Interest::WRITABLE,
            )
            .unwrap();

        WriteFuture {
            poll: Arc::clone(&self.poll),
            events: Arc::clone(&self.events),
            response,
            stream: Arc::clone(&self.stream),
            client_token: Arc::clone(&self.client_token),
        }
    }
}

pub struct WriteFuture {
    poll: Arc<Mutex<mio::Poll>>,
    events: Arc<Mutex<Events>>,
    response: String,
    stream: Arc<Mutex<TcpStream>>,
    client_token: Arc<Mutex<Token>>,
}

impl Future for WriteFuture {
    type Output = io::Result<()>;

    // TODO
    // writefuture has to poll mio for writable event
    // need to use write and not write_all
    // while response is not fully written to socket -> Pending

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut events = self.events.lock().unwrap();

        self.poll
            .lock()
            .unwrap()
            .poll(&mut events, Some(Duration::from_millis(10)))
            .unwrap();

        let poll_ref = Arc::clone(&self.poll);
        let event_ref = Arc::clone(&self.events);
        let client_token_ref = Arc::clone(&self.client_token);

        let data = self.response.as_bytes();

        let data_len = data.len();

        match self.stream.lock().unwrap().write(self.response.as_bytes()) {
            // We want to write the entire `DATA` buffer in a single go. If we
            // write less we'll return a short write error (same as
            // `io::Write::write_all` does).
            Ok(n) if n < data.len() => {
                println!("writezero");
                return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
            }
            Ok(n) => {
                // After we've written something we'll reregister the connection
                // to only respond to readable events.
                println!("written {n} bytes");
                println!("response has {data_len} bytes");

                if n == data_len {
                    return Poll::Ready(Ok(()));
                }
                self.poll.lock().unwrap().registry().reregister(
                    &mut *self.stream.lock().unwrap(),
                    *self.client_token.lock().unwrap(),
                    Interest::READABLE,
                )?
            }
            // Would block "errors" are the OS's way of saying that the
            // connection is not actually ready to perform this I/O operation.
            Err(ref err) if would_block(err) => {
                println!("would block")
            }
            // Got interrupted (how rude!), we'll try again.
            Err(ref err) if interrupted(err) => {
                println!("interrupted")
            }
            // Other errors we'll consider fatal.
            Err(err) => return Poll::Ready(Err(err)),
        }

        thread::spawn(move || {
            loop {
                poll_ref
                    .lock()
                    .unwrap()
                    .poll(&mut event_ref.lock().unwrap(), None)
                    .unwrap();

                for event in event_ref.lock().unwrap().iter() {
                    if event.token() == *client_token_ref.lock().unwrap() {
                        println!("client socket ready");
                    }
                }
            }
        });

        Poll::Pending
    }
}

pub fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

pub fn interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}

pub struct ReadFuture {
    poll: Arc<Mutex<mio::Poll>>,
    events: Arc<Mutex<Events>>,
    stream: Arc<Mutex<TcpStream>>,
    client_token: Arc<Mutex<Token>>,
}

impl Future for ReadFuture {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}
