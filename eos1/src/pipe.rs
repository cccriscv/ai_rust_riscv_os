// === FILE: ./eos1/src/pipe.rs ===
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use crate::sync::SpinLock;

const PIPE_SIZE: usize = 512;

pub struct Pipe {
    buffer: VecDeque<u8>,
    pub write_count: usize, // 記錄目前有多少個活躍的寫入端
}

impl Pipe {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::with_capacity(PIPE_SIZE),
            write_count: 0, 
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let mut read_count = 0;
        for b in buf.iter_mut() {
            if let Some(data) = self.buffer.pop_front() {
                *b = data;
                read_count += 1;
            } else {
                break;
            }
        }
        read_count
    }

    pub fn write(&mut self, data: &[u8]) -> usize {
        let mut write_count = 0;
        for &b in data {
            if self.buffer.len() < PIPE_SIZE {
                self.buffer.push_back(b);
                write_count += 1;
            } else {
                break; 
            }
        }
        write_count
    }
}

// --- 讀寫端封裝 (Handle) ---

// 寫入端 Handle
pub struct Writer {
    pub pipe: Arc<SpinLock<Pipe>>,
}

impl Writer {
    // 建立一個新的 Writer，並增加計數
    pub fn new(pipe: Arc<SpinLock<Pipe>>) -> Self {
        pipe.lock().write_count += 1;
        Self { pipe }
    }
}

// 當 Writer 被複製 (如 dup2, fork) 時，增加計數
impl Clone for Writer {
    fn clone(&self) -> Self {
        self.pipe.lock().write_count += 1;
        Self { pipe: self.pipe.clone() }
    }
}

// 當 Writer 被銷毀 (如 close, task exit) 時，減少計數
impl Drop for Writer {
    fn drop(&mut self) {
        let mut p = self.pipe.lock();
        if p.write_count > 0 {
            p.write_count -= 1;
        }
    }
}

// 讀取端 Handle (目前不需要特殊計數，但也封裝起來)
#[derive(Clone)]
pub struct Reader {
    pub pipe: Arc<SpinLock<Pipe>>,
}

// 讓 Task 使用的 Enum
#[derive(Clone)]
pub enum PipeEnd {
    Read(Reader),
    Write(Writer),
}