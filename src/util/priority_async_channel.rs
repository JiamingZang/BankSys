use anyhow::{anyhow, Result};
use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Condvar, Mutex,
    },
};

pub struct Shared<T> {
    queue: Mutex<VecDeque<T>>,
    available: Condvar,
    senders_num: AtomicUsize,
    receivers_num: AtomicUsize,
}

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Sender<T> {
    pub fn send(&mut self, t: T,high_priority:bool) -> Result<()> {
        //如果没有接收者直接返回错误
        if self.get_receivers_num() == 0 {return Err(anyhow!("no receiver"));}

        //检查消息队列在push前是否为空，然后再push消息
        let was_empty = {
            let mut inner = self.shared.queue.lock().unwrap();
            let empty = inner.is_empty();

            //实现优先级消息队列
            if !high_priority{
                inner.push_back(t);
            }else if high_priority{
                inner.push_front(t);
            }

            empty
        };

        //如果消息队列在push前为空，可能有接收者线程阻塞，使用condvar通知
        if was_empty {
            self.shared.available.notify_one();
        }

        Ok(())
    }

    pub fn get_receivers_num(&self) -> usize {
        //Ordering::SeqCst,严格内存序保证多个接收者线程能读到最新值
        self.shared.receivers_num.load(Ordering::SeqCst)
    }

    pub fn get_queued_items(&self) -> usize {
        //访问共享数据结构之前先加锁
        let inner = self.shared.queue.lock().unwrap();
        inner.len()
    }
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Result<T> {
        let mut inner = self.shared.queue.lock().unwrap();
        loop {
            match inner.pop_front() {
                //队列存在消息。直接返回消息
                Some(v) => {return Ok(v)},
                //队列没有消息且发送者都已经drop，返回错误
                None if self.get_senders_num() == 0 => {
                    return Err(anyhow!("no sender!"));
                },
                //队列没有消息还存在发送者，阻塞线程
                //wait()释放锁并挂起线程，等收到notify再拿回锁，重新初始化inner
                None => {
                    inner = self.shared
                    .available
                    .wait(inner)
                    .map_err(|_| anyhow!("lock error"))?;
                }
            }
        }
    }

    pub fn get_senders_num(&self) -> usize {
        self.shared.senders_num.load(Ordering::SeqCst)
    }
}

impl<T> Iterator for Receiver<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.recv().ok()
    }
}

//mpsc channel 只有发送者需要实现Clone 
impl<T> Clone for Sender<T> {
    //克隆方法只需要增加引用计数即可
    fn clone(&self) -> Self {
        self.shared.senders_num.fetch_add(1, Ordering::AcqRel);
        Self {
            shared: self.shared.clone(),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let old = self.shared.senders_num.fetch_sub(1, Ordering::AcqRel);
        //防止接收者线程阻塞时，发送者线程都结束，无法唤醒接收者线程
        if old <= 1 {
            self.shared.available.notify_all();
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.shared.receivers_num.fetch_sub(1, Ordering::SeqCst);
    }
}

pub fn async_channel<T>() -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(Shared::default());

    (
        Sender {shared: shared.clone()},
        Receiver {shared},
    )
}

const INIT_SIZE: usize = 32;
impl<T> Default for Shared<T> {
    fn default() -> Self {
        Self {
            queue: Mutex::new(VecDeque::with_capacity(INIT_SIZE)),
            available: Condvar::new(),
            senders_num: AtomicUsize::new(1),
            receivers_num: AtomicUsize::new(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};
    use super::*;

    #[test]
    fn channel_should_works() {
        let (mut s, mut r) = async_channel();
        s.send(1,false).unwrap();

        let msg = r.recv().unwrap();
        assert_eq!(1, msg);
    }

    #[test]
    fn multi_sender_should_work() {
        let (mut s, mut r) = async_channel();
        let mut v = vec![];
        for i in 0..2 {
            let mut ts = s.clone();
            thread::spawn(move || {
                ts.send(i,false).unwrap();
            })
            .join()
            .unwrap();
        }
        s.send(2,false).unwrap();
        //如果不drop，会出现接收者线程持续阻塞的情况。
        drop(s);

        while let Ok(res) = r.recv() {
            v.push(res);
        }
        
        v.sort();
        assert_eq!(v, [0, 1, 2]);
    }

    #[test]
    fn all_sender_drop_should_error_when_receive() {
        let (s, mut r) = async_channel();
        let s1 = s.clone();

        let senders = [s, s1];
        let senders_num = senders.len();

        for mut sender in senders {
            thread::spawn(move || {
                sender.send("hello",false).unwrap();
            })
            .join()
            .unwrap();
        }

        for _ in 0..senders_num {
            r.recv().unwrap();
        }

        //接收完所有在缓冲队列中的数据时，继续接收会报错
        assert!(r.recv().is_err());
    }

    #[test]
    fn receiver_should_be_blocked_when_queue_empty() {
        let (s, r) = async_channel();
        let mut s1 = s.clone();
        let mut s2 = s.clone();
        thread::spawn(move || {
            for (idx, i) in r.into_iter().enumerate() {
                assert_eq!(idx, i);
            }
            //如果线程阻塞则无法执行到该步骤
            assert!(false);
        });

        thread::spawn(move || {
            for i in 0..100usize {
                s1.send(i,false).unwrap();
            }
        });
        thread::sleep(Duration::from_millis(1));

        thread::spawn(move || {
            for i in 100..200usize {
                s2.send(i,false).unwrap();
            }
        });
        thread::sleep(Duration::from_millis(1));

        //已经接收完所有的消息，任务队列为空
        assert_eq!(s.get_queued_items(), 0);
    } 

    #[test]
    fn receiver_drop_should_error_when_send() {
        let (mut s, _) = async_channel();
        let mut s1 = s.clone();

        assert!(s.send(0,false).is_err());
        assert!(s1.send(1,false).is_err());
    }

    #[test]
    fn all_sender_drop_when_receiver_block_should_work() {
        let (mut s, mut r) = async_channel();
        let mut v = 0;
        
        thread::spawn(move || {
            s.send(1,false).unwrap();
            //使得接收者线程在阻塞时drop；
            thread::sleep(Duration::from_millis(100));
        });

        while let Ok(res) = r.recv() {
            v = res;
        }
        
        assert_eq!(v, 1);
    }
}

