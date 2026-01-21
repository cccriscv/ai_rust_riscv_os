## ccc: 請說明這個程式的 round-robin 部分

這個作業系統中的 **Round-Robin (輪詢) 排程器** 實作非常精簡且經典。它的核心邏輯是讓所有任務「排排坐，吃果果」，每個任務執行固定的時間片（Time Slice）後，就把 CPU 讓給下一個任務。

以下我將從 **資料結構**、**演算法邏輯** 到 **硬體觸發** 三個層面來剖析這段程式碼。

---

### 1. 資料結構 (`src/task.rs`)

排程器需要知道「現在輪到誰」以及「總共有哪些任務」。

```rust
pub struct Scheduler {
    // 1. 任務佇列：一個動態陣列，存放所有任務的指標 (Box<Task>)
    pub tasks: Vec<Box<Task>>,
    
    // 2. 當前指標：記錄目前正在執行的任務在陣列中的索引 (0, 1, 2...)
    pub current_index: usize,
}
```

*   **`tasks`**: 這就像是一個圓桌，所有任務都圍繞在桌邊。
*   **`current_index`**: 這就像是一根手指，指向現在正在吃東西（使用 CPU）的人。

---

### 2. 核心演算法 (`src/task.rs` -> `schedule`)

這是 Round-Robin 的靈魂所在。每次被呼叫時，它只做一件事：**把手指移到下一個人**。

```rust
    pub unsafe fn schedule(&mut self) -> *mut Context {
        // ... (省略錯誤檢查) ...

        // [核心演算法] 計算下一個任務的索引
        // 假設有 3 個任務 (len=3)，目前是 0
        // 0 -> (0+1)%3 = 1
        // 1 -> (1+1)%3 = 2
        // 2 -> (2+1)%3 = 0  <-- 這裡發生「繞圈 (Wrap around)」
        self.current_index = (self.current_index + 1) % self.tasks.len();
        
        // 取出下一個任務
        let next_task = &mut self.tasks[self.current_index];

        // ... (切換 SATP 記憶體空間) ...

        // 回傳下一個任務的 Context (暫存器狀態) 指標
        &mut next_task.context as *mut Context
    }
```

*   **`% self.tasks.len()`**：這是實現「循環」的關鍵數學運算（取餘數）。它保證了索引永遠不會超出範圍，並且會在到達尾端後自動回到 0。

---

### 3. 硬體觸發機制 (`src/main.rs` -> `handle_timer`)

光有演算法是不夠的，必須有人定期去「推」這個排程器一下。這個推手就是 **硬體計時器 (CLINT)**。

```rust
// 這是中斷處理函式，由 assembly (trap.S) 呼叫
#[unsafe(no_mangle)]
pub extern "C" fn handle_timer(_ctx_ptr: *mut Context) -> *mut Context {
    // 1. 重設鬧鐘：告訴硬體「下次請在 X 毫秒後再叫我」
    set_next_timer();
    
    // 2. 取得排程器實例
    let scheduler = task::get_scheduler();
    
    // 3. 執行排程：切換到下一個任務，並拿到新任務的 Context 指標
    // 注意：這裡的回傳值會被 trap.S 拿去恢復暫存器
    unsafe { scheduler.schedule() }
}
```

**流程圖解：**

1.  **CPU 執行中**：Task A 正在快樂地跑著。
2.  **時間到**：CLINT 硬體發出 Timer Interrupt。
3.  **暫停**：CPU 暫停 Task A，跳到 `trap_entry` (保存 Task A 的 Context)。
4.  **排程**：呼叫 `handle_timer` -> `scheduler.schedule()`。
5.  **切換**：`current_index` 從 A 移到 B。切換 SATP (記憶體空間)。
6.  **恢復**：回傳 Task B 的 Context。
7.  **返回**：`trap.S` 恢復 Task B 的暫存器，執行 `mret`。
8.  **執行**：現在 CPU 改跑 Task B 了。

---

### 4. 記憶體空間切換 (Process Isolation)

你的排程器不只切換 CPU 暫存器 (Context)，還切換了 **視角 (Memory Space)**。

在 `schedule` 函式中：

```rust
        let satp_val = if next_task.root_ppn != 0 {
            // 如果是 User App，切換到它專屬的分頁表
            (8 << 60) | next_task.root_ppn
        } else {
            // 如果是 Shell/Kernel，切換回核心分頁表
            // ...
        };
        
        // 寫入 SATP 暫存器，並刷新 TLB (快取)
        core::arch::asm!("csrw satp, {}", "sfence.vma", in(reg) satp_val);
```

這一步至關重要。因為有了這一步，Task 1 (Shell) 和 Task 2 (User App) 雖然可能都使用虛擬位址 `0x10000`，但因為 **SATP 不同**，CPU 看到的實體記憶體完全不同。

### 總結

你的 Round-Robin 實作非常標準且乾淨：
1.  **公平性**：每個任務輪流執行，沒有優先級之分。
2.  **搶佔式**：由 Timer 強制切換，任務無法霸佔 CPU。
3.  **隔離性**：結合 SATP 切換，達成了完整的行程隔離。