use ringbuffer::VariableSizeRingBuffer;

struct FixedSizedString<const S: usize> {
    buffer: [u8; S],
    len: usize,
}
impl<const S: usize> FixedSizedString<S> {
    fn new() -> FixedSizedString<S> {
        FixedSizedString {
            buffer: [0; S],
            len: 0,
        }
    }
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.buffer[..self.len]).unwrap()
    }
}

struct Test {
    pub name: [u8; 256],
}
struct Test1 {
    pub name: String,
}
unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
}
#[test]
fn add_bytes_into_buffer() {
    let ring_buffer = VariableSizeRingBuffer::<10>::default();
    let id: u32 = 10;
    let id = unsafe {
        ::std::slice::from_raw_parts(
            (id as *const u32) as *const u8,
            ::std::mem::size_of::<u32>(),
        )
    };
    let str = String::from("wat is dat denn?");
    let str1: Vec<u8> = str.into();

    let mut bytes1: [u8; 256] = [0; 256];
    let mut len = 0;
    {
        let mut test1 = Test1 {
            name: String::new(),
        };
        {
            test1.name = String::from("wat is dat denn?");
        }
        let bytes12: &[u8] = unsafe { any_as_u8_slice(&test1) };
        bytes1[..bytes12.len()].clone_from_slice(&bytes12);
        len = bytes12.len();
    }

    let returnTest1: Test1 = unsafe { std::ptr::read(bytes1[..len].as_ptr() as *const _) };

    let mut bytes2: [u8; 256] = [0; 256];
    len = 0;
    {
        let mut test = Test { name: [0; 256] };
        test.name[..str1.len()].clone_from_slice(&str1);
        let bytes22: &[u8] = unsafe { any_as_u8_slice(&test) };
        bytes2[..bytes22.len()].clone_from_slice(&bytes22);
        len = bytes22.len();
    }

    let returnTest1: Test1 = unsafe { std::ptr::read(bytes1[..len].as_ptr() as *const _) };

    println!("");

    //assert_eq!(ring_buffer.add_item(id,))
}
