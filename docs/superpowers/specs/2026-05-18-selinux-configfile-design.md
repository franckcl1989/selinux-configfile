# /etc/selinux/config 文件格式规范

基于以下官方来源：
- `selinux_config(5)` man page (policycoreutils)
- libselinux 源码 (`selinux_config.c`)
- Red Hat SELinux 管理指南

---

## 文件位置

```
/etc/selinux/config
```

文件不存在或损坏时，不加载 SELinux 策略（等同于 disabled）。

---

## 文件格式

- `KEY = VALUE` 格式（`=` 两侧空格可选）
- `#` 开头的行为注释行
- 空行忽略
- key 大小写：libselinux 中 `SELINUX=` 和 `REQUIRESEUSERS=` 用 `strncmp`（大小写敏感），`SELINUXTYPE=` 用 `strncasecmp`（大小写不敏感）。**建议统一大小写不敏感。**
- value 前后空白会被 strip
- value 尾部控制字符（如 `\r`）会被 strip

---

## 标准参数

### 1. SELINUX（必须）

控制 SELinux 执行状态。

| 值 | 含义 |
|---|---|
| `enforcing` | 强制执行 SELinux 安全策略 |
| `permissive` | 不强制执行，但记录警告日志 |
| `disabled` | 不加载 SELinux 策略（**已废弃**，推荐用内核参数 `selinux=0`） |

### 2. SELINUXTYPE（必须）

策略类型名称，也是 `/etc/selinux/` 下的策略目录名。

| 值 | 含义 |
|---|---|
| `targeted` | 默认值（libselinux 中 `SELINUXDEFAULT = "targeted"`） |
| `mls` | 多级安全策略 |
| `minimum` | 最小策略 |
| 其他自定义值 | 任意合法目录名 |

### 3. REQUIRESEUSERS（可选）

控制 `getseuserbyname(3)` 在没有匹配 SELinux 用户时的行为。

| 值 | 含义 |
|---|---|
| `0` 或不设置 | 返回 Linux 用户名作为 SELinux 用户名 |
| `1` | 调用失败，导致登录程序（如 PAM）拒绝访问 |

### 4. AUTORELABEL（可选）

控制发现 `/.autorelabel` 文件时是否自动重标记文件系统。

| 值 | 含义 |
|---|---|
| `0` | 跳到 root shell 让管理员手动 relabel |
| `1` 或不设置（默认） | 自动执行 `fixfiles -F restore` 来 relabel |

### 5. SETLOCALDEFS（可选，已废弃）

| 值 | 含义 |
|---|---|
| `0` | 不加载本地定制（推荐） |
| `1` | `selinux_mkload_policy(3)` 读取本地 booleans/users 定制（已废弃） |

> 注意：SETLOCALDEFS 在 libselinux 源码中**未被解析**，仅出现在部分发行版文档中。本库可支持读写但不作为核心参数。

---

## 最小示例

```
# SELinux configuration
SELINUX = enforcing
SELINUXTYPE = targeted
```

---

## libselinux 解析行为总结

| 行为 | 实现 |
|---|---|
| 文件打开 | `fopen(SELINUXCONFIG, "re")` |
| 行读取 | `getline` 动态分配，替换尾部 `\n` 为 `\0` |
| 行首空白 | `isspace` 跳过 |
| 注释/空行 | `#` 开头或 `\0` 开头的行跳过 |
| key 匹配 | `SELINUXTYPE=` 大小写不敏感；其余大小写敏感（libselinux 不一致，本库建议统一不敏感） |
| value 前空白 | `isspace` 跳过 |
| value 后空白和控制字符 | 从尾部向前抹除 |
| SELINUX 值匹配 | 大小写不敏感 (`strncasecmp`) |
| REQUIRESEUSERS 值解析 | `atoi` 若首字符为数字；否则 `"true"` → 1, `"false"` → 0 (大小写不敏感) |
| 默认 SELINUXTYPE | `"targeted"` |

---

## 设计原则

### 安全约束

- **100% safe Rust**：代码中不允许 `unsafe` 块
- **线程安全**：所有公开类型实现 `Send + Sync`，文件写入使用原子操作
- **内存安全**：零 `unsafe`，所有解析基于 safe Rust 字符串操作
- **类型安全**：标准 key 全部使用类型化 getter/setter（如 `SelinuxMode` 枚举代替裸字符串），杜绝运行时值错误

### 格式保持

**这是本库最核心的功能约束。** 所有写操作必须保持原有文件格式不受破坏：

- 注释行原样保留，位置不变
- 空行原样保留，位置不变
- key-value 行原有的等号两侧空格格式保留
- 未修改的 key 行原样保留
- 修改 key 时只替换 value 部分，key 名和分隔符格式不变
- 序列化时行的顺序不变

---

## API 设计

### 关键行为规则

- **key 匹配大小写不敏感**：`get("selinux")`、`get("SELINUX")`、`get("SeLinux")` 等效。匹配时用归一化比较，回写时保留原始大小写。
- **已知 key 的名称归一化**：`set()` 若匹配到已知标准 key（SELINUX / SELINUXTYPE / REQUIRESEUSERS / AUTORELABEL / SETLOCALDEFS），自动使用标准大写形式写入。这确保与 libselinux 大小写敏感解析的兼容性。未知 key 保留调用方传入的大小写。
- **重复 key 后值覆盖前值**：与 libselinux 行为一致（last wins）。`get()` 返回最后一个，`set()` 更新最后一个匹配项（就地修改，不删除前项以保持格式）。`remove()` 和 `disable()` 删除/注释**所有**匹配项。
- **`read_default()` 文件不存在时返回 `Ok(ConfigFile::new())`**（空 config），不报错。与 libselinux 一致：文件缺失 = 无策略加载。
- **`new()` 创建完全空的 config**。写入空 config 将产生无条目文件→系统不加载策略。如需带默认值，使用 `ConfigFile::default()` 返回 `SELINUX=enforcing, SELINUXTYPE=targeted` 的最小配置。
- **行尾兼容 `\r\n`**：解析时归一化处理，序列化时统一输出 `\n`。
- **内联注释保留**：`SELINUX = enforcing  # mode` 中的 `  # mode` 存入 `raw_suffix`，修改 value 后原样写回。
- **无 `=` 的行归入 `Raw` 变体**：原样保留不做解析，序列化时原文输出。
- **`remove("SELINUX")` 和 `remove("SELINUXTYPE")` 允许执行但不推荐**：这两个 key 被 libselinux 视为必须，移除后系统可能不加载策略。文档警告即可，不做硬性阻止。
- **修改 value 为相同值时仍然标记为已修改**：不做 dirty 检测，保持简单。调用方自行判断。

---

### 类型定义

```rust
/// SELinux 执行模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelinuxMode {
    Enforcing,
    Permissive,
    Disabled,
}

impl SelinuxMode {
    /// 从字符串解析，大小写不敏感
    pub fn from_str(s: &str) -> Result<SelinuxMode, ValueError>;
}

impl fmt::Display for SelinuxMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl FromStr for SelinuxMode {
    type Err = ValueError;
    fn from_str(s: &str) -> Result<Self, Self::Err>;
}

/// 标准 key 名称常量
pub const SELINUX_KEY: &str = "SELINUX";
pub const SELINUXTYPE_KEY: &str = "SELINUXTYPE";
pub const REQUIRESEUSERS_KEY: &str = "REQUIRESEUSERS";
pub const AUTORELABEL_KEY: &str = "AUTORELABEL";
pub const SETLOCALDEFS_KEY: &str = "SETLOCALDEFS";

/// SELINUXTYPE 默认值
pub const SELINUXTYPE_DEFAULT: &str = "targeted";

/// 文件中的一行
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    /// 注释行（含前导空白和 `#`）
    Comment(String),
    /// 空行或仅含空白（含换行符）
    Blank(String),
    /// 无法识别格式的行，原样保留不做解析
    Raw(String),
    /// key-value 条目，保留原始格式元数据用于精确回写
    Entry {
        /// 原始 key（保留大小写），用于精确回写
        key_raw: String,
        /// 逻辑值（已 strip 空白和内联注释）
        value: String,
        /// 行首至 key 之间的文本（如缩进空白）
        raw_leading: String,
        /// key 和 value 之间的分隔符原文（如 " = " 或 "="）
        raw_separator: String,
        /// value 之后至行尾的全部原文（含内联注释、尾部空白、换行符）
        /// 例如 "  # my comment\n"
        raw_suffix: String,
    },
}
```

### 构造与 IO

```rust
impl ConfigFile {
    /// 创建空的 config（不含任何条目）
    pub fn new() -> Self;

    /// 创建带最小默认值的 config：SELINUX=enforcing, SELINUXTYPE=targeted
    pub fn default() -> Self;

    /// 从字符串解析
    pub fn parse(input: &str) -> Result<Self, ParseError>;

    /// 从文件读取
    pub fn read_from(path: impl AsRef<Path>) -> Result<Self, IoError>;

    /// 便捷方法：从 /etc/selinux/config 读取。文件不存在返回 Ok(ConfigFile::new())
    pub fn read_default() -> Result<Self, IoError>;

    /// 序列化到字符串（精确还原格式）
    pub fn to_string(&self) -> String;

    /// 原子写入到文件（write tmp + fsync + rename）
    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<(), IoError>;

    /// 便捷方法：写回 /etc/selinux/config
    pub fn write_default(&self) -> Result<(), IoError>;
}
```

### 类型化查询

```rust
impl ConfigFile {
    /// SELINUX 值，未设置返回 None
    pub fn selinux(&self) -> Option<SelinuxMode>;

    /// SELINUXTYPE 值
    pub fn selinuxtype(&self) -> Option<&str>;

    /// REQUIRESEUSERS 值
    pub fn require_seusers(&self) -> Option<bool>;

    /// AUTORELABEL 值
    pub fn autorelabel(&self) -> Option<bool>;

    /// SETLOCALDEFS 值
    pub fn setlocaldefs(&self) -> Option<bool>;
}
```

### 类型化修改（含校验）

```rust
impl ConfigFile {
    /// 设置 SELINUX，值已类型安全无需额外校验
    pub fn set_selinux(&mut self, mode: SelinuxMode);

    /// 设置 SELINUXTYPE，校验：非空、非纯空白
    pub fn set_selinuxtype(&mut self, policy_type: &str) -> Result<(), ValueError>;

    /// 设置 REQUIRESEUSERS
    pub fn set_require_seusers(&mut self, value: bool);

    /// 设置 AUTORELABEL
    pub fn set_autorelabel(&mut self, value: bool);

    /// 设置 SETLOCALDEFS
    pub fn set_setlocaldefs(&mut self, value: bool);
}
```

### 通用操作

```rust
impl ConfigFile {
    /// 获取任意 key 的逻辑值，key 匹配大小写不敏感，存在重复 key 时返回最后一个
    pub fn get(&self, key: &str) -> Option<&str>;

    /// 设置任意 key 的值，key 存在则就地修改最后一个匹配项的 value（保留格式），
    /// key 不存在则追加到文件末尾，新行格式为 `KEY=VALUE\n`
    pub fn set(&mut self, key: &str, value: &str);

    /// 移除 key 条目（整行删除，包括该行前后的关联注释不受影响）。
    /// 返回 true 表示找到并删除了，false 表示 key 不存在。
    /// 对于 SELINUX/SELINUXTYPE：可执行但系统可能因此不加载策略，调用方自行负责。
    pub fn remove(&mut self, key: &str) -> bool;

    /// 注释掉 key（在行首添加 "# "），等效于 disable 该配置。
    /// 返回 true 表示找到并注释了，false 表示 key 不存在。
    pub fn disable(&mut self, key: &str) -> bool;

    /// 文件是否没有任何 key-value 条目
    pub fn is_empty(&self) -> bool;

    /// 所有去重后的 key 列表（保持文件中首次出现的顺序）
    pub fn keys(&self) -> Vec<&str>;

    /// 是否包含某个 key（大小写不敏感）
    pub fn contains(&self, key: &str) -> bool;

    /// 所有行的切片，用于遍历（含注释、空行、条目、无法识别行）
    pub fn lines(&self) -> &[Line];

    /// 在末尾追加一条注释行，自动补 `# ` 前缀和换行
    pub fn add_comment_line(&mut self, comment: &str);

    /// 在末尾追加一个空行
    pub fn add_blank_line(&mut self);
}
```

### 校验

```rust
impl ConfigFile {
    /// 校验整个文件：检查所有已设置 key 的值是否合法。
    /// 返回所有发现的错误，空 Vec 表示全部通过。
    pub fn validate(&self) -> Vec<ValueError>;
}
```

### 错误类型

```rust
/// 解析错误，包含行号和描述
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

/// 值校验错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueError {
    pub key: String,
    pub message: String,
}

/// IO 错误
#[derive(Debug)]
pub struct IoError {
    pub path: PathBuf,
    pub source: std::io::Error,
}

// 所有错误类型实现 Display + std::error::Error
impl fmt::Display for ParseError { /* ... */ }
impl std::error::Error for ParseError {}
impl fmt::Display for ValueError { /* ... */ }
impl std::error::Error for ValueError {}
impl fmt::Display for IoError { /* ... */ }
impl std::error::Error for IoError {}
```

### 典型使用示例

```rust
use selinux_configfile::ConfigFile;
use selinux_configfile::SelinuxMode;

// 读取
let mut cfg = ConfigFile::read_default()?;

// 查询
assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
assert_eq!(cfg.selinuxtype(), Some("targeted"));

// 修改
cfg.set_selinux(SelinuxMode::Permissive);
cfg.set_selinuxtype("mls")?;
cfg.set_require_seusers(true);

// 写回，注释和格式不受影响
cfg.write_default()?;
```
