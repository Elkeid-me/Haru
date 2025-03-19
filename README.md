## 在 Ubuntu 24.04 上构建

1. 从 `apt.llvm.org` 安装 LLVM 19。
2. 同时, 安装 `libpolly-19-dev` 包。
3. 运行
   ```bash
   cargo build -r
   ```
   以构建优化的版本。