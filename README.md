## 在 Ubuntu 24.04 上构建

1. 从 `apt.llvm.org` 安装 LLVM 19。
2. 同时, 安装 `libpolly-19-dev` 包。
3. （可选）中国大陆地区的开发者，可考虑使用 `crates.io` 镜像，如[清华大学镜像源](https://mirrors.tuna.tsinghua.edu.cn/help/crates.io-index/)
4. 运行
   ```bash
   cargo build -r
   ```
   以构建优化的版本。

## 使用

```bash
haru [OPTIONS] <INPUT>
```

其中：

### `<INPUT>`

一个 LLVM IR 文件。

### `[OPTIONS]`

- `-b`，`--bc`，指定输入为二进制格式的 LLVM IR。
- `-l`，`--ll`，指定输入为文本格式的 LLVM IR。

以上两个参数至多指定一个。未指定任何一个时，将根据 `<INPUT>` 文件扩展名猜测。

- `-f`，`--func <FUNC>`，指定函数名，不指定时为 `op`。
- `-i`，`--inst <INST>`，指定输出指令名，不指定时与输入函数同名。
- `-o`，`--output <OUTPUT>`，指定输出文件名，不指定时将输出到 `trans_<INST>.c`。
- `-h`，`--help`，打印帮助。
- `-V`，`-version`，显示 Haru 版本。