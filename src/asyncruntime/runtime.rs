use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::thread::{self, Thread};

use super::simple_waker::CustomWaker;
use super::task_spawner::Spawner;

pub type Job = Pin<Box<dyn Future<Output = ()> + Send>>;

pub struct Runtime {
    spawner: Arc<Spawner>,
}

impl Runtime {
    pub fn run<F>(&self, f: F)
    where
        F: Future<Output = ()> + 'static + Send,
    {
        let main_fut = Box::pin(f);

        self.spawner.spawn(main_fut);

        // TODO think of something better than incrementing id here
        // TODO think about who should manage both queues
        let mut id = 0;

        loop {
            let task = {
                let mut task_queue = self.spawner.jobs.lock().unwrap();
                if task_queue.is_empty() && self.spawner.pending_jobs.lock().unwrap().is_empty() {
                    break;
                }

                task_queue.pop_front()
            };

            let t = match task {
                Some(task) => task,
                None => {
                    println!("no tasks ready, going to sleep");
                    thread::park();
                    continue;
                }
            };

            let waker: Waker = Arc::new(CustomWaker::new(
                thread::current(),
                Arc::clone(&t),
                Arc::clone(&self.spawner.jobs),
                id,
                Arc::clone(&self.spawner.pending_jobs),
            ))
            .into();

            let mut ctx = Context::from_waker(&waker);

            match t.lock().unwrap().as_mut().poll(&mut ctx) {
                Poll::Ready(_) => println!("task ready!"),
                Poll::Pending => {
                    println!("Task pending..!");
                    self.spawner.queue_pending(Arc::clone(&t), id);
                    id += 1;
                    println!("task {id:?} inserted into pending");
                }
            }
        }
    }

    pub fn new(spawner: Arc<Spawner>) -> Self {
        Runtime { spawner }
    }
}

pub fn create_waker(data: Thread) -> Waker {
    // let raw_waker = RawWaker::new(Arc::into_raw(data.clone()) as *const (), &VTABLE);
    let boxed = Box::new(data);
    let raw_waker = RawWaker::new(Box::into_raw(boxed) as *const (), &VTABLE);
    // let raw_waker = RawWaker::new(&data as *const Thread as *const (), &VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

const VTABLE: RawWakerVTable = RawWakerVTable::new(
    |data| {
        println!("waker clone");
        RawWaker::new(data, &VTABLE)
    },
    |data| {
        println!("wake");
        let d = unsafe { &*(data as *const Thread) };
        // unsafe {
        //     let a = &*d;
        //     a.unpark();
        // }
    },
    |data| {
        println!("wake by ref");
    },
    drop,
);
