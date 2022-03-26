use std::ptr::NonNull;

use error::Error;

use crate::{ErrorCode, VariableSizeRingBuffer};

/// A thread safe implementation of a single-producer-single-consumer
/// ring buffer.
pub struct SPSC<T> {
    ring_buffer: T,
}

/// Sender facade
pub struct Sender<'a> {
    buffer: NonNull<VariableSizeRingBuffer<'a>>,
}
/// Receiver facade
pub struct Consumer<'a> {
    buffer: NonNull<VariableSizeRingBuffer<'a>>,
}

unsafe impl<'a> Send for Consumer<'a> {}
unsafe impl<'a> Send for Sender<'a> {}

impl<'a> SPSC<VariableSizeRingBuffer<'a>> {
    pub fn default() -> SPSC<VariableSizeRingBuffer<'a>> {
        SPSC::<VariableSizeRingBuffer> {
            ring_buffer: VariableSizeRingBuffer::default(),
        }
    }

    pub fn new(size: usize) -> SPSC<VariableSizeRingBuffer<'a>> {
        SPSC::<VariableSizeRingBuffer> {
            ring_buffer: VariableSizeRingBuffer::new(size),
        }
    }

    /// Splits the ring buffer into a sender and a receiver
    /// Both need mutable access, so this split uses unsafe key word.
    pub fn split(&self) -> (Sender<'a>, Consumer<'a>) {
        unsafe {
            let nn1 = NonNull::new_unchecked(&self.ring_buffer as *const _ as *mut _);
            let nn2 = NonNull::new_unchecked(&self.ring_buffer as *const _ as *mut _);

            (Sender { buffer: nn1 }, Consumer { buffer: nn2 })
        }
    }
}

impl<'a> Sender<'a> {
    pub fn new_entry<T>(&mut self) -> Result<&'a mut T, Error<ErrorCode>> {
        unsafe { self.buffer.as_mut().new_entry_no_commit::<T>() }
    }

    pub fn commit(&mut self) {
        unsafe { self.buffer.as_mut().commit() }
    }
}

impl<'a> Consumer<'a> {
    pub fn consume<T>(&mut self) -> Result<&'a T, Error<ErrorCode>> {
        unsafe { self.buffer.as_mut().consume_no_release::<T>() }
    }

    pub fn release(&mut self) {
        unsafe {
            self.buffer.as_mut().release();
        }
    }
}
