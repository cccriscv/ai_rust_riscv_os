改進 Shell：支援引號字串解析 (修復 write 問題)。
多工排程：實作 fork 和 wait，這是重新引入 Pipe 的必要前置條件（避免死鎖）。