static PAGE_SIZE: usize = 65536; // 64 KiB
pub struct Memory {
    data: Vec<u8>,
    page_count: u32,  // each page = 65536 bytes
}
impl Memory {
    pub(crate) fn new(initial_pages: u32) -> Self {
        Self { data: vec![0; initial_pages as usize * PAGE_SIZE], page_count: initial_pages }
    }
    fn read_bytes(&self, offset: usize, len: usize) -> &[u8]{
        assert!(offset + len <= self.data.len());
        &self.data[offset..offset + len]
    }
    fn write_bytes(&mut self, offset: usize, bytes: &[u8]){
        assert!(offset + bytes.len() <= self.data.len());
        self.data[offset..offset + bytes.len()].copy_from_slice(bytes);
    }
    fn size_pages(&self) -> u32{
        self.page_count
    }
    /// Grows memory by the specified number of pages. Returns an error if the new size would exceed
    /// 2 GiB.
    /// This size limit is a simplification for this implementation; real Wasm allows up to 4 GiB,
    /// but that would require using a 64-bit offset type and more complex bounds checks.
    fn grow(&mut self, pages: u32) -> Result<(), &'static str> {
        let new_size = (self.page_count as usize + pages as usize) * PAGE_SIZE;
        if new_size > 2 * 1024 * 1024 * 1024 { // 2 GiB limit
            return Err("Memory size exceeds 2 GiB");
        }
        self.data.resize(new_size, 0);
        self.page_count += pages;
        Ok(())
    }
    fn as_ptr(&self) -> *const u8{
        self.data.as_ptr()
    }
}