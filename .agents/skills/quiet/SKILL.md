---
name: quiet
description: 专为 AI Agent 设计的 Quiet.md 本地知识库与富文本笔记操作技能。当需要查询知识库、阅读或搜索笔记、记录灵感/总结/架构设计、创建文档树、操作待办任务（Todo）、插入图片与图表、或精细更新指定内容块时使用。提供以 Markdown 为媒介的最高效读写流程与 Block 级精准定位实践。
---

# Quiet.md Agent 交互与知识库管理指南

Quiet.md 是一款原生 GPU 绘制、本地优先（Local-First）、兼容飞书 DocxXML 格式的富文本个人知识库应用。

本技能指导 AI Agent 以**最低 Token 开销、最高结构保真度、零 GUI 冲突**的方式操作 Quiet.md。

---

## 核心设计哲学（Agent 必读）

1. **以 Markdown 为交互媒介，以 DocxXML 为底层存储**：
   - 优先通过 `quiet read "<doc>" --format md` 阅读笔记（Token 消耗最小、语义最清晰）；
   - 写入与增补时输入 Markdown，底层由 `fdl_core` 自动解析为结构化 Block。
2. **细粒度局部修改，严禁盲目全篇覆盖**：
   - 编辑已有文档时，必须遵循：**“读/查 -> 探测结构 (`inspect`) -> 定位插入/更新 (`insert`/`update-block`) -> 验证 (`info`)”**；
   - 避免将整篇文档读出后全篇重写，以防破坏原生富文本属性（封面、折叠状态、画廊排版等）。
3. **并发安全与无损协作**：
   - CLI 自动继承底层 flock 文件锁，当用户在桌面上打开 Quiet.md GUI 窗口时，Agent 在后台通过 CLI 修改仍能**安全并发，互不冲突**。
4. **Agent 模式默认开启 `--json`**：
   - 程序化调用请一律追加 `--json` 参数，获取干净结构化的数据，便于提取文档 ID、块路径与修改状态。

---

## 典型场景与工作流决策树

```
需要与 Quiet.md 交互？
├── 查找或回忆已有内容
│   ├── 检索关键词/概念 ──> `quiet search "<query>" --json`
│   └── 梳理文档树与层级 ──> `quiet list --json` (按 parent 梳理层级)
│
├── 查阅某篇具体笔记
│   ├── 理解正文内容 ──> `quiet read "<doc>" --format md`
│   └── 分析排版与块结构 ──> `quiet inspect "<doc>" --json`
│
├── 局部修改已有笔记
│   ├── 1. 结构探测 ──> `quiet inspect "<doc>" --json` 获取定位器 [locator]
│   ├── 2. 局部增改：
│   │   ├── 插入段落/小节 ──> `quiet insert "<doc>" --after <loc> --content "..."`
│   │   ├── 插入图片/图表 ──> `quiet add-image "<doc>" <path> --caption "..." --after <loc>`
│   │   ├── 勾选/切换待办 ──> `quiet update-block "<doc>" <loc> --done true` 或 `--toggle-done`
│   │   ├── 更新代码语言 ──> `quiet update-block "<doc>" <loc> --lang <lang>`
│   │   ├── 转换块类型 ──> `quiet update-block "<doc>" <loc> --turn-into <kind>`
│   │   └── 删除过期块 ──> `quiet remove-block "<doc>" <loc>`
│   └── 3. 统计确认 ──> `quiet info "<doc>" --json`
│
├── 沉淀新内容/创建体系
│   ├── 独立文档 ──> `quiet create "<title>" --icon "📝" --content "..." --json`
│   └── 子文档/专栏 ──> `quiet create "<title>" --parent "<parent_id>" --json`
│
└── 简易文末追加
    └── 追加更新记录/灵感 ──> `quiet append "<doc>" --content "..."`
```

---

## 定位器（Locator）使用规范

在执行 `add-image`、`insert`、`update-block`、`remove-block` 时，需通过定位器指定目标块：

| 定位器形式 | 示例 | 含义与适用场景 |
|---|---|---|
| **树路径索引** | `0`、`1`、`2.0` | **推荐**。直观反映块的展示层级。`0` 表示第 1 个顶层块，`2.0` 表示第 3 个顶层块的第 1 个子块（如 Quote、Callout 内的子块）。 |
| **块 ID 标识** | `#14` 或 `14` | 针对特定唯一内存/存储 ID，在复杂多重嵌套中不易产生歧义。 |
| **边界关键字** | `first`、`last` | 快速指定文档的第一块或最后一块。 |

---

## 高频命令速查与模式范例

### 1. 结构化检索与读取

```bash
# 全文检索并获取摘要匹配
quiet search "架构设计" --json

# 读取 Markdown 格式正文
quiet read "架构设计"

# 探测 Block 树层级
quiet inspect "架构设计" --json
```

### 2. 插入图片（多模态与图表归档）
当用户生成了架构图、流程图或信息图时，使用 `add-image` 将其归档至笔记：

```bash
# 自动复制进 assets/ 目录、设置 800px 宽度并插入在第 1 块之后
quiet add-image "架构设计" ./diagram.png \
  --caption "系统分层拓扑图" \
  --width 800 \
  --align center \
  --after 1 \
  --json
```

### 3. 精准操作待办清单（Todo）

```bash
# 1. 探测待办所在定位器
quiet inspect "项目周报"
# 假设输出：[2.1] #18 Todo [ ]: 完成单元测试

# 2. 将其标记为已完成
quiet update-block "项目周报" 2.1 --done true

# 3. 或使用 toggle 快速翻转状态
quiet update-block "项目周报" 2.1 --toggle-done
```

### 4. 局部插入 Markdown 块

```bash
# 在第 2 块之后插入新小节
quiet insert "架构设计" --after 2 --content $'### 核心数据流\n1. 接收客户端请求\n2. 经由中间件鉴权\n3. 写入持久化存储'
```

### 5. 创建多级文档树

```bash
# 1. 创建父文档
PARENT_ID=$(quiet create "Rust 开发指南" --icon "🦀" --content "# 概览" --json | jq -r .doc_id)

# 2. 在其下方创建子文档
quiet create "内存模型" --parent "$PARENT_ID" --icon "🧠" --content "# 所有权机制" --json
quiet create "并发原语" --parent "$PARENT_ID" --icon "⚡" --content "# Mutex 与 Channel" --json
```

### 6. 查看文档元数据与资源统计

```bash
quiet info "架构设计" --json
```
返回：
```json
{
  "id": "9bc3efa8-1bd1-4ef4-8dc5-219d9d00cf14",
  "title": "架构设计",
  "icon": "📐",
  "words": 1250,
  "chars": 4820,
  "blocks_count": 34,
  "assets_count": 2,
  "assets": [
    "assets/5d035df3-d4d1-419b-a320-7f24ad6b4c10.png"
  ],
  "size_bytes": 105432,
  "created": 1730894000,
  "modified": 1730895200
}
```

---

## 避坑指南与最佳实践

1. **标题解析与歧义处理**：
   - 如果库内存在多篇重名文档，`quiet` 会报错并列出候选 ID。
   - **最佳实践**：在获得 `quiet search` 或 `quiet list` 的结果后，后续命令一律传确切的 `doc_id`。
2. **多库与非默认路径**：
   - 默认库路径为 `~/Documents/Quiet.md`。
   - 如需使用独立项目库或测试库，传全局参数 `-L /path/to/lib`，或设定环境变量 `export QUIET_MD_HOME="/path/to/lib"`。
3. **块类型转换支持一览**：
   - `--turn-into` 支持的值：
     - 段落：`text` / `paragraph`
     - 标题：`h1`, `h2`, `h3`, `h4`, `h5`, `h6`
     - 列表：`bullet` (无序), `ordered` (有序), `todo` (待办)
     - 格式块：`code` (代码块), `quote` (引用), `callout` (高亮块), `divider` (分割线)
