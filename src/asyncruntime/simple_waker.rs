use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    task::Wake,
    thread::Thread,
};

use super::runtime::Job;

pub struct CustomWaker {
    main_thread_handle: Thread,
    job: Arc<Mutex<Job>>,
    task_queue: Arc<Mutex<VecDeque<Arc<Mutex<Job>>>>>,
    id: usize,
    pending_jobs: Arc<Mutex<HashMap<usize, Arc<Mutex<Job>>>>>,
}
impl CustomWaker {
    pub fn new(
        main_thread_handle: Thread,
        job: Arc<Mutex<Job>>,
        task_queue: Arc<Mutex<VecDeque<Arc<Mutex<Job>>>>>,
        id: usize,
        pending_jobs: Arc<Mutex<HashMap<usize, Arc<Mutex<Job>>>>>,
    ) -> Self {
        CustomWaker {
            main_thread_handle,
            job,
            task_queue,
            id,
            pending_jobs,
        }
    }
}
impl Wake for CustomWaker {
    fn wake(self: Arc<Self>) {
        println!("custom waker hello");
        self.task_queue
            .lock()
            .unwrap()
            .push_back(Arc::clone(&self.job));
        println!("task {:?} inserted to ready queue", &self.id);
        self.pending_jobs.lock().unwrap().remove(&self.id);
        println!("task {:?} removed from pending queue", &self.id);
        self.main_thread_handle.unpark();
    }
}
