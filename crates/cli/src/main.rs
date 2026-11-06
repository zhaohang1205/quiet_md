use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use fdl_core::editor::core::Core;
use fdl_core::model::{Align, Document};
use fdl_core::store::Store;
use serde::Serialize;
use std::io::{IsTerminal, Read};
use std::path::PathBuf;

mod doc_ops;
mod format;
mod image_ops;

use doc_ops::{
    compute_doc_stats, delete_block_in_core, insert_blocks_with_pos, resolve_doc_id, update_block_in_core,
    BlockUpdate, InsertPosition,
};
use format::{build_block_tree, print_block_tree};
use image_ops::{create_image_block, ImageOptions};

#[derive(Parser)]
#[command(
    name = "quiet",
    author,
    version,
    about = "Command-line interface for Quiet.md, built for scripts and AI agents"
)]
pub struct Cli {
    /// Library directory path (defaults to QUIET_MD_HOME or ~/Documents/Quiet.md)
    #[arg(long, short = 'L', global = true)]
    pub library: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum AlignArg {
    Left,
    Center,
    Right,
}

impl From<AlignArg> for Align {
    fn from(a: AlignArg) -> Self {
        match a {
            AlignArg::Left => Align::Left,
            AlignArg::Center => Align::Center,
            AlignArg::Right => Align::Right,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub enum FormatArg {
    Md,
    Xml,
    Json,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List documents in the library
    List {
        /// Parent document ID filter
        #[arg(long)]
        parent: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Read document content
    Read {
        /// Document title or ID
        doc: String,
        /// Output format (md, xml, json)
        #[arg(long, short, value_enum, default_value = "md")]
        format: FormatArg,
    },
    /// Inspect block tree structure for fine-grained targeting
    Inspect {
        /// Document title or ID
        doc: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Add an image block with fine-grained controls
    AddImage {
        /// Document title or ID
        doc: String,
        /// Image file path
        path: PathBuf,
        /// Caption text
        #[arg(long)]
        caption: Option<String>,
        /// Display width in pixels
        #[arg(long)]
        width: Option<f32>,
        /// Display height in pixels
        #[arg(long)]
        height: Option<f32>,
        /// Alignment (left, center, right)
        #[arg(long, value_enum, default_value = "center")]
        align: AlignArg,
        /// Copy image to library assets folder (default: true)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        copy: bool,
        /// Insert after specified block locator (e.g. 0, 1.2, #14, first, last)
        #[arg(long)]
        after: Option<String>,
        /// Insert before specified block locator
        #[arg(long)]
        before: Option<String>,
        /// Insert inside container block (e.g. callout, quote)
        #[arg(long)]
        parent: Option<String>,
        /// Insert at the very beginning of the document
        #[arg(long)]
        first: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Insert Markdown content at a specific position
    Insert {
        /// Document title or ID
        doc: String,
        /// Markdown text (or read from stdin if omitted and no --file)
        content: Option<String>,
        /// Read markdown from file
        #[arg(long, short)]
        file: Option<PathBuf>,
        /// Insert after specified block locator
        #[arg(long)]
        after: Option<String>,
        /// Insert before specified block locator
        #[arg(long)]
        before: Option<String>,
        /// Insert inside container block
        #[arg(long)]
        parent: Option<String>,
        /// Insert at the very beginning
        #[arg(long)]
        first: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Append Markdown content to the end of a document
    Append {
        /// Document title or ID
        doc: String,
        /// Markdown text (or read from stdin if omitted and no --file)
        content: Option<String>,
        /// Read markdown from file
        #[arg(long, short)]
        file: Option<PathBuf>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Update a block's attributes, content, or type
    UpdateBlock {
        /// Document title or ID
        doc: String,
        /// Block locator (e.g. 0, 1.2, #14)
        block: String,
        /// New caption
        #[arg(long)]
        caption: Option<String>,
        /// New width
        #[arg(long)]
        width: Option<f32>,
        /// New height
        #[arg(long)]
        height: Option<f32>,
        /// New alignment
        #[arg(long, value_enum)]
        align: Option<AlignArg>,
        /// New plain text content
        #[arg(long)]
        text: Option<String>,
        /// Todo completion state (true/false)
        #[arg(long)]
        done: Option<bool>,
        /// Toggle todo completion state
        #[arg(long)]
        toggle_done: bool,
        /// Code block language
        #[arg(long)]
        lang: Option<String>,
        /// Callout emoji icon
        #[arg(long)]
        emoji: Option<String>,
        /// Heading level (1-6)
        #[arg(long)]
        level: Option<u8>,
        /// Convert block type (text, h1..h6, bullet, ordered, todo, code, quote, callout, divider)
        #[arg(long)]
        turn_into: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show document statistics and metadata
    Info {
        /// Document title or ID
        doc: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Remove a block from a document
    RemoveBlock {
        /// Document title or ID
        doc: String,
        /// Block locator
        block: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Create a new document
    Create {
        /// Document title (defaults to "未命名文档")
        title: Option<String>,
        /// Initial markdown content
        #[arg(long)]
        content: Option<String>,
        /// Import from markdown file
        #[arg(long, short)]
        file: Option<PathBuf>,
        /// Parent document ID
        #[arg(long)]
        parent: Option<String>,
        /// Document icon emoji
        #[arg(long)]
        icon: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Move a document to trash
    Delete {
        /// Document title or ID
        doc: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Search documents by title and content
    Search {
        /// Search query
        query: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Serialize)]
struct DocSummary {
    id: String,
    title: String,
    icon: Option<String>,
    pinned: bool,
    starred: u64,
    parent: Option<String>,
    created: u64,
    modified: u64,
}

fn open_store(lib: Option<PathBuf>) -> Result<Store> {
    if let Some(p) = lib {
        Store::open_at(p)
    } else {
        Store::open()
    }
}

fn get_markdown_input(content: Option<String>, file: Option<PathBuf>) -> Result<String> {
    if let Some(c) = content {
        return Ok(c);
    }
    if let Some(f) = file {
        return std::fs::read_to_string(&f).with_context(|| format!("无法读取文件：{}", f.display()));
    }
    if !std::io::stdin().is_terminal() {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        return Ok(buf);
    }
    anyhow::bail!("请通过参数、--file 或标准输入提供 Markdown 内容")
}

fn parse_pos(after: Option<String>, before: Option<String>, parent: Option<String>, first: bool) -> InsertPosition {
    if let Some(loc) = after {
        InsertPosition::After(loc)
    } else if let Some(loc) = before {
        InsertPosition::Before(loc)
    } else if let Some(loc) = parent {
        InsertPosition::Parent(loc)
    } else if first {
        InsertPosition::First
    } else {
        InsertPosition::End
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut store = open_store(cli.library)?;

    match cli.command {
        Commands::List { parent, json } => {
            let mut entries: Vec<_> = store
                .list()
                .into_iter()
                .filter(|e| parent.is_none() || e.meta.parent == parent)
                .map(|e| DocSummary {
                    id: e.id,
                    title: fdl_core::model::display_title(&e.title),
                    icon: e.icon,
                    pinned: e.meta.pinned,
                    starred: e.meta.starred,
                    parent: e.meta.parent,
                    created: e.meta.created,
                    modified: e.meta.modified,
                })
                .collect();
            entries.sort_by_key(|e| (std::cmp::Reverse(e.pinned), std::cmp::Reverse(e.modified)));

            if json {
                println!("{}", serde_json::to_string_pretty(&entries)?);
            } else {
                for e in entries {
                    let pin = if e.pinned { "📌 " } else { "" };
                    let icon = e.icon.as_deref().unwrap_or("📄");
                    println!("{pin}{icon} {} ({})", e.title, e.id);
                }
            }
        }
        Commands::Read { doc, format } => {
            let doc_id = resolve_doc_id(&mut store, &doc)?;
            let d = store.load(&doc_id)?;
            match format {
                FormatArg::Md => {
                    let export = fdl_core::markdown::export(&d, &Default::default());
                    print!("{}", export.text);
                }
                FormatArg::Xml => {
                    let xml = fdl_core::docxml::serialize(&d);
                    print!("{xml}");
                }
                FormatArg::Json => {
                    let nodes = build_block_tree(&d.blocks, "");
                    let obj = serde_json::json!({
                        "id": doc_id,
                        "title": d.title.plain_text(),
                        "icon": d.icon,
                        "blocks": nodes,
                    });
                    println!("{}", serde_json::to_string_pretty(&obj)?);
                }
            }
        }
        Commands::Inspect { doc, json } => {
            let doc_id = resolve_doc_id(&mut store, &doc)?;
            let d = store.load(&doc_id)?;
            let nodes = build_block_tree(&d.blocks, "");
            if json {
                println!("{}", serde_json::to_string_pretty(&nodes)?);
            } else {
                println!("文档: {} ({})\n---", d.title.plain_text(), doc_id);
                print_block_tree(&nodes, 0);
            }
        }
        Commands::AddImage { doc, path, caption, width, height, align, copy, after, before, parent, first, json } => {
            let doc_id = resolve_doc_id(&mut store, &doc)?;
            let d = store.load_for_edit(&doc_id, false)?;
            let mut core = Core::new(d);

            let (block, src) = create_image_block(
                &path,
                &store.assets_path(),
                ImageOptions { caption, width, height, align: align.into(), copy },
            )?;

            let pos = parse_pos(after, before, parent, first);
            let inserted_ids = insert_blocks_with_pos(&mut core, vec![block], pos)?;
            store.save(&doc_id, &core.doc)?;

            let block_id = inserted_ids[0].0;
            if json {
                let res = serde_json::json!({
                    "status": "ok",
                    "doc_id": doc_id,
                    "block_id": block_id,
                    "src": src,
                });
                println!("{}", serde_json::to_string_pretty(&res)?);
            } else {
                println!("已添加图片 (Block #{block_id}) -> {src}");
            }
        }
        Commands::Insert { doc, content, file, after, before, parent, first, json } => {
            let doc_id = resolve_doc_id(&mut store, &doc)?;
            let text = get_markdown_input(content, file)?;
            let blocks = fdl_core::paste::markdown_to_blocks(&text, fdl_core::paste::Off { file: true, ..Default::default() });
            anyhow::ensure!(!blocks.is_empty(), "解析 Markdown 后的块为空");

            let d = store.load_for_edit(&doc_id, false)?;
            let mut core = Core::new(d);
            let pos = parse_pos(after, before, parent, first);
            let inserted_ids = insert_blocks_with_pos(&mut core, blocks, pos)?;
            store.save(&doc_id, &core.doc)?;

            let raw_ids: Vec<u64> = inserted_ids.iter().map(|id| id.0).collect();
            if json {
                let res = serde_json::json!({
                    "status": "ok",
                    "doc_id": doc_id,
                    "block_ids": raw_ids,
                });
                println!("{}", serde_json::to_string_pretty(&res)?);
            } else {
                println!("已插入 {} 个块 (ID: {:?})", raw_ids.len(), raw_ids);
            }
        }
        Commands::Append { doc, content, file, json } => {
            let doc_id = resolve_doc_id(&mut store, &doc)?;
            let text = get_markdown_input(content, file)?;
            let blocks = fdl_core::paste::markdown_to_blocks(&text, fdl_core::paste::Off { file: true, ..Default::default() });
            anyhow::ensure!(!blocks.is_empty(), "解析 Markdown 后的块为空");

            let d = store.load_for_edit(&doc_id, false)?;
            let mut core = Core::new(d);
            let inserted_ids = insert_blocks_with_pos(&mut core, blocks, InsertPosition::End)?;
            store.save(&doc_id, &core.doc)?;

            let raw_ids: Vec<u64> = inserted_ids.iter().map(|id| id.0).collect();
            if json {
                let res = serde_json::json!({
                    "status": "ok",
                    "doc_id": doc_id,
                    "block_ids": raw_ids,
                });
                println!("{}", serde_json::to_string_pretty(&res)?);
            } else {
                println!("已追加 {} 个块 (ID: {:?})", raw_ids.len(), raw_ids);
            }
        }
        Commands::UpdateBlock {
            doc,
            block,
            caption,
            width,
            height,
            align,
            text,
            done,
            toggle_done,
            lang,
            emoji,
            level,
            turn_into,
            json,
        } => {
            let doc_id = resolve_doc_id(&mut store, &doc)?;
            let d = store.load_for_edit(&doc_id, false)?;
            let mut core = Core::new(d);

            let updated_id = update_block_in_core(
                &mut core,
                &block,
                BlockUpdate {
                    caption,
                    width,
                    height,
                    align: align.map(Into::into),
                    text,
                    done,
                    toggle_done,
                    lang,
                    emoji,
                    level,
                    turn_into,
                },
            )?;
            store.save(&doc_id, &core.doc)?;

            if json {
                let res = serde_json::json!({
                    "status": "ok",
                    "doc_id": doc_id,
                    "block_id": updated_id.0,
                });
                println!("{}", serde_json::to_string_pretty(&res)?);
            } else {
                println!("已更新块 #{}", updated_id.0);
            }
        }
        Commands::Info { doc, json } => {
            let doc_id = resolve_doc_id(&mut store, &doc)?;
            let d = store.load(&doc_id)?;
            let entries = store.list();
            let entry = entries.iter().find(|e| e.id == doc_id);
            let created = entry.map(|e| e.meta.created).unwrap_or(0);
            let modified = entry.map(|e| e.meta.modified).unwrap_or(0);
            let stats = compute_doc_stats(&store, &doc_id, &d, created, modified);

            if json {
                println!("{}", serde_json::to_string_pretty(&stats)?);
            } else {
                let icon = stats.icon.as_deref().unwrap_or("📄");
                println!("{icon} 文档信息: {} ({})", stats.title, stats.id);
                println!("---");
                println!("字数:        {} 字", stats.words);
                println!("字符数:      {} 字符", stats.chars);
                println!("块数量:      {} 个", stats.blocks_count);
                println!("磁盘大小:    {} 字节", stats.size_bytes);
                if !stats.assets.is_empty() {
                    println!("引用资产 ({} 个):", stats.assets.len());
                    for a in &stats.assets {
                        println!("  - {a}");
                    }
                }
            }
        }
        Commands::RemoveBlock { doc, block, json } => {
            let doc_id = resolve_doc_id(&mut store, &doc)?;
            let d = store.load_for_edit(&doc_id, false)?;
            let mut core = Core::new(d);

            let deleted_id = delete_block_in_core(&mut core, &block)?;
            store.save(&doc_id, &core.doc)?;

            if json {
                let res = serde_json::json!({
                    "status": "ok",
                    "doc_id": doc_id,
                    "deleted_block_id": deleted_id.0,
                });
                println!("{}", serde_json::to_string_pretty(&res)?);
            } else {
                println!("已删除块 #{}", deleted_id.0);
            }
        }
        Commands::Create { title, content, file, parent, icon, json } => {
            let mut d = Document::default();
            let title_text = title.unwrap_or_else(|| "未命名文档".to_string());
            d.title = fdl_core::model::RichText::plain(&title_text);
            if let Some(ic) = icon {
                d.icon = Some(ic);
            }
            if content.is_some() || file.is_some() {
                let text = get_markdown_input(content, file)?;
                let blocks = fdl_core::paste::markdown_to_blocks(&text, fdl_core::paste::Off { file: true, ..Default::default() });
                d.blocks = blocks;
            }
            let doc_id = store.create_from(&d)?;
            if let Some(p) = parent {
                let _ = store.set_parent(&doc_id, Some(&p));
            }

            if json {
                let res = serde_json::json!({
                    "status": "ok",
                    "doc_id": doc_id,
                    "title": d.title.text,
                });
                println!("{}", serde_json::to_string_pretty(&res)?);
            } else {
                println!("已创建文档: {} ({})", d.title.text, doc_id);
            }
        }
        Commands::Delete { doc, json } => {
            let doc_id = resolve_doc_id(&mut store, &doc)?;
            store.delete(&doc_id, None)?;
            if json {
                let res = serde_json::json!({
                    "status": "ok",
                    "deleted_doc_id": doc_id,
                });
                println!("{}", serde_json::to_string_pretty(&res)?);
            } else {
                println!("文档已移入回收站: {doc_id}");
            }
        }
        Commands::Search { query, json } => {
            let q = query.to_lowercase();
            let matches: Vec<_> = store
                .list()
                .into_iter()
                .filter(|e| e.title.to_lowercase().contains(&q) || e.body.to_lowercase().contains(&q))
                .map(|e| {
                    let snippet = e
                        .body
                        .lines()
                        .find(|l| l.to_lowercase().contains(&q))
                        .unwrap_or_else(|| e.body.lines().next().unwrap_or(""))
                        .chars()
                        .take(80)
                        .collect::<String>();
                    serde_json::json!({
                        "id": e.id,
                        "title": e.title,
                        "snippet": snippet,
                        "modified": e.meta.modified,
                    })
                })
                .collect();

            if json {
                println!("{}", serde_json::to_string_pretty(&matches)?);
            } else {
                println!("找到 {} 篇相关文档：", matches.len());
                for m in matches {
                    println!("- {} ({})\n  {}", m["title"].as_str().unwrap_or(""), m["id"].as_str().unwrap_or(""), m["snippet"].as_str().unwrap_or(""));
                }
            }
        }
    }

    Ok(())
}
