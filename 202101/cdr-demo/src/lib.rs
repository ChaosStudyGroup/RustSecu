//! https://github.com/hrektts/cdr-rs/issues/10
//!
//! `Deserializer::read_vec`方法创建一个未初始化的缓冲区，
//! 并将其传递给用户提供的`Read`实现（self.reader.read_exact）。
//! 这是不合理的，因为它允许安全的`Rust`代码表现出未定义的行为（从未初始化的内存读取）。

/**

    ### 漏洞代码
    ```rust
    fn read_vec(&mut self) -> Result<Vec<u8>> {
        let len: u32 = de::Deserialize::deserialize(&mut *self)?;
        // 创建了未初始化buf
        let mut buf = Vec::with_capacity(len as usize);
        unsafe { buf.set_len(len as usize) }
        self.read_size(u64::from(len))?;
        // 将其传递给了用户提供的`Read`实现
        self.reader.read_exact(&mut buf[..])?;
        Ok(buf)
    }
    ```

    ### 修正
    ```rust
    fn read_vec(&mut self) -> Result<Vec<u8>> {
        let len: u32 = de::Deserialize::deserialize(&mut *self)?;
        // 创建了未初始化buf
        let mut buf = Vec::with_capacity(len as usize);
        // 初始化为 0；
        buf.resize(len as usize, 0);
        self.read_size(u64::from(len))?;
        // 将其传递给了用户提供的`Read`实现
        self.reader.read_exact(&mut buf[..])?;
        Ok(buf)
    }
    ```
**/
fn t(){}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
