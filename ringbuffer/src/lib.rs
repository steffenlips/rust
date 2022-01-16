use std::{marker::PhantomData, ops::Index};

pub struct VariableSizeRingBuffer<'a> {
    memory: Vec<u8>,
    front: usize,
    tail: usize,
    len: usize,
    phantom: PhantomData<&'a u8>,
}

impl<'a> VariableSizeRingBuffer<'a> {
    pub fn default() -> VariableSizeRingBuffer<'a> {
        VariableSizeRingBuffer {
            memory: vec![0; 1024],
            front: 0,
            tail: 0,
            len: 1024,
            phantom: PhantomData,
        }
    }

    pub fn new(size: usize) -> VariableSizeRingBuffer<'a> {
        VariableSizeRingBuffer {
            memory: vec![0; size],
            front: 0,
            tail: 0,
            len: size,
            phantom: PhantomData,
        }
    }

    pub fn push_entry<T>(&mut self) -> Option<&'a mut T> {
        let rest_size = self.memory.len() - self.tail;
        let entry_size = std::mem::size_of::<T>();
        if self.tail >= self.front && rest_size > entry_size {
            let slice = &mut self.memory.as_mut_slice()[self.tail..entry_size];
            let result = unsafe {
                let res = slice as *mut [u8] as *mut T;
                let res = &mut *res;
                res
            };
            self.tail += entry_size;
            return Some(result);
        } else if self.front >= entry_size {
            self.len = self.tail;
            self.tail = 0;
            let slice = &mut self.memory.as_mut_slice()[..entry_size];
            let result = unsafe {
                let res = slice as *mut [u8] as *mut T;
                let res = &mut *res;
                res
            };
            self.tail += entry_size;
            return Some(result);
        }
        None
    }

    pub fn consume<T>(&mut self) -> Option<&'a T> {
        let entry_size = std::mem::size_of::<T>();
        let new_front = self.front + entry_size;
        if self.front < self.tail && self.tail >= new_front {
            let slice = &self.memory.as_slice()[self.front..entry_size];
            self.front = new_front;
            let result = unsafe {
                let res = slice as *const [u8] as *const T;
                let res = &*res;
                res
            };
            return Some(result);
        } else if self.front >= self.tail && self.front <= self.len {
            let mut slice = &self.memory.as_slice()[self.front..entry_size];
            self.front = new_front;
            if self.front >= self.len {
                self.front = 0;
                slice = &self.memory.as_slice()[self.front..entry_size];
            }
            let result = unsafe {
                let res = slice as *const [u8] as *const T;
                let res = &*res;
                res
            };
            return Some(result);
        }
        None
    }
}
