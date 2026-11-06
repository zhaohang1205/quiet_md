# Quiet CLI 参数完整速查手册

本文档为 `quiet` 命令行工具的详尽参考，供 AI Agent 或开发者按需深入查阅。

---

## 全局选项

- `-L, --library <PATH>`：显式指定 Quiet.md 知识库根目录。默认读取 `QUIET_MD_HOME`，若无则使用 `~/Documents/Quiet.md`。
- `-h, --help`：打印命令帮助。
- `-V, --version`：打印版本号。

---

## 命令详解与参数列表

### 1. `list`
列出知识库内的文档。
- `--parent <DOC_ID>`：仅列出指定父文档的直接子文档。
- `--json`：输出 JSON 数组，包含所有文档的 `id`, `title`, `icon`, `pinned`, `starred`, `parent`, `created`, `modified`。

### 2. `read <DOC>`
读取文档完整内容。
- `<DOC>`：文档标题或文档 ID。
- `-f, --format <FORMAT>`：输出格式，支持 `md`（默认）、`xml`（原始 DocxXML）、`json`（Block 树结构）。

### 3. `inspect <DOC>`
探测文档树状结构。
- `<DOC>`：文档标题或文档 ID。
- `--json`：输出 JSON 树形结构，每个节点包含 `path`, `id`, `kind`, `align`, `text`, `caption`, `src`, `width`, `height`, `children`。

### 4. `add-image <DOC> <PATH>`
向文档插入图片。
- `<DOC>`：文档标题或文档 ID。
- `<PATH>`：本地图片文件路径。
- `--caption <TEXT>`：图片题注。
- `--width <FLOAT>` / `--height <FLOAT>`：显示尺寸。
- `--align <left|center|right>`：对齐方式（默认 `center`）。
- `--copy <true|false>`：是否拷贝至库的 `assets/` 目录（默认 `true`）。
- `--after <LOC>` / `--before <LOC>` / `--parent <LOC>` / `--first`：指定插入位置。
- `--json`：输出包含 `status`, `doc_id`, `block_id`, `src` 的 JSON。

### 5. `insert <DOC> [CONTENT]`
在指定位置插入 Markdown 块。
- `<DOC>`：文档标题或文档 ID。
- `[CONTENT]`：Markdown 文本（支持通过 `--file` 或标准输入管道传入）。
- `-f, --file <PATH>`：从文件读取 Markdown。
- `--after <LOC>` / `--before <LOC>` / `--parent <LOC>` / `--first`：插入位置定位器。
- `--json`：返回插入的所有 `block_ids`。

### 6. `append <DOC> [CONTENT]`
向文档末尾追加 Markdown 块。
- `<DOC>`：文档标题或文档 ID。
- `[CONTENT]`：Markdown 文本。
- `-f, --file <PATH>`：从文件读取 Markdown。
- `--json`：返回追加的 `block_ids`。

### 7. `update-block <DOC> <BLOCK>`
更新单个块的属性、内容或转换块类型。
- `<DOC>`：文档标题或文档 ID。
- `<BLOCK>`：块定位器（如 `0`, `1.2`, `#14`）。
- `--text <TEXT>`：更新文本内容。
- `--caption <TEXT>`：更新图片或代码块题注。
- `--width <FLOAT>` / `--height <FLOAT>`：更新图片尺寸。
- `--align <left|center|right>`：更新对齐方式。
- `--done <true|false>`：设置 Todo 完成状态。
- `--toggle-done`：切换 Todo 完成状态。
- `--lang <LANG>`：设置代码块语言。
- `--emoji <EMOJI>`：设置高亮块图标。
- `--level <1-6>`：设置标题级别。
- `--turn-into <KIND>`：转换块类型（`text`, `h1`..`h6`, `bullet`, `ordered`, `todo`, `code`, `quote`, `callout`, `divider`）。
- `--json`：输出结果 JSON。

### 8. `remove-block <DOC> <BLOCK>`
删除指定块。
- `<DOC>`：文档标题或文档 ID。
- `<BLOCK>`：块定位器。
- `--json`：输出已删除的 `deleted_block_id`。

### 9. `create [TITLE]`
新建文档。
- `[TITLE]`：文档标题（缺省时为 `"未命名文档"`）。
- `--content <TEXT>` / `-f, --file <PATH>`：初始 Markdown 正文。
- `--parent <DOC_ID>`：指定父文档 ID。
- `--icon <EMOJI>`：指定图标表情。
- `--json`：输出 `status`, `doc_id`, `title`。

### 10. `delete <DOC>`
将文档移入回收站。
- `<DOC>`：文档标题或文档 ID。
- `--json`：输出 `status`, `deleted_doc_id`。

### 11. `search <QUERY>`
搜索文档标题与正文。
- `<QUERY>`：搜索关键词。
- `--json`：输出匹配列表（包含 `id`, `title`, `snippet`, `modified`）。

### 12. `info <DOC>`
查看文档统计信息。
- `<DOC>`：文档标题或文档 ID。
- `--json`：输出字数、字符数、块总数、资产数及大小。
