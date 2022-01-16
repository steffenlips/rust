use std::marker::PhantomData;

pub struct VariableSizeRingBuffer<'a> {
    memory: Vec<u8>,
    front: usize,
    tail: usize,
    phantom: PhantomData<&'a u8>,
}

impl<'a> VariableSizeRingBuffer<'a> {
    pub fn default() -> VariableSizeRingBuffer<'a> {
        VariableSizeRingBuffer {
            memory: vec![0; 1024],
            front: 0,
            tail: 0,
            phantom: PhantomData,
        }
    }

    pub fn new(size: usize) -> VariableSizeRingBuffer<'a> {
        VariableSizeRingBuffer {
            memory: vec![0; size],
            front: 0,
            tail: 0,
            phantom: PhantomData,
        }
    }

    pub fn push_entry<T>(&mut self) -> Option<&'a mut T> {
        let rest_size = self.memory.len() - self.tail;
        let entry_size = std::mem::size_of::<T>();
        if rest_size < entry_size {
            return None;
        }

        let slice = &mut self.memory.as_mut_slice()[self.tail..entry_size] as *mut [u8];
        let result = unsafe {
            let res = slice as *mut [u8] as *mut T;
            let res = &mut *res;
            res
        };
        self.tail += entry_size;
        Some(result)
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
        } else if self.front > self.tail && self.tail <= new_front {
        }
        None
    }
}
