use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use lazy_static::lazy_static;

lazy_static! {
    static ref ROUTES: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
}
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Arc<Mutex<mpsc::Sender<Message>>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let sender = Arc::new(Mutex::new(sender));

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..(size) {
            workers.push(Worker::new(id, receiver.clone()));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        let job = Box::new(f);

        self.sender
            .lock()
            .unwrap()
            .send(Message::NewJob(job))
            .unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender
                .lock()
                .unwrap()
                .send(Message::Terminate)
                .unwrap();
        }
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} stopping...", id);

                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

pub enum Message {
    NewJob(Job),
    Terminate,
}
