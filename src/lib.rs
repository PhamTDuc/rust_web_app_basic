use std::{thread, sync::{mpsc, Mutex, Arc}};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let mut workers = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));                

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        return ThreadPool { workers, sender:Some(sender) };
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().expect("Sender not valid").send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take(){
                thread.join().unwrap();
                println!("Shutting down worker {} COMPLETED", worker.id);
            }
        }
        println!("Shutting down server COMPLETED.");
    }
}
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop{
           let message = receiver.lock().unwrap().recv();
           match message{
            Ok(job)=>{
               println!("Worker {id} got a job, executing...");
               job();  
            }
            Err(error) => {
                eprintln!("Worker {id} got an error...\r\nERROR: {error}");
                break;
            }  
           }
        });
        Worker {
            id,
            thread:Some(thread),
        }
    }
}
