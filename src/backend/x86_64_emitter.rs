pub struct X86_64Emitter { buffer: Vec<u8> }

impl X86_64Emitter {
}

impl X86_64Emitter {
    /// Creates a new `X86_64Emitter` with an empty instruction buffer.
    pub(crate) fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    /// Appends raw bytes to the instruction buffer.
    fn emit(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    /// `push rbp` (opcode: 0x55)
    /// Pushes the base pointer onto the stack. Typically used at function entry
    /// to save the caller's frame pointer before establishing a new stack frame.
    pub fn push_rbp(&mut self) {
        self.buffer.push(0x55);
    }

    /// `pop rbp` (opcode: 0x5D)
    /// Pops the top of the stack into rbp. Used at function exit to restore
    /// the caller's frame pointer.
    pub fn pop_rbp(&mut self) {
        self.buffer.push(0x5D);
    }

    /// `push rax` (opcode: 0x50)
    /// Pushes the accumulator register onto the stack. Used to temporarily
    /// save rax (e.g., before evaluating a subexpression) so it can be restored later.
    pub fn push_rax(&mut self) {
        self.buffer.push(0x50);
    }

    /// `pop rax` (opcode: 0x58)
    /// Pops the top of the stack into rax. Restores a previously saved accumulator value.
    pub fn pop_rax(&mut self) {
        self.buffer.push(0x58);
    }

    /// `pop rcx` (opcode: 0x59)
    /// Pops the top of the stack into rcx. Typically used to retrieve the
    /// left-hand operand of a binary operation while rax holds the right-hand operand.
    pub fn pop_rcx(&mut self) {
        self.buffer.push(0x59);
    }

    /// `ret` (opcode: 0xC3)
    /// Returns from the current procedure by popping the return address from
    /// the stack into rip.
    pub fn ret(&mut self) {
        self.buffer.push(0xC3);
    }

    /// `mov rbp, rsp` (opcodes: 0x48 0x89 0xE5)
    /// Sets the base pointer to the current stack pointer, establishing
    /// the stack frame for the current function.
    pub fn mov_rbp_rsp(&mut self) {
        self.emit(&[0x48, 0x89, 0xE5]);
    }

    /// `mov rsp, rbp` (opcodes: 0x48 0x89 0xEC)
    /// Restores the stack pointer from the base pointer, tearing down
    /// the current stack frame (function epilogue).
    pub fn mov_rsp_rbp(&mut self) {
        self.emit(&[0x48, 0x89, 0xEC]);
    }

    /// `sub rsp, imm8` (opcodes: 0x48 0x83 0xEC imm8)
    /// Subtracts an 8-bit immediate from rsp, allocating `n` bytes of
    /// space on the stack for local variables.
    pub fn sub_rsp_imm8(&mut self, n: u8) {
        self.emit(&[0x48, 0x83, 0xEC, n]);
    }

    /// `add rsp, imm8` (opcodes: 0x48 0x83 0xC4 imm8)
    /// Adds an 8-bit immediate to rsp, deallocating `n` bytes from the
    /// stack (freeing local variable space).
    pub fn add_rsp_imm8(&mut self, n: u8) {
        self.emit(&[0x48, 0x83, 0xC4, n]);
    }

    /// `add rax, rcx` (opcodes: 0x48 0x01 0xC8)
    /// Adds rcx to rax, storing the result in rax. Used to implement
    /// the `+` binary operator: `rax = rax + rcx`.
    pub fn add_rax_rcx(&mut self) {
        self.emit(&[0x48, 0x01, 0xC8]);
    }

    /// `sub rax, rcx` (opcodes: 0x48 0x29 0xC8)
    /// Subtracts rcx from rax, storing the result in rax. Used to implement
    /// the `-` binary operator: `rax = rax - rcx`.
    pub fn sub_rax_rcx(&mut self) {
        self.emit(&[0x48, 0x29, 0xC8]);
    }

    /// `imul rax, rcx` (opcodes: 0x48 0x0F 0xAF 0xC1)
    /// Signed multiplication of rax by rcx, storing the result in rax.
    /// Used to implement the `*` binary operator: `rax = rax * rcx`.
    pub fn imul_rax_rcx(&mut self) {
        self.emit(&[0x48, 0x0F, 0xAF, 0xC1]);
    }

    /// `mov eax, imm32` (opcode: 0xB8 + imm32)
    /// Loads a 32-bit signed immediate value into eax. The upper 32 bits
    /// of rax are implicitly zero-extended. Efficient encoding for values
    /// that fit in 32 bits.
    pub fn mov_eax_imm32(&mut self, v: i32) {
        self.buffer.push(0xB8);
        self.emit(&v.to_le_bytes());
    }

    /// `movabs rax, imm64` (opcodes: 0x48 0xB8 + imm64)
    /// Loads a full 64-bit immediate value into rax. Used for large constants
    /// or absolute addresses (e.g., function pointers for `call`).
    pub fn mov_rax_imm64(&mut self, v: u64) {
        self.emit(&[0x48, 0xB8]);
        self.emit(&v.to_le_bytes());
    }

    /// `call rax` (opcodes: 0xFF 0xD0)
    /// Calls the function whose address is stored in rax. Pushes the return
    /// address onto the stack and transfers control to the target.
    pub fn call_rax(&mut self) {
        self.emit(&[0xFF, 0xD0]);
    }

    /// `mov rax, [rbp + disp8]` (opcodes: 0x48 0x8B 0x45 disp8)
    /// Loads a 64-bit value from the stack at `rbp + d` into rax. Used to
    /// read a local variable or function parameter from its stack slot.
    pub fn mov_rax_rbp_disp8(&mut self, d: i8) {
        self.emit(&[0x48, 0x8B, 0x45, d as u8]);
    }

    /// `mov [rbp + disp8], rax` (opcodes: 0x48 0x89 0x45 disp8)
    /// Stores the value in rax into the stack at `rbp + d`. Used to write
    /// a value into a local variable's stack slot.
    pub fn mov_rbp_disp8_rax(&mut self, d: i8) {
        self.emit(&[0x48, 0x89, 0x45, d as u8]);
    }

    /// `mov rcx, rax` (opcodes: 0x48 0x89 0xC1)
    /// Copies rax into rcx. On the Windows x64 ABI the first integer
    /// argument is passed in rcx, so this moves the computed argument
    /// value into the correct register before a `call`.
    pub fn mov_rcx_rax(&mut self) {
        self.emit(&[0x48, 0x89, 0xC1]);
    }

    /// Consumes the emitter and returns the assembled machine code as a
    /// byte vector, ready to be written to executable memory.
    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }
}