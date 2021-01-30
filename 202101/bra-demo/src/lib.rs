//!  https://github.com/Enet4/bra-rs/issues/1
//!
//! bra-rs 安全漏洞：读取未初始化内存导致 UB
//!
//! `GreedyAccessReader::fill_buf`方法创建了一个未初始化的缓冲区，
//! 并将其传递给用户提供的Read实现（`self.inner.read（buf）`）。
//! 这是不合理的，因为它允许`Safe Rust`代码表现出未定义的行为（从未初始化的内存读取）。
//!
//! 在标准库`Read` trait 的 `read` 方法文档中所示：
//!
//! > 您有责任在调用`read`之前确保`buf`已初始化。
//! > 用未初始化的`buf`（通过`MaybeUninit <T>`获得的那种）调用`read`是不安全的，并且可能导致未定义的行为。
//! https://doc.rust-lang.org/std/io/trait.Read.html#tymethod.read
//!
//! 解决方法：
//! 在`read`之前将新分配的`u8`缓冲区初始化为零是安全的，以防止用户提供的`Read`读取新分配的堆内存的旧内容。


// 以下是有安全风险的代码示例：
impl<R> BufRead for GreedyAccessReader<R>
    where
        R: Read,
{
    fn fill_buf(&mut self) -> IoResult<&[u8]> {
        if self.buf.capacity() == self.consumed {
            self.reserve_up_to(self.buf.capacity() + 16);
        }

        let b = self.buf.len();
        let buf = unsafe {
            // safe because it's within the buffer's limits
            // and we won't be reading uninitialized memory
            // 这里虽然没有读取未初始化内存，但是会导致用户读取
            std::slice::from_raw_parts_mut(
                self.buf.as_mut_ptr().offset(b as isize),
                self.buf.capacity() - b)
        };

        match self.inner.read(buf) {
            Ok(o) => {
                unsafe {
                    // reset the size to include the written portion,
                    // safe because the extra data is initialized
                    self.buf.set_len(b + o);
                }

                Ok(&self.buf[self.consumed..])
            }
            Err(e) => Err(e),
        }
    }

    fn consume(&mut self, amt: usize) {
        self.consumed += amt;
    }
}

// 修正以后的代码示例，去掉了未初始化的buf：
impl<R> BufRead for GreedyAccessReader<R>
    where
        R: Read,
{
    fn fill_buf(&mut self) -> IoResult<&[u8]> {
        if self.buf.capacity() == self.consumed {
            self.reserve_up_to(self.buf.capacity() + 16);
        }

        let b = self.buf.len();
        self.buf.resize(self.buf.capacity(), 0);
        let buf = &mut self.buf[b..];
        let o = self.inner.read(buf)?;

        // truncate to exclude non-written portion
        self.buf.truncate(b + o);

        Ok(&self.buf[self.consumed..])
    }

    fn consume(&mut self, amt: usize) {
        self.consumed += amt;
    }
}