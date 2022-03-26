use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use error::Error;

pub mod spsc;
///////////////////////////////////////////////////////////////////////////////
/// This crate implements a basic ring buffer.
/// This implementation is implemented as a synchronous FIFO. It is not
/// thread safe.
#[derive(PartialEq, Debug)]
pub enum ErrorCode {
    NoSpaceAvailable,
    Empty,
    SizeMismatch,
}

///////////////////////////////////////////////////////////////////////////////
///
pub struct VariableSizeRingBuffer<'a> {
    memory: UnsafeCell<Vec<u8>>,
    front: AtomicUsize,
    tail: AtomicUsize,
    front_to_release: usize,
    tail_to_commit: usize,
    capacity: usize,
    wants_to_push: AtomicBool,
    phantom: PhantomData<&'a u8>,
}
/// Mark this implementation as Sync and Send, so it can be shared
/// between threads without using an Arc and a Mutex
unsafe impl<'a> Sync for VariableSizeRingBuffer<'a> {}
unsafe impl<'a> Send for VariableSizeRingBuffer<'a> {}

impl<'a> VariableSizeRingBuffer<'a> {
    pub fn default() -> VariableSizeRingBuffer<'a> {
        VariableSizeRingBuffer {
            memory: UnsafeCell::new(vec![0; 1024]),
            front: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            capacity: 1024,
            front_to_release: 0,
            tail_to_commit: 0,
            //items_in_queue: AtomicUsize::new(0),
            wants_to_push: AtomicBool::new(false),
            phantom: PhantomData,
        }
    }

    pub fn new(size: usize) -> VariableSizeRingBuffer<'a> {
        VariableSizeRingBuffer {
            memory: UnsafeCell::new(vec![0; size]),
            front: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            capacity: size,
            front_to_release: 0,
            tail_to_commit: 0,
            //items_in_queue: AtomicUsize::new(0),
            wants_to_push: AtomicBool::new(false),
            phantom: PhantomData,
        }
    }

    pub fn wants_to_push(&mut self) -> bool {
        match self
            .wants_to_push
            .compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed)
        {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn new_entry<T>(&mut self) -> Result<&'a mut T, Error<ErrorCode>> {
        let result = self.new_entry_no_commit();
        self.commit();
        result
    }

    pub fn new_entry_no_commit<T>(&mut self) -> Result<&'a mut T, Error<ErrorCode>> {
        let front = self.front.load(Ordering::Acquire);
        let mut tail = self.tail.load(Ordering::Acquire);

        let entry_size = std::mem::size_of::<T>();
        if front <= tail {
            if tail + entry_size <= self.capacity {
                // there is enough space until end
            } else if front > entry_size {
                // there is enough space from 0 to front
                tail = 0;
            } else {
                self.wants_to_push.store(true, Ordering::Relaxed);
                return Err(Error::<ErrorCode>::new(
                    ErrorCode::NoSpaceAvailable,
                    "No space available",
                ));
            }
        } else if front - tail > entry_size {
            // there is enough space between tail and front
        } else {
            self.wants_to_push.store(true, Ordering::Relaxed);
            return Err(Error::<ErrorCode>::new(
                ErrorCode::NoSpaceAvailable,
                "No space available",
            ));
        }
        //let slice = &mut self.memory.as_mut_slice()[self.tail..(self.tail + entry_size)];
        let slice = &mut *self.memory.get_mut();
        let slice = &mut slice.as_mut_slice()[tail..(tail + entry_size)];
        let result = unsafe {
            let res = slice as *mut [u8] as *mut T;
            let res = &mut *res;
            res
        };
        tail += entry_size;
        self.tail_to_commit = tail;
        return Ok(result);
    }
    pub fn commit(&mut self) {
        //self.items_in_queue.fetch_add(1, Ordering::Relaxed);
        self.tail.store(self.tail_to_commit, Ordering::Relaxed);
    }

    pub fn consume<T>(&mut self) -> Result<&'a T, Error<ErrorCode>> {
        let result = self.consume_no_release();
        self.release();
        result
    }

    pub fn consume_no_release<T>(&mut self) -> Result<&'a T, Error<ErrorCode>> {
        let mut front = self.front.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        let entry_size = std::mem::size_of::<T>();
        if front == tail && entry_size > 0 {
            return Err(Error::<ErrorCode>::new(ErrorCode::Empty, "Buffer is empty"));
        }

        if front < tail {
            if tail - front >= entry_size {
                // there is enough space between front and taile
            } else {
                // Indicates a wrong order while pushing and consuming
                return Err(Error::<ErrorCode>::new(
                    ErrorCode::SizeMismatch,
                    "The requested size is not available",
                ));
            }
        } else if front >= tail {
            if self.capacity >= front + entry_size {
                // there is enough space until the end
            } else if tail >= entry_size {
                front = 0;
                // there is enough space from 0 to tail
            } else {
                return Err(Error::<ErrorCode>::new(
                    ErrorCode::SizeMismatch,
                    "The requested size is not available",
                ));
            }
        }

        //let slice = &self.memory.as_slice()[self.front..(self.front + entry_size)];
        let slice = &mut *self.memory.get_mut();
        let slice = &mut slice.as_mut_slice()[front..(front + entry_size)];

        let result = unsafe {
            let res = slice as *const [u8] as *const T;
            let res = &*res;
            res
        };
        front += entry_size;
        self.front_to_release = front;
        return Ok(result);
    }

    pub fn release(&mut self) {
        self.front.store(self.front_to_release, Ordering::Relaxed);
    }
}
