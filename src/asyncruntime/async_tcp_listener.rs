use mio::{Events, Interest, Token, net::TcpListener};
use std::{
    sync::{Arc, Mutex, atomic::AtomicBool},
    task::Poll,
    thread,
    time::Duration,
};

use super::async_tcp_stream::AsyncTcpStream;

const SERVER: Token = Token(0);

pub struct AsyncTcpListener {
    poll: Arc<Mutex<mio::Poll>>,
    events: Arc<Mutex<Events>>,
    server: Option<TcpListener>,
    client_token: Arc<Mutex<Token>>,
}
impl AsyncTcpListener {
    pub fn new(
        poll: Arc<Mutex<mio::Poll>>,
        events: Arc<Mutex<Events>>,
        client_token: Arc<Mutex<Token>>,
    ) -> Self {
        AsyncTcpListener {
            poll,
            events,
            server: None,
            client_token,
        }
    }

    pub fn bind(&mut self, addr: &str) {
        let mut server = TcpListener::bind(addr.parse().unwrap()).unwrap();

        self.poll
            .lock()
            .unwrap()
            .registry()
            .register(&mut server, SERVER, Interest::READABLE)
            .unwrap();

        self.server = Some(server);
    }

    pub fn accept(&self) -> AcceptFuture {
        let token = Token(self.client_token.lock().unwrap().0);
        self.client_token.lock().unwrap().0 += 1;
        AcceptFuture {
            poll: Arc::clone(&self.poll),
            events: Arc::clone(&self.events),
            stream: self.server.as_ref(),
            socket_ready: Arc::new(AtomicBool::new(false)),
            client_token: Arc::new(Mutex::new(token)),
        }
    }
}

fn next(current: &mut Token) -> Token {
    let next = current.0;
    current.0 += 1;
    Token(next)
}

pub struct AcceptFuture<'a> {
    poll: Arc<Mutex<mio::Poll>>,
    events: Arc<Mutex<Events>>,
    stream: Option<&'a TcpListener>,
    socket_ready: Arc<AtomicBool>,
    client_token: Arc<Mutex<Token>>,
}

impl<'a> Future for AcceptFuture<'a> {
    // type Output = Option<&'a TcpListener>;
    type Output = AsyncTcpStream;

    // executor needs to poll mio poll
    // when token is available future can advance and accept connection
    // executor needs to wake task

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.socket_ready.load(std::sync::atomic::Ordering::SeqCst) {
            let s = self.stream.unwrap().accept().unwrap().0;
            let a_s = AsyncTcpStream::new(
                Arc::clone(&self.poll),
                Arc::clone(&self.events),
                Arc::new(Mutex::new(s)),
                Arc::clone(&self.client_token),
            );
            // return Poll:Ready(AsyncTcpStream{})
            return Poll::Ready(a_s);
        }

        // let mut events = self.events.lock().unwrap();

        // self.poll
        //     .lock()
        //     .unwrap()
        //     .poll(&mut events, Some(Duration::from_millis(10)))
        //     .unwrap();

        let waker = cx.waker().clone();

        let poll_ref = Arc::clone(&self.poll);
        let event_ref = Arc::clone(&self.events);
        let socket_ready_ref = Arc::clone(&self.socket_ready);

        thread::spawn(move || {
            'main: loop {
                poll_ref
                    .lock()
                    .unwrap()
                    .poll(
                        &mut event_ref.lock().unwrap(),
                        Some(Duration::from_millis(10)),
                    )
                    .unwrap();

                // println!("events empty{}", event_ref.lock().unwrap().is_empty());

                for event in event_ref.lock().unwrap().iter() {
                    println!("is readable: {}", event.is_readable());

                    if let SERVER = event.token() {
                        println!("socket ready");
                        socket_ready_ref.store(true, std::sync::atomic::Ordering::SeqCst);
                        waker.wake_by_ref();
                        break 'main;
                    }
                }
                // thread::sleep(Duration::from_millis(5000));
            }
        });
        Poll::Pending
    }
}
