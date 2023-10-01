use std::{
    cmp,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Condvar, Mutex,
    },
};

use log::info;

const MUTEX: Mutex<()> = Mutex::new(());

#[derive(Debug)]
pub struct Rb {
    pub buf: Vec<f32>,
    pub write: AtomicUsize,
    pub read: AtomicUsize,

    pub slots_requested: AtomicUsize,

    pub block: Condvar,
    pub can_write: AtomicBool,
}

impl Rb {
    pub fn new(size: usize) -> Self {
        Self {
            buf: vec![0.0; size + 1],
            write: AtomicUsize::new(0),
            read: AtomicUsize::new(0),
            slots_requested: AtomicUsize::new(0),
            block: Condvar::new(),
            can_write: AtomicBool::new(true),
        }
    }
}

impl Rb {
    pub fn append(&mut self, slice: &[f32]) {
        if slice.is_empty() {
            return;
        }
        if slice.len() > self.buf.len() {
            todo!("Allow growing");
        }

        let write = self.write.load(Ordering::Relaxed);
        let read = self.read.load(Ordering::Relaxed);

        let slots_free = if write < read {
            read - write - 1
        } else {
            (self.buf.len()) - write + read
        };

        //Buffer is full.
        // if (write + 1) % (N + 1) == read {
        //     drop(self.block.wait(MUTEX.lock().unwrap()).unwrap());
        // }

        //Not enough space free.
        if slice.len() > slots_free {
            // info!("Blocking until {} are free.", slice.len());
            self.slots_requested.store(slice.len(), Ordering::Relaxed);
            drop(self.block.wait(MUTEX.lock().unwrap()).unwrap());
        }

        //After waiting. Get the write position again.
        let write = self.write.load(Ordering::Relaxed);

        let count = slice.len();
        let buf_len = self.buf.len();

        if (write + count) < buf_len {
            self.buf[write..write + count].copy_from_slice(&slice[..count]);
        } else {
            let diff = buf_len - write;
            self.buf[write..].copy_from_slice(&slice[..diff]);
            self.buf[..(count - diff)].copy_from_slice(&slice[diff..count]);
        }

        self.write
            .store((write + count) % self.buf.len(), Ordering::Relaxed);
    }

    pub fn pop(&mut self) -> Option<f32> {
        let write = self.write.load(Ordering::Relaxed);
        let read = self.read.load(Ordering::Relaxed);
        if read == write {
            return None;
        }
        let item = self.buf[read];
        self.read
            .store((read + 1) % self.buf.len(), Ordering::Relaxed);

        let slots_free = if write < read {
            read - write
        } else {
            self.buf.len() - write + read
        };

        let requested = self.slots_requested.load(Ordering::Relaxed);
        if slots_free >= requested {
            // info!(
            //     "Asking for {} slots, {} are free. This is {:.0}% of the buffer({})",
            //     requested,
            //     slots_free,
            //     (requested as f32 / N as f32) * 100.0,
            //     N,
            // );
            self.block.notify_one();
        }

        Some(item)
    }
    pub fn slots_free(&self) -> usize {
        let write = self.write.load(Ordering::Relaxed);
        let read = self.read.load(Ordering::Relaxed);

        if write < read {
            //N = 2
            //write to 0
            //write is 1

            //read 0
            //read is 1

            //write to 1
            //write is 2

            //write to 2
            //write is 0

            //read 1, write 0
            //here only slot 0 is free
            //1 - 0 = 1
            read - write
        } else {
            //N = 2
            //write 1, read 0
            //(2 + 1) - (1 + 0)
            //2 slots free(1 & 2)
            self.buf.len() - write + read
        }
    }

    #[inline(always)]
    pub fn read(&self) -> usize {
        self.read.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn write(&self) -> usize {
        self.write.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_free() {
        let mut rb = Rb::new(2);

        rb.append(&[1.0; 1]);
        assert_eq!(rb.write(), 1);
        rb.pop().unwrap();
        assert_eq!(rb.read(), 1);
        rb.append(&[1.0; 1]);
        assert_eq!(rb.write(), 2);
        rb.append(&[1.0; 1]);
        assert_eq!(rb.write(), 0);

        assert!(rb.write() < rb.read());
        assert_eq!(rb.slots_free(), 1);
    }
    #[test]
    fn heap_rb() {
        let mut rb = Rb::new(2);

        rb.append(&[1.0; 2]);
        assert_eq!(rb.buf, vec![1.0, 1.0, 0.0]);

        assert_eq!(rb.pop().unwrap(), 1.0);
        assert_eq!(rb.pop().unwrap(), 1.0);
        assert_eq!(rb.buf, vec![1.0, 1.0, 0.0]);

        rb.append(&[2.0; 2]);
        assert_eq!(rb.buf, vec![2.0, 1.0, 2.0]);

        assert_eq!(rb.pop().unwrap(), 2.0);
        assert_eq!(rb.pop().unwrap(), 2.0);
        assert_eq!(rb.buf, vec![2.0, 1.0, 2.0]);

        rb.append(&[3.0; 2]);

        assert_eq!(rb.pop().unwrap(), 3.0);
        assert_eq!(rb.pop().unwrap(), 3.0);
        assert_eq!(rb.buf, vec![2.0, 3.0, 3.0]);

        rb.append(&[4.0; 2]);
        assert_eq!(rb.buf, vec![4.0, 4.0, 3.0]);
    }
}
