pub struct VariableSizeRingBuffer<const S: usize> {
    memory: [u8; S],
    front: usize,
    tail: usize,
}

impl<const S: usize> VariableSizeRingBuffer<S> {
    pub fn default() -> VariableSizeRingBuffer<S> {
        VariableSizeRingBuffer {
            memory: [0; S],
            front: 0,
            tail: 0,
        }
    }
}
