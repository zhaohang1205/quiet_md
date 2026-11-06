# Quiet CLI (`quiet`)

Quiet.md 的官方原生命令行工具，专为脚本自动化与 AI Agent 编程打造。

基于 [`fdl_core`](../core) 文档引擎构建，无头（Headless）运行，零 GUI 依赖。在后台读写笔记时自动遵循 Quiet.md 底层文件锁（flock），与桌面 GUI 应用并发运行互不冲突。

---

## 安装与编译

在 Quiet.md 仓库根目录下编译：

```bash
cargo build --release -p quiet_cli
# 二进制位于 target/release/quiet
```

可软链接至系统 PATH（如 `~/.local/bin`）：

```bash
ln -sf "$PWD/target/release/quiet" ~/.local/bin/quiet
```

---

## 快速上手

### 1. 文档库定位

- 默认读取环境变量 `QUIET_MD_HOME`，若未设置则使用默认库目录（`~/Documents/Quiet.md`）。
- 可通过全局参数 `-L` / `--library <PATH>` 显式指定文档库路径：

```bash
quiet -L /path/to/library list
```

### 2. 查看与搜索文档

```bash
# 列出文档（按置顶和修改时间排序）
quiet list

# 以 JSON 格式输出完整元数据（供 AI Agent 解析）
quiet list --json

# 全文与标题搜索
quiet search "架构设计" --json
```

### 3. 阅读文档

```bash
# 导出为清晰的 Markdown（默认格式，推荐 Agent 阅读）
quiet read "架构设计"

# 导出原始 Feishu DocxXML 格式
quiet read "架构设计" --format xml

# 导出 Block 树结构 JSON
quiet read "架构设计" --format json
```

### 4. 查看文档统计信息

```bash
quiet info "架构设计"
quiet info "架构设计" --json
```
**输出内容包含：**
- 文档 ID、标题、图标
- 字数（Word Count，飞书对齐计数规则）
- 字符数（Char Count）
- 块总数（Blocks Count）
- 磁盘占用（含正文与关联资源）
- 引用的本地图片与附件列表（Assets）
- 创建与修改时间戳

---

## 细粒度 Block 结构与控制

### 1. 结构探测（`inspect`）

修改文档前，可通过 `inspect` 查看文档块的层级结构与定位器（Locator）：

```bash
quiet inspect "架构设计"
```

输出示例：
```text
文档: 架构设计 (9bc3efa8-1bd1-4ef4-8dc5-219d9d00cf14)
---
[0] #4 Heading 1: 系统架构与模块划分
[1] #5 Paragraph: 核心文档引擎独立在 crates/core 中运行...
[2] #6 Callout (💡): 注意事项
    [2.0] #7 Todo [ ]: 请在提交前运行 scripts/check.sh
[3] #8 Code (rust): fn main() { ... }
```

> [!NOTE]
> **Locator 机制**：
> - 树路径索引：`0`（第 1 块）、`2.0`（第 3 块的第 1 个子块）
> - 块 ID：`#4` 或数字 `4`
> - 语义位置：`first`（首块）、`last`（末块）

支持 JSON 输出：
```bash
quiet inspect "架构设计" --json
```

---

### 2. 插入图片（`add-image`）

向文档插入图片，支持自定义大小、对齐、题注与插入位置：

```bash
# 自动拷贝至库的 assets/ 目录并插入末尾
quiet add-image "架构设计" ./architecture.png --caption "系统拓扑图"

# 精细控制：指定宽度、居中对齐、插入在第 0 块之后
quiet add-image "架构设计" ./architecture.png \
  --after 0 \
  --width 800 \
  --align center \
  --caption "系统拓扑图" \
  --json
```

**选项：**
- `--after <locator>`：插入在指定块之后
- `--before <locator>`：插入在指定块之前
- `--parent <locator>`：作为子块插入容器内（如 Callout 或 Quote）
- `--first`：插入在文档开头
- `--width <px>` / `--height <px>`：显示宽高（未指定时自动读取图片原生尺寸）
- `--align <left|center|right>`：对齐方式
- `--copy` / `--no-copy`：是否复制至库的 `assets/` 目录（默认 `true`）

---

### 3. 插入与追加 Markdown（`insert` / `append`）

支持单段或多段 Markdown 块的精准定位插入：

```bash
# 在指定块后插入内容
quiet insert "架构设计" --after 1 --content $'### 关键接口\n- `Store::save`\n- `Core::turn_into`'

# 从文件或标准输入导入插入
cat section.md | quiet insert "架构设计" --after 2

# 追加到文档末尾
quiet append "架构设计" --content "- 补充事项"
```

---

### 4. 更新块属性与转换类型（`update-block`）

精准修改单个块的文本、属性或转换块类型：

```bash
# 勾选 Todo 待办事项
quiet update-block "架构设计" 2.0 --done true

# 切换 Todo 勾选状态（Toggle）
quiet update-block "架构设计" 2.0 --toggle-done

# 修改代码块语言与题注
quiet update-block "架构设计" 3 --lang rust --caption "入口函数"

# 修改 Callout 高亮块图标
quiet update-block "架构设计" 2 --emoji "🚀"

# 转换块类型（turn-into：支持 text, h1..h6, bullet, ordered, todo, code, quote, callout）
quiet update-block "架构设计" 1 --turn-into h2
```

---

### 5. 删除块（`remove-block`）

```bash
quiet remove-block "架构设计" 2.0
```

---

## 文档管理

### 创建新文档（`create`）

```bash
# 创建带有初始内容的文档
quiet create "Sprint 42 计划" \
  --icon "📋" \
  --content $'# 周期目标\n- [ ] 完成 CLI 模块' \
  --json

# 创建子文档
quiet create "子模块设计" --parent "Sprint 42 计划"
```

### 删除文档（移入回收站）

```bash
quiet delete "Sprint 42 计划"
```

---

## AI Agent 集成建议

1. **结构读取**：优先使用 `quiet read <doc> --format md` 快速获取正文语义；如需结构定位，使用 `quiet inspect <doc> --json` 获取精确的 `path` 和 `id`。
2. **精细修改**：善用 `--after` / `--before` 配合路径（如 `0`、`1.2`）进行局部增补；使用 `update-block` 操纵 Todo 和代码块。
3. **安全并发**：CLI 自动处理并发互斥文件锁，可在 Quiet.md 界面打开的同时进行自动化编辑。
