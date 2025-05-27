use futures_timer::Delay;
use hello::asyncruntime::{
    runtime::{self, Runtime},
    task_spawner::Spawner,
};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
    time::Duration,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    // let pool = ThreadPool::new(4);

    let spawner = Arc::new(Spawner::new());
    let runtime = Arc::new(Runtime::new(Arc::clone(&spawner)));

    runtime.run(async move {
        println!("spawn from async block");
        spawner.spawn(async {
            Delay::new(Duration::from_secs(14)).await;
            println!("spawn from async block done")
        });

        spawner.spawn(async {
            Delay::new(Duration::from_secs(21)).await;
            // println!("subtask from 2 done");
            //Delay::new(Duration::from_secs(25)).await;
            println!("spawn from async block 2 done")
        });

        Delay::new(Duration::from_secs(7)).await;
        println!("main block done")
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
