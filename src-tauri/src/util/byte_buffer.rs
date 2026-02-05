use thiserror::Error;

#[derive(Debug, Error)]
pub enum ByteBufferError {
    #[error("not enough bytes")]
    NotEnoughBytes,
    #[error("failed to decode string")]
    FailedToDecodeString,
}

pub struct ByteBuffer {
    pub data: Vec<u8>,
    index: usize,
}

impl ByteBuffer {
    /// 构造一个 `ByteBuffer` 用于读取数据。
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            index: 0,
        }
    }

    /// 构造一个 `ByteBuffer` 用于写入数据。
    pub fn new_empty() -> Self {
        Self {
            data: Vec::new(),
            index: 0,
        }
    }

    /// 读取 `n` 个字节的数据。
    ///
    /// - Parameter length: 读取的数据长度。
    /// - Returns: 长度为 `length` 字节的 `Vec<u8>`。
    /// - Throws: 若剩余字节不足，抛出 `ByteBufferError::NotEnoughBytes` 错误。
    pub fn read_data(&mut self, length: usize) -> Result<Vec<u8>, ByteBufferError> {
        if length == 0 {
            return Ok(Vec::new());
        }

        if self.index + length > self.data.len() {
            return Err(ByteBufferError::NotEnoughBytes);
        }

        let result = self.data[self.index..self.index + length].to_vec();
        self.index += length;
        Ok(result)
    }

    /// 读取一个 `u8`。
    ///
    /// - Throws: 若剩余字节不足，抛出 `ByteBufferError::NotEnoughBytes` 错误。
    pub fn read_u8(&mut self) -> Result<u8, ByteBufferError> {
        let data = self.read_data(1)?;
        Ok(data[0])
    }

    /// 读取一个 `u16`（大端序）。
    ///
    /// - Throws: 若剩余字节不足，抛出 `ByteBufferError::NotEnoughBytes` 错误。
    pub fn read_u16(&mut self) -> Result<u16, ByteBufferError> {
        let data = self.read_data(2)?;
        let value = u16::from_be_bytes([data[0], data[1]]);
        Ok(value)
    }

    /// 读取一个 `u32`（大端序）。
    ///
    /// - Throws: 若剩余字节不足，抛出 `ByteBufferError::NotEnoughBytes` 错误。
    pub fn read_u32(&mut self) -> Result<u32, ByteBufferError> {
        let data = self.read_data(4)?;
        let value = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        Ok(value)
    }

    /// 读取一个 **UTF-8** 字符串。
    ///
    /// - Parameter length: 字符串长度。
    /// - Throws:
    ///   - 若剩余字节不足，抛出 `ByteBufferError::NotEnoughBytes` 错误。
    ///   - 若解析字符串失败，抛出 `ByteBufferError::FailedToDecodeString` 错误。
    pub fn read_string(&mut self, length: Option<usize>) -> Result<String, ByteBufferError> {
        let read_length = length.unwrap_or(self.data.len() - self.index);
        let data = self.read_data(read_length)?;
        String::from_utf8(data).map_err(|_| ByteBufferError::FailedToDecodeString)
    }

    /// 写入一段数据
    /// - Parameter data: 写入的数据
    pub fn write_data(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    /// 写入一个 u8
    /// - Parameter value: 要写入的 u8
    pub fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    /// 写入一个 u16（大端序）
    /// - Parameter value: 要写入的 u16
    pub fn write_u16(&mut self, value: u16) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    /// 写入一个 u32（大端序）
    /// - Parameter value: 要写入的 u32
    pub fn write_u32(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    /// 写入一个 **UTF-8** 字符串。
    pub fn write_string(&mut self, string: &str) {
        self.data.extend_from_slice(string.as_bytes());
    }

    /// 写入一个 VarInt。
    /// - Parameter value: 要写入的 VarInt 值。
    pub fn write_varint(&mut self, value: usize) {
        let mut val = value;
        while val > 0x7F {
            self.data.push((val as u8) | 0x80);
            val >>= 7;
        }
        self.data.push(val as u8);
    }

    /// 读取一个 VarInt。
    /// - Returns: 读取的 VarInt 值。
    /// - Throws: 若数据不足或 VarInt 过大，抛出错误。
    pub fn read_varint(&mut self) -> Result<i32, ByteBufferError> {
        let mut result = 0;
        let mut shift = 0;

        loop {
            if self.index >= self.data.len() {
                return Err(ByteBufferError::NotEnoughBytes);
            }

            let byte = self.data[self.index];
            self.index += 1;

            result |= ((byte & 0x7F) as i32) << shift;
            shift += 7;

            if (byte & 0x80) == 0 {
                break;
            }

            if shift >= 32 {
                return Err(ByteBufferError::NotEnoughBytes); // 或者创建新的错误类型
            }
        }

        Ok(result)
    }
}