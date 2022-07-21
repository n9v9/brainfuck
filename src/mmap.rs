use std::io::{self, Error};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut, Range};
use std::{mem, ptr, slice};

use libc::{c_void, MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE};

pub struct ReadWritable;
pub struct Executable;

pub struct MemoryMap<'a, Mode = ReadWritable> {
    addr: &'a mut [u8],
    mode: PhantomData<Mode>,
}

impl<'a> MemoryMap<'a, ReadWritable> {
    pub fn new(len: usize) -> io::Result<Self> {
        let addr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                len,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                -1,
                0,
            )
        };

        if addr == MAP_FAILED {
            return Err(Error::last_os_error());
        }

        let addr = unsafe { slice::from_raw_parts_mut(addr as *mut u8, len) };

        Ok(Self {
            addr,
            mode: PhantomData,
        })
    }

    pub fn get_mut(&mut self) -> &mut [u8] {
        &mut self.addr
    }

    pub fn set_executable(self) -> io::Result<MemoryMap<'a, Executable>> {
        let addr = &mut self.addr[0] as *mut u8 as *mut c_void;

        if unsafe { libc::mprotect(addr, self.addr.len(), PROT_EXEC) } == -1 {
            Err(Error::last_os_error())
        } else {
            Ok(MemoryMap {
                addr: self.addr,
                mode: PhantomData,
            })
        }
    }
}

impl<'a> MemoryMap<'a, Executable> {
    pub fn execute(self) {
        let addr = &mut self.addr[0] as *mut u8 as *mut c_void;
        let fn_ptr = unsafe { mem::transmute::<_, fn() -> ()>(addr) };
        fn_ptr();
    }
}
