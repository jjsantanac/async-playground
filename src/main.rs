use hello::asyncruntime::{
    async_tcp_listener::AsyncTcpListener,
    async_tcp_stream::{interrupted, would_block},
    runtime::Runtime,
    task_spawner::Spawner,
};
use mio::{Events, Poll, Token, net::TcpStream};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

fn main() {
    // let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    // let pool = ThreadPool::new(4);

    let poll = Arc::new(Mutex::new(Poll::new().unwrap()));
    let events = Arc::new(Mutex::new(Events::with_capacity(128)));

    let spawner = Arc::new(Spawner::new());
    let runtime = Rc::new(Runtime::new(
        Arc::clone(&spawner),
        Arc::clone(&poll),
        Arc::clone(&events),
    ));

    let client_token = Arc::new(Mutex::new(Token(1)));

    runtime.run(async move {
        let mut listener = AsyncTcpListener::new(poll, events, Arc::clone(&client_token));
        listener.bind("127.0.0.1:13265");

        loop {
            // let r = listener.accept().await.unwrap();
            let stream = listener.accept().await;

            let mut buf: Vec<u8> = vec![];

            // TODO read fut
            match stream.stream.lock().unwrap().read(&mut buf) {
                Ok(n) if n < buf.len() => {
                    println!("writezero");
                    break;
                }
                Ok(n) => {
                    println!("read {n} bytes");
                    println!("buf: {buf:?}")
                }
                Err(ref err) if would_block(err) => {
                    println!("would block")
                }
                // Got interrupted (how rude!), we'll try again.
                Err(ref err) if interrupted(err) => {
                    println!("interrupted")
                }
                Err(_) => {
                    panic!("very bad!")
                }
            };

            println!("connection accepted...");
            // Delay::new(Duration::from_secs(14)).await;

            spawner.spawn(async move {
                // Delay::new(Duration::from_secs(5)).await;

                let (status_line, filename) = ("HTTP/1.1 200 OK", "hello.html");

                let contents = fs::read_to_string(filename).unwrap();
                let length = contents.len();

                println!("creating response..");

                let response =
                    format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

                stream.write(response).await.unwrap();

                // Delay::new(Duration::from_secs(5)).await;
                // println!("spawn from async block done")
            });
        }
        // println!("spawn from async block");
        // spawner.spawn(async {
        //     Delay::new(Duration::from_secs(14)).await;
        //     println!("spawn from async block done")
        // });

        // spawner.spawn(async {
        //     Delay::new(Duration::from_secs(21)).await;
        //     println!("subtask from 2 done");
        //     Delay::new(Duration::from_secs(25)).await;
        //     println!("spawn from async block 2 done")
        // });

        // Delay::new(Duration::from_secs(7)).await;
        // println!("main block done")
    });

    // for stream in listener.incoming().take(2) {
    //     let stream = stream.unwrap();

    //     pool.run(|| handle_connection(stream));
    // }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);

    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}
