type Job = Box<dyn FnOnce() + Send + 'static>;

use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver
                    .lock()       // acquire the mutex
                    .unwrap()     // panic if the mutex is poisoned
                    .recv();       // wait for a job (blocks here)

                match message {
                    Ok(job) => {
                        job();       // run the job
                    }
                    Err(_) => {
                        break;       // channel closed → shut down
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}