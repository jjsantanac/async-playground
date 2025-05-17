use std::{
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    pool: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}
impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let mut pool: Vec<Worker> = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            pool.push(Worker::new(id, receiver.clone()));
        }

        ThreadPool {
            pool,
            sender: Some(sender),
        }
    }

    pub fn run(&self, job: impl FnOnce() + Send + 'static) {
        self.sender.as_ref().unwrap().send(Box::new(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in self.pool.drain(..) {
            worker.thread_handle.join().unwrap();
        }
    }
}

struct Worker {
    thread_handle: JoinHandle<()>,
    id: usize,
}
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread_handle = thread::spawn(move || {
            loop {
                println!("{id} waiting for work");
                let message = receiver.lock().unwrap().recv();
                match message {
                    Ok(job) => {
                        job();
                    }
                    Err(_) => {
                        println!("shutting down worker {id}");
                        break;
                    }
                }
            }
        });

        Worker { id, thread_handle }
    }
}
