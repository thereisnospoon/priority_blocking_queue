use std::collections::BinaryHeap;
use std::sync::{Mutex, Condvar, RwLock};

#[derive(Debug)]
pub struct PriorityBlockingQueue<T> {
    elements: RwLock<BinaryHeap<T>>,
    non_empty: (Mutex<bool>, Condvar),
    max_capacity: usize,
}

impl<T: Ord> PriorityBlockingQueue<T> {
    pub fn new(max_capacity: usize) -> PriorityBlockingQueue<T> {
        PriorityBlockingQueue {
            non_empty: (Mutex::new(false), Condvar::new()),
            max_capacity,
            elements: RwLock::new(BinaryHeap::with_capacity(max_capacity)),
        }
    }

    pub fn len(&self) -> usize {
        self.elements.read().unwrap().len()
    }

    pub fn push(&self, t: T) -> Result<(), Error> {
        let mut elements = self.elements.write().unwrap();
        if elements.len() >= self.max_capacity {
            Err(Error::QueueCapacityReached)
        } else {
            elements.push(t);
            self.notify_waiters_for_push();
            Ok(())
        }
    }

    fn notify_waiters_for_push(&self) {
        let (mutex, non_empty_cond_var) = &self.non_empty;
        let mut mutex_guard = mutex.lock().unwrap();
        println!("Notifying on push");
        *mutex_guard = true;
        non_empty_cond_var.notify_one();
    }

    fn wait_non_empty(&self) {
        println!("Waiting until non-empty");
        let (mutex, non_empty_cond_var) = &self.non_empty;
        let mutex_guard = mutex.lock().unwrap();
        non_empty_cond_var.wait_while(mutex_guard, |non_empty| { !*non_empty });
    }

    pub fn pop(&self) -> T {
        self.wait_non_empty();
        let mut elements = self.elements.write().unwrap();
        elements.pop().unwrap()
    }
}

pub enum Error {
    QueueCapacityReached
}


#[cfg(test)]
mod tests {
    use std::thread;
    use crate::PriorityBlockingQueue;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    #[test]
    fn it_should_create_new_queue() {
        let q = PriorityBlockingQueue::<i32>::new(10);
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn it_should_push_and_pop_elements_from_the_queue() {
        let q = PriorityBlockingQueue::new(10);
        q.push(3);
        q.push(4);
        q.push(2);

        assert_eq!(q.pop(), 4);
        assert_eq!(q.pop(), 3);
        assert_eq!(q.pop(), 2);
    }

    #[test]
    fn it_should_block_on_pop_from_empty_queue() {
        let q = Arc::new(PriorityBlockingQueue::new(10));
        let q_clone = Arc::clone(&q);
        let start = Instant::now();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(3));
            q_clone.push(1);
        });
        let popped = q.pop();
        let elapsed = start.elapsed().as_secs();
        assert_eq!(popped, 1);
        assert!(elapsed >= 2);
    }

    #[test]
    fn it_should_return_error_on_push_when_capcity_reached() {
        let q = PriorityBlockingQueue::new(2);
        assert!(q.push(1).is_ok());
        assert!(q.push(2).is_ok());
        assert!(q.push(3).is_err());
        assert_eq!(q.pop(), 2);
        assert!(q.push(3).is_ok());
    }

    #[test]
    fn it_should_handle_boxed_values() {
        let q = PriorityBlockingQueue::new(10);
        q.push(Box::new(2));
        q.push(Box::new(1));
        q.push(Box::new(3));
        assert_eq!(*q.pop(), 3);
        assert_eq!(*q.pop(), 2);
        assert_eq!(*q.pop(), 1);
    }
}
