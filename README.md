# find

一个小型、无外部依赖的 Rust 文件查找工具。

## 功能

- 按文件名查找目录、文件或全部项。
- 支持模糊匹配（包含）和精确匹配（忽略 ASCII 大小写）。
- 支持多个搜索根：目录、绝对路径、相对路径或 Windows 盘符。
- 默认搜索当前目录。
- 默认跳过隐藏文件/目录。
- 会把无法遍历或搜索根不存在的警告输出到 stderr。

## 安装/构建

```bash
cargo build --release
```

可执行文件默认生成在 Cargo 目标目录，例如：

```bash
cargo run -- main.rs
```

## 用法

```text
find [标志] <关键词> [搜索根...]
```

### 标志

| 字符 | 含义 |
|---|---|
| `d` | 只搜目录 |
| `f` | 只搜文件 |
| `a` | 搜目录和文件 |
| `z` | 模糊匹配（包含） |
| `e` | 精确匹配（忽略 ASCII 大小写） |

省略标志时默认：`全部类型 + 模糊匹配`。

两个字符顺序无关，例如 `-dz`、`-zd`、`-fe`、`-ef` 都可用。

### 搜索根

不填搜索根时，默认搜索当前目录。

可以指定任意已存在目录或盘符：

```bash
cargo run -- main.rs
cargo run -- main.rs src
cargo run -- -e main.rs C: D:
cargo run -- config -dz . E:\project
```

### 以 `-` 开头的关键词

如果关键词本身以 `-` 开头，把关键词放在 `--` 后面：

```bash
cargo run -- -- -my-file.txt
```

### 帮助与版本

```bash
cargo run -- --help
cargo run -- --version
```

## 匹配规则

- 模糊匹配：文件名包含关键词，忽略 ASCII 大小写。
- 精确匹配：文件名等于关键词，忽略 ASCII 大小写。
- 匹配对象是文件名，不是完整路径。
- 默认不会搜索隐藏文件/目录，例如 `.git`、`.env`。

## 退出码

- `0`：搜索完成且没有警告。
- `1`：命令参数错误，或搜索过程中存在警告，例如搜索根不存在、无法遍历某些目录。

## 开发

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
