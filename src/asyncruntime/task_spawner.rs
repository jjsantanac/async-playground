use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use super::runtime::Job;

#[derive(Default)]
pub struct Spawner {
    pub jobs: Arc<Mutex<VecDeque<Arc<Mutex<Job>>>>>,
    pub pending_jobs: Arc<Mutex<HashMap<usize, Arc<Mutex<Job>>>>>,
}
impl Spawner {
    pub fn new() -> Self {
        Spawner {
            jobs: Arc::new(Mutex::new(VecDeque::new())),
            pending_jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub fn spawn<F>(&self, task: F)
    where
        F: Future<Output = ()> + 'static + Send,
    {
        self.jobs
            .lock()
            .unwrap()
            .push_back(Arc::new(Mutex::new(Box::pin(task))));
    }

    pub fn queue_pending(&self, task: Arc<Mutex<Job>>, id: usize) {
        self.pending_jobs.lock().unwrap().insert(id, task);
    }
}
