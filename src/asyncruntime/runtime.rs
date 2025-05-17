use std::collections::VecDeque;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Wake, Waker};
use std::thread::{self, Thread};
use std::time::Duration;

use futures_timer::Delay;

type Job = Pin<Box<dyn Future<Output = ()>>>;

pub struct Runtime {
    spawner: Rc<Spawner>,
}

impl Runtime {
    pub fn run<F>(&self, f: F)
    where
        F: Future<Output = ()> + 'static,
    {
        let fut = Box::pin(f);

        self.spawner.spawn(fut);
        self.spawner.spawn(async {
            Delay::new(Duration::from_secs(10)).await;
            println!("spawn from run done")
        });

        let custom_waker: Waker = Arc::new(CustomWaker::new(thread::current())).into();

        let mut ctx = Context::from_waker(&custom_waker);

        let mut waiting_tasks: VecDeque<Job> = VecDeque::new();

        // self.spawn(fut);

        // loop {
        loop {
            let task = {
                let mut task_queue = self.spawner.jobs.lock().unwrap();
                task_queue.pop_front()
            };

            match task {
                Some(mut t) => match t.as_mut().poll(&mut ctx) {
                    Poll::Ready(_) => {
                        println!("task ready");
                    }
                    Poll::Pending => {
                        // println!("task pending");
                        waiting_tasks.push_back(t);
                        // thread::sleep(Duration::from_secs(5));
                        thread::park();
                    }
                },
                None => break,
            }

            while let Some(t) = waiting_tasks.pop_front() {
                self.spawner.jobs.lock().unwrap().push_back(t);
            }
            // }
            // while let Some(mut task) = self.spawner.jobs.lock().unwrap().pop_front() {
            // println!("task loop");
            // match task.as_mut().poll(&mut context) {
            //     Poll::Ready(_) => {
            //         println!("task ready");
            //     }
            //     Poll::Pending => {
            //         println!("task pending");
            //         waiting_tasks.push_back(task);
            //         thread::sleep(Duration::from_secs(5));
            //     }
            // }
            // // }
            // if let Some(t) = waiting_tasks.pop_front() {
            //     self.spawner.jobs.lock().unwrap().push_back(t);
            // }
            // println!("hi");
            // match fut.as_mut().poll(&mut context) {
            //     Poll::Ready(_) => {
            //         break 'outer;
            //     }
            //     Poll::Pending => {
            //         println!("hello from pending");
            //         thread::sleep(Duration::from_secs(5));
            //     }
            // }
        }
    }

    pub fn new(spawner: Rc<Spawner>) -> Self {
        Runtime { spawner }
    }
}

pub struct Spawner {
    jobs: Mutex<VecDeque<Job>>,
    jobs2: VecDeque<Job>,
}
impl Spawner {
    pub fn new() -> Self {
        Spawner {
            jobs: Mutex::new(VecDeque::new()),
            jobs2: VecDeque::new(),
        }
    }
    pub fn spawn<F>(&self, task: F)
    where
        F: Future<Output = ()> + 'static,
    {
        println!("spawn task");
        self.jobs.lock().unwrap().push_back(Box::pin(task));
        println!("spawn success")
    }

    pub fn spawn2<F>(&mut self, task: F)
    where
        F: Future<Output = ()> + 'static,
    {
        println!("spawn task");
        self.jobs2.push_back(Box::pin(task));
        println!("spawn success")
    }
}

struct CustomWaker {
    main_thread_handle: Thread,
}
impl CustomWaker {
    fn new(main_thread_handle: Thread) -> Self {
        CustomWaker { main_thread_handle }
    }
}
impl Wake for CustomWaker {
    fn wake(self: Arc<Self>) {
        println!("custom waker hello");
        self.main_thread_handle.unpark();
    }
}

pub fn create_waker(data: Arc<AtomicBool>) -> Waker {
    let raw_waker = RawWaker::new(Arc::into_raw(data.clone()) as *const (), &VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

fn create_vtable() -> RawWakerVTable {
    RawWakerVTable::new(
        |data| {
            println!("waker clone");
            RawWaker::new(data, &VTABLE)
        },
        |data| {
            println!("wake");
        },
        |data| {
            println!("wake by ref");
        },
        drop,
    )
}

const VTABLE: RawWakerVTable = RawWakerVTable::new(
    |data| {
        println!("waker clone");
        RawWaker::new(data, &VTABLE)
    },
    |data| {
        println!("wake");
    },
    |data| {
        println!("wake by ref");
    },
    drop,
);
