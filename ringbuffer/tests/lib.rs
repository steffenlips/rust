use ringbuffer::VariableSizeRingBuffer;

#[test]
fn push_single_value() {
    let mut ring_buffer = VariableSizeRingBuffer::new(10);

    let entry = ring_buffer.push_entry::<u16>();
    assert!(entry.is_some());
    let entry = entry.unwrap();
    assert!(*entry == 0);
}
#[test]
fn consume_single_value() {
    let mut ring_buffer = VariableSizeRingBuffer::new(10);
    {
        let entry = ring_buffer.push_entry::<u16>();
        let entry = entry.unwrap();
        *entry = 300;
    }
    {
        let entry = ring_buffer.consume::<u16>();
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert!(*entry == 300);
    }
}
#[test]
fn push_two_different_and_consume() {
    let mut ring_buffer = VariableSizeRingBuffer::new(10);
    {
        let entry = ring_buffer.push_entry::<u8>();
        let entry = entry.unwrap();
        *entry = 16;
    }
    {
        let entry = ring_buffer.push_entry::<u16>();
        let entry = entry.unwrap();
        *entry = 300;
    }
    assert_eq!(*ring_buffer.consume::<u8>().unwrap(), 16);
    assert_eq!(*ring_buffer.consume::<u16>().unwrap(), 300);
}

