use std::io::{self, Error};
use std::marker::PhantomData;
use std::{mem, ptr, slice};

use libc::{c_void, MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE};

pub struct ReadWritable;
pub struct Executable;

/// A wrapper around the `mmap(2)` syscall.
pub struct MemoryMap<Mode = ReadWritable> {
    addr: *mut c_void,
    len: usize,
    mode: PhantomData<Mode>,
}

impl MemoryMap<ReadWritable> {
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

        Ok(Self {
            addr,
            len,
            mode: PhantomData,
        })
    }

    /// Returns a mutable reference to the memory mapped region.
    pub fn get_mut(&mut self) -> &mut [u8] {
        // SAFETY: `mmap` returns a mutable pointer to the mapped area that is `len` bytes long.
        unsafe { slice::from_raw_parts_mut(self.addr as *mut u8, self.len) }
    }

    /// Changes the permissions of the memory mapped region from readable and writable
    /// to only executable.
    pub fn set_executable(self) -> io::Result<MemoryMap<Executable>> {
        // SAFETY: `mprotect` can only be called once and only after a successful call to `mmap`.
        if unsafe { libc::mprotect(self.addr, self.len, PROT_EXEC) } == -1 {
            Err(Error::last_os_error())
        } else {
            Ok(MemoryMap {
                addr: self.addr,
                len: self.len,
                mode: PhantomData,
            })
        }
    }
}

impl MemoryMap<Executable> {
    /// Casts the first byte of the memory mapped region into a function pointer and calls it.
    ///
    /// # Safety
    ///
    /// The method is unsafe because the caller can write arbitrary values to the memory mapped
    /// region by calling [get_mut](crate::mmap::MemoryMap::get_mut).
    pub unsafe fn execute(self) {
        let function = mem::transmute::<_, fn() -> ()>(self.addr);
        function();
    }
}
