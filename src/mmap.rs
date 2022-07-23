use std::io::{self, Error};
use std::marker::PhantomData;
use std::{mem, ptr, slice};

use libc::{c_void, MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE};

pub struct ReadWritable;
pub struct Executable;

/// A wrapper around the `mmap(2)` syscall.
pub struct MemoryMap<'a, Mode = ReadWritable> {
    addr: &'a mut [u8],
    mode: PhantomData<Mode>,
}

impl<'a> MemoryMap<'a, ReadWritable> {
    /// Create a new readable and writable memory mapped region.
    pub fn new(len: usize) -> io::Result<Self> {
        // SAFETY: This call is according to the man pages.
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

        // SAFETY: `mmap` returns a mutable pointer to the mapped area that is `len` bytes long.
        let addr = unsafe { slice::from_raw_parts_mut(addr as *mut u8, len) };

        Ok(Self {
            addr,
            mode: PhantomData,
        })
    }

    /// Returns a mutable reference to the memory mapped region.
    pub fn get_mut(&mut self) -> &mut [u8] {
        &mut self.addr
    }

    /// Changes the permissions of the memory mapped region from readable and writable
    /// to only executable.
    pub fn set_executable(self) -> io::Result<MemoryMap<'a, Executable>> {
        let addr = &mut self.addr[0] as *mut u8 as *mut c_void;

        // SAFETY: `mprotect` can only be called once and only after a successful call to `mmap`.
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
    /// Casts the first byte of the memory mapped region into a function pointer and calls it.
    ///
    /// # Safety
    ///
    /// The method is unsafe because the caller can write arbitrary values to the memory mapped
    /// region by calling [get_mut](crate::mmap::MemoryMap::get_mut).
    pub unsafe fn execute(self) {
        let addr = &mut self.addr[0] as *mut u8 as *mut c_void;
        let fn_ptr = mem::transmute::<_, fn() -> ()>(addr);
        fn_ptr();
    }
}
