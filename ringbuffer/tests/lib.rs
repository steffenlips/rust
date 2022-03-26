use error::Error;
use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Instant,
};

use ringbuffer::{spsc::SPSC, ErrorCode, VariableSizeRingBuffer};

#[test]
fn push_single_value() {
    let mut ring_buffer = VariableSizeRingBuffer::new(10);

    let entry = ring_buffer.new_entry::<u16>();
    assert!(entry.is_ok());
    let entry = entry.unwrap();
    assert!(*entry == 0);
}
#[test]
fn no_space_available() {
    let mut ring_buffer = VariableSizeRingBuffer::new(3);

    let entry = ring_buffer.new_entry::<u32>();
    assert!(entry.is_err());
    assert_eq!(entry, Err(Error::new(ErrorCode::NoSpaceAvailable, "")));
}
#[test]
fn no_space_availavle2() {
    let mut ring_buffer = VariableSizeRingBuffer::new(3);

    let entry = ring_buffer.new_entry::<u16>();
    assert!(entry.is_ok());
    let entry = ring_buffer.new_entry::<u16>();
    assert!(entry.is_err());
    assert_eq!(entry, Err(Error::new(ErrorCode::NoSpaceAvailable, "")));
}
#[test]
fn consume_single_value() {
    let mut ring_buffer = VariableSizeRingBuffer::new(10);
    {
        let entry = ring_buffer.new_entry::<u16>();
        let entry = entry.unwrap();
        *entry = 300;
    }
    {
        let entry = ring_buffer.consume::<u16>();
        assert!(entry.is_ok());
        let entry = entry.unwrap();
        assert!(*entry == 300);
    }
}
#[test]
fn consume_empty() {
    let mut ring_buffer = VariableSizeRingBuffer::new(3);

    let entry = ring_buffer.consume::<u32>();
    assert!(entry.is_err());
    assert_eq!(entry, Err(Error::new(ErrorCode::Empty, "")));
}
#[test]
fn consume_empty2() {
    let mut ring_buffer = VariableSizeRingBuffer::new(3);
    ring_buffer.new_entry::<u16>().unwrap();
    ring_buffer.consume::<u16>().unwrap();
    let entry = ring_buffer.consume::<u16>();
    assert!(entry.is_err());
    assert_eq!(entry, Err(Error::new(ErrorCode::Empty, "")));
}
#[test]
fn size_mismatch() {
    let mut ring_buffer = VariableSizeRingBuffer::new(3);
    ring_buffer.new_entry::<u8>().unwrap();
    let entry = ring_buffer.consume::<u16>();
    assert!(entry.is_err());
    assert_eq!(entry, Err(Error::new(ErrorCode::SizeMismatch, "")));
}
#[test]
fn push_two_different_and_consume() {
    let mut ring_buffer = VariableSizeRingBuffer::new(10);
    {
        let entry = ring_buffer.new_entry::<u8>();
        let entry = entry.unwrap();
        *entry = 16;
    }
    {
        let entry = ring_buffer.new_entry::<u16>();
        let entry = entry.unwrap();
        *entry = 300;
    }
    assert_eq!(*ring_buffer.consume::<u8>().unwrap(), 16);
    assert_eq!(*ring_buffer.consume::<u16>().unwrap(), 300);
}
#[test]
fn push_and_consume_over_bounds() {
    let mut ring_buffer = VariableSizeRingBuffer::new(5);
    {
        let entry = ring_buffer.new_entry::<u16>();
        let entry = entry.unwrap();
        *entry = 300;
        assert_eq!(*ring_buffer.consume::<u16>().unwrap(), 300);
    }
    {
        let entry = ring_buffer.new_entry::<u16>();
        let entry = entry.unwrap();
        *entry = 16000;
        assert_eq!(*ring_buffer.consume::<u16>().unwrap(), 16000);
    }
}
#[test]
fn multiple_push_and_consume() {
    let mut ring_buffer = VariableSizeRingBuffer::new(9);
    for n in 1..1000000 {
        let entry = ring_buffer.new_entry::<u32>();
        let entry = entry.unwrap();
        *entry = 100 + n;
        assert_eq!(*ring_buffer.consume::<u32>().unwrap(), (100 + n));
    }
}
#[test]
fn multiple_push_and_consume2() {
    let mut ring_buffer = VariableSizeRingBuffer::new(9);
    for n in 1..1000000 {
        let entry = ring_buffer.new_entry::<u32>();
        let entry = entry.unwrap();
        *entry = 100 + n;
        assert_eq!(*ring_buffer.consume::<u32>().unwrap(), (100 + n));
    }
}
#[test]
fn multiple_push_and_consume3() {
    let start = Instant::now();
    let mut ring_buffer = VariableSizeRingBuffer::new(8);
    for n in 1..10000000 {
        let entry = ring_buffer.new_entry::<u32>();
        let entry = entry.unwrap();
        *entry = 100 + n;
        assert_eq!(*ring_buffer.consume::<u32>().unwrap(), (100 + n));
    }
    let duration = start.elapsed();

    println!(
        "Time elapsed in multiple_push_and_consume3() is: {:?}",
        duration
    );
}
struct BoxedValue {
    pub val: u32,
}
#[test]
fn compare_to_channel2() {
    let (push_sender, push_receiver): (Sender<BoxedValue>, Receiver<BoxedValue>) = mpsc::channel();
    let times: u32 = 10000000;
    let handle = thread::spawn(move || {
        for n in 1..times {
            let test = push_receiver.recv().unwrap();
            assert_eq!(test.val, n + 100);
        }
    });
    let start = Instant::now();

    for n in 1..times {
        let v = BoxedValue { val: 100 + n };
        //let v = Box::new(v);
        push_sender.send(v).unwrap();
    }

    handle.join().unwrap();
    let duration = start.elapsed();
    println!("Time elapsed in compare_to_channel2() is: {:?}", duration);
}
#[test]
fn compare_to_channel() {
    let (push_sender, push_receiver): (Sender<u32>, Receiver<u32>) = mpsc::channel();
    let times: u32 = 10000000;
    let handle = thread::spawn(move || {
        for n in 1..times {
            let test = push_receiver.recv().unwrap();
            assert_eq!(n + 100, test);
        }
    });
    let start = Instant::now();

    for n in 1..times {
        push_sender.send(100 + n).unwrap();
    }

    handle.join().unwrap();
    let duration = start.elapsed();
    println!("Time elapsed in compare_to_channel() is: {:?}", duration);
}
#[test]
fn multiple_push_and_consume_thread_safe() {
    let ring_buffer = Arc::new(Mutex::new(VariableSizeRingBuffer::new(1024 * 1024)));
    let clone = ring_buffer.clone();
    let (push_sender, push_receiver): (Sender<u32>, Receiver<u32>) = mpsc::channel();
    let (consume_sender, consume_receiver): (Sender<u32>, Receiver<u32>) = mpsc::channel();
    let times: u32 = 10000000;
    let handle = thread::spawn(move || {
        for n in 1..times {
            let test = consume_receiver.recv().unwrap();
            assert_eq!(test, (100 + n));
            {
                let mut locked = clone.lock().expect("Cannot read lock!");
                let entry = locked.consume::<u32>();
                assert!(entry.is_ok());

                assert_eq!(*entry.unwrap(), (100 + n));

                if locked.wants_to_push() {
                    push_sender.send(0).unwrap();
                }
            }
        }
    });

    let start = Instant::now();

    for n in 1..times {
        loop {
            let mut wait = false;
            {
                let mut locked = ring_buffer.lock().expect("Cannot read lock!");
                let entry = locked.new_entry::<u32>();
                if entry.is_ok() {
                    *entry.unwrap() = 100 + n;
                } else {
                    wait = true;
                }
            }
            if wait {
                push_receiver.recv().unwrap();
                continue;
            }

            consume_sender.send(100 + n).unwrap();
            break;
        }
    }

    handle.join().unwrap();
    let duration = start.elapsed();
    println!(
        "Time elapsed in multiple_push_and_consume_thread_safe() is: {:?}",
        duration
    );
}

#[test]
fn multiple_push_and_consume_thread_safe_no_lock() {
    let ring_buffer = SPSC::<VariableSizeRingBuffer>::new(1024 * 1024 * 16);
    let (mut sender, mut receiver) = ring_buffer.split();
    let times: u32 = 10000000;
    let current_handle = thread::current();
    let handle = thread::spawn(move || {
        for n in 1..times {
            loop {
                let entry = receiver.consume::<BoxedValue>();
                match entry {
                    Ok(entry) => {
                        let test = entry;
                        assert!(n == test.val);

                        receiver.release();
                        current_handle.unpark();
                        break;
                    }
                    Err(err) => match err.code {
                        ErrorCode::Empty => thread::park(),
                        _ => panic!("Receiver is broken"),
                    },
                }
            }
        }
    });

    let start = Instant::now();

    for n in 1..times {
        loop {
            let entry = sender.new_entry::<BoxedValue>();
            match entry {
                Ok(entry) => {
                    (*entry).val = n;
                    //*entry = n;
                    sender.commit();
                    handle.thread().unpark();
                    break;
                }
                Err(err) => match err.code {
                    ErrorCode::NoSpaceAvailable => thread::park(),
                    _ => panic!("Sender is broken"),
                },
            }
        }
    }

    handle.join().unwrap();
    let duration = start.elapsed();
    println!(
        "Time elapsed in multiple_push_and_consume_thread_safe_no_lock() is: {:?}",
        duration
    );
}
