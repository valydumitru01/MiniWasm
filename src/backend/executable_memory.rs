use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE};


/// ExecutableMemory solves one specific problem:
/// the OS won't let you execute bytes from a normal Vec<u8>.
///
/// When you allocate memory normally (Vec::new(), Box::new(), etc.), the OS marks those pages as
/// read/write but NOT execute. This is a security feature called DEP (Data Execution Prevention).
/// It prevents malware from writing shellcode into a buffer and jumping to it.
pub struct ExecutableMemory {
    ptr: *mut u8,
    size: usize,
}

impl ExecutableMemory {
    pub(crate) fn new(code: &[u8]) -> Self {
        let size = code.len();
        let ptr = unsafe {
            // Ask Windows for memory that is writable AND executable
            let p = VirtualAlloc(
                std::ptr::null_mut(),       // let OS choose the address
                size,                        // how many bytes
                MEM_COMMIT | MEM_RESERVE,   // allocate and commit pages
                PAGE_EXECUTE_READWRITE,     // the key flag: W+X
            ) as *mut u8;

            assert!(!p.is_null(), "VirtualAlloc failed");

            // Copy our JIT-generated machine code into this region
            // Using non overlapping copy because it is faster and we know the source and
            // destination memory regions are not overlapping.
            std::ptr::copy_nonoverlapping(code.as_ptr(), p, size);

            p
        };

        Self { ptr, size }
    }

    pub(crate) fn as_fn_ptr(&self, offset: usize) -> *const u8 {
        assert!(offset <= self.size);
        unsafe { self.ptr.add(offset) }
    }
}

impl Drop for ExecutableMemory {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                // Return the memory to the OS
                VirtualFree(self.ptr as *mut _, 0, MEM_RELEASE);
            }
        }
    }
}