use std::{
    cell::RefCell,
    collections::VecDeque,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use super::runtime::Job;

pub struct Task {
    pub task_queue: Rc<RefCell<VecDeque<Job>>>,
    task: Job,
    ready: bool,
}
impl Task {
    pub fn new(task_queue: Rc<RefCell<VecDeque<Job>>>, task: Job) -> Self {
        Task {
            task_queue,
            task,
            ready: false,
        }
    }
}
impl Future for Task {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.ready {
            return Poll::Ready(());
        }

        match self.task.as_mut().poll(cx) {
            Poll::Ready(_) => {}
            Poll::Pending => todo!(),
        }
        Poll::Pending
    }
}
