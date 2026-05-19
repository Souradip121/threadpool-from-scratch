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

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        // Create the channel: sender stays here, receiver goes to workers
        let (sender, receiver) = mpsc::channel();

        // Wrap receiver so all workers can share it safely
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            // Arc::clone does NOT clone the data —
            // it just increments the reference count
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender
            .as_ref()
            .unwrap()
            .send(job)
            .unwrap();
    }
}


fn main() {
    let pool = ThreadPool::new(4);

    for i in 0..8 {
        pool.execute(move || {
            println!("Job {} running on thread {:?}", i,
                     thread::current().id());
        });
    }

    thread::sleep(std::time::Duration::from_secs(1));
}
