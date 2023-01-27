use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex};
use super::priority_async_channel;

type Job = Box<dyn FnOnce() + 'static + Send>;
enum Message {
    ByeBye,
    NewJob(Job),
}

struct Worker where
{
    _id: usize,
    t: Option<JoinHandle<()>>,
}

impl Worker
{
    fn new(id: usize, receiver: Arc::<Mutex<priority_async_channel::Receiver<Message>>>) -> Worker {
        let t = thread::spawn( move || {
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();
                match message {
                    Message::NewJob(job) => {
                        println!("do job from worker[{}]", id);
                        job();
                    },
                    Message::ByeBye => {
                        println!("ByeBye from worker[{}]", id);
                        break
                    },
                }  
            }
        });

        Worker {
            _id: id,
            t: Some(t),
        }
    }
}

pub struct Pool {
    workers: Vec<Worker>,
    max_workers: usize,
    sender:priority_async_channel::Sender<Message>,
}

impl Pool where {
    pub fn new(max_workers: usize) -> Pool {
        if max_workers == 0 {
            panic!("max_workers must be greater than zero!")
        }
        let (tx, rx) = priority_async_channel::async_channel();
        // mpsc::channel();

        let mut workers = Vec::with_capacity(max_workers);
        let receiver = Arc::new(Mutex::new(rx));
        for i in 0..max_workers {
            workers.push(Worker::new(i, Arc::clone(&receiver)));
        }

        Pool { workers: workers, max_workers: max_workers, sender: tx }
    }
    
    pub fn execute<F>(&mut self, f:F,high_priority:bool) where F: FnOnce() + 'static + Send
    {
        let job = Message::NewJob(Box::new(f));

        self.sender.send(job,high_priority).unwrap();
        
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        for _ in 0..self.max_workers {
            self.sender.send(Message::ByeBye,false).unwrap();
        }
        for w in &mut self.workers {
            if let Some(t) = w.t.take() {
                t.join().unwrap();
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut p = Pool::new(4);
        p.execute(|| {println!("do new job1");  },false);
        p.execute(|| {println!("do new job2");  },false);
        p.execute(||{println!("do new job3");  },false);
        p.execute(|| {println!("do new job4"); },false);
        thread::sleep_ms(2000);
        p.execute(|| println!("do new job5"),false);
        p.execute(|| {println!("do new job6"); thread::sleep_ms(1000); },true);
        p.execute(|| {println!("do new job7"); thread::sleep_ms(1000); },true);
        p.execute(||{println!("do new job8"); thread::sleep_ms(1000); },true);
        p.execute(|| {println!("do new job9"); thread::sleep_ms(1000); },true);
    }
}
