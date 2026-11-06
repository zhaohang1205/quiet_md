use anyhow::{Context, Result};
use fdl_core::editor::core::{Core, DropPlace};
use fdl_core::model::{display_title, Align, Block, BlockId, BlockKind, Document, RichText};
use fdl_core::store::Store;
use serde::Serialize;

pub enum InsertPosition {
    After(String),
    Before(String),
    Parent(String),
    First,
    End,
}

#[derive(Default)]
pub struct BlockUpdate {
    pub caption: Option<String>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub align: Option<Align>,
    pub text: Option<String>,
    pub done: Option<bool>,
    pub toggle_done: bool,
    pub lang: Option<String>,
    pub emoji: Option<String>,
    pub level: Option<u8>,
    pub turn_into: Option<String>,
}

#[derive(Serialize)]
pub struct DocStats {
    pub id: String,
    pub title: String,
    pub icon: Option<String>,
    pub words: usize,
    pub chars: usize,
    pub blocks_count: usize,
    pub assets_count: usize,
    pub assets: Vec<String>,
    pub size_bytes: u64,
    pub created: u64,
    pub modified: u64,
}

fn is_cjk(c: char) -> bool {
    matches!(c as u32,
        0x3400..=0x4dbf | 0x4e00..=0x9fff | 0xf900..=0xfaff | 0x20000..=0x2fa1f
        | 0x3040..=0x30ff | 0xac00..=0xd7af)
}

pub fn word_count(s: &str) -> usize {
    let mut n = 0;
    let mut in_word = false;
    for c in s.chars() {
        let wordy = c.is_alphanumeric() && !is_cjk(c) || (in_word && matches!(c, '\'' | '’' | '-' | '.' | '_'));
        if wordy {
            if !in_word {
                n += 1;
            }
            in_word = true;
            continue;
        }
        in_word = false;
        if c.is_whitespace() || c.is_ascii() || c == '\u{fffc}' {
            continue;
        }
        n += 1;
    }
    n
}

pub fn char_count(s: &str) -> usize {
    s.chars().filter(|c| !c.is_whitespace() && *c != '\u{fffc}').count()
}

pub fn compute_doc_stats(store: &Store, id: &str, doc: &Document, created: u64, modified: u64) -> DocStats {
    let plain = doc.plain_text();
    let words = word_count(&doc.title.text) + word_count(&plain);
    let chars = char_count(&doc.title.text) + char_count(&plain);
    let mut blocks_count = 0;
    doc.for_each_block(|_| blocks_count += 1);
    let assets = doc.asset_sources();
    let size_bytes = store.size_of(id, doc);

    DocStats {
        id: id.to_string(),
        title: display_title(&doc.title.text),
        icon: doc.icon.clone(),
        words,
        chars,
        blocks_count,
        assets_count: assets.len(),
        assets,
        size_bytes,
        created,
        modified,
    }
}

pub fn resolve_doc_id(store: &mut Store, query: &str) -> Result<String> {
    let entries = store.list();
    if let Some(e) = entries.iter().find(|e| e.id == query) {
        return Ok(e.id.clone());
    }
    if query.len() >= 6 {
        let prefix_matches: Vec<_> = entries.iter().filter(|e| e.id.starts_with(query)).collect();
        if prefix_matches.len() == 1 {
            return Ok(prefix_matches[0].id.clone());
        }
    }
    let title_matches: Vec<_> = entries.iter().filter(|e| e.title == query || display_title(&e.title) == query).collect();
    if title_matches.len() == 1 {
        return Ok(title_matches[0].id.clone());
    } else if title_matches.len() > 1 {
        anyhow::bail!("找到多篇标题为 '{query}' 的文档，请使用文档 ID 指定");
    }

    let title_ci_matches: Vec<_> = entries.iter().filter(|e| e.title.eq_ignore_ascii_case(query) || display_title(&e.title).eq_ignore_ascii_case(query)).collect();
    if title_ci_matches.len() == 1 {
        return Ok(title_ci_matches[0].id.clone());
    }

    let title_contains: Vec<_> = entries.iter().filter(|e| e.title.contains(query) || display_title(&e.title).contains(query)).collect();
    if title_contains.len() == 1 {
        return Ok(title_contains[0].id.clone());
    } else if title_contains.len() > 1 {
        let list: Vec<String> = title_contains.iter().map(|e| format!("- {} ({})", display_title(&e.title), e.id)).collect();
        anyhow::bail!("匹配到多篇文档：\n{}\n请使用文档 ID 指定", list.join("\n"));
    }

    anyhow::bail!("未找到匹配的文档：'{query}'")
}

pub fn resolve_block_id(blocks: &[Block], locator: &str) -> Option<BlockId> {
    let loc = locator.trim();
    if loc.eq_ignore_ascii_case("first") {
        return blocks.first().map(|b| b.id);
    }
    if loc.eq_ignore_ascii_case("last") {
        return blocks.last().map(|b| b.id);
    }
    if let Some(id_str) = loc.strip_prefix('#')
        && let Ok(id_num) = id_str.parse::<u64>()
    {
        return find_by_id(blocks, id_num);
    }

    let parts: Result<Vec<usize>, _> = loc.split('.').map(str::parse).collect();
    if let Ok(indices) = parts {
        let mut slice = blocks;
        let mut target = None;
        for (i, &idx) in indices.iter().enumerate() {
            if let Some(b) = slice.get(idx) {
                if i == indices.len() - 1 {
                    target = Some(b.id);
                    break;
                }
                slice = &b.children;
            } else {
                target = None;
                break;
            }
        }
        if let Some(id) = target {
            return Some(id);
        }
    }

    if let Ok(id_num) = loc.parse::<u64>() {
        return find_by_id(blocks, id_num);
    }
    None
}

fn find_by_id(bs: &[Block], id: u64) -> Option<BlockId> {
    for b in bs {
        if b.id.0 == id {
            return Some(b.id);
        }
        if let Some(found) = find_by_id(&b.children, id) {
            return Some(found);
        }
    }
    None
}

pub fn insert_blocks_with_pos(
    core: &mut Core,
    blocks: Vec<Block>,
    pos: InsertPosition,
) -> Result<Vec<BlockId>> {
    let ids: Vec<BlockId> = blocks.iter().map(|b| b.id).collect();
    match pos {
        InsertPosition::After(loc) => {
            let target = resolve_block_id(&core.doc.blocks, &loc)
                .with_context(|| format!("找不到目标块：{loc}"))?;
            core.insert_blocks_at(blocks, target, DropPlace::After);
        }
        InsertPosition::Before(loc) => {
            let target = resolve_block_id(&core.doc.blocks, &loc)
                .with_context(|| format!("找不到目标块：{loc}"))?;
            core.insert_blocks_at(blocks, target, DropPlace::Before);
        }
        InsertPosition::Parent(loc) => {
            let target = resolve_block_id(&core.doc.blocks, &loc)
                .with_context(|| format!("找不到目标容器块：{loc}"))?;
            core.insert_blocks_at(blocks, target, DropPlace::FirstChild);
        }
        InsertPosition::First => {
            if let Some(first) = core.doc.blocks.first().map(|b| b.id) {
                core.insert_blocks_at(blocks, first, DropPlace::Before);
            } else {
                core.doc.blocks = blocks;
            }
        }
        InsertPosition::End => {
            if let Some(last) = core.doc.blocks.last().map(|b| b.id) {
                core.insert_blocks_at(blocks, last, DropPlace::After);
            } else {
                core.doc.blocks = blocks;
            }
        }
    }
    Ok(ids)
}

fn parse_block_kind(s: &str) -> Result<BlockKind> {
    match s.trim().to_ascii_lowercase().as_str() {
        "text" | "p" | "paragraph" => Ok(BlockKind::Text),
        "h1" | "heading1" | "heading 1" => Ok(BlockKind::Heading { level: 1, seq_auto: false }),
        "h2" | "heading2" | "heading 2" => Ok(BlockKind::Heading { level: 2, seq_auto: false }),
        "h3" | "heading3" | "heading 3" => Ok(BlockKind::Heading { level: 3, seq_auto: false }),
        "h4" | "heading4" | "heading 4" => Ok(BlockKind::Heading { level: 4, seq_auto: false }),
        "h5" | "heading5" | "heading 5" => Ok(BlockKind::Heading { level: 5, seq_auto: false }),
        "h6" | "heading6" | "heading 6" => Ok(BlockKind::Heading { level: 6, seq_auto: false }),
        "bullet" | "ul" => Ok(BlockKind::Bullet),
        "ordered" | "ol" => Ok(BlockKind::Ordered { seq: None }),
        "todo" | "task" => Ok(BlockKind::Todo { done: false }),
        "code" => Ok(BlockKind::Code { lang: String::new(), caption: None, wrap: true }),
        "quote" => Ok(BlockKind::Quote),
        "callout" => Ok(BlockKind::Callout { emoji: "💡".into(), bg: None, border: None, text_color: None }),
        "divider" | "hr" => Ok(BlockKind::Divider),
        other => anyhow::bail!("未知块类型 '{other}'，支持: text, h1..h6, bullet, ordered, todo, code, quote, callout, divider"),
    }
}

pub fn update_block_in_core(
    core: &mut Core,
    locator: &str,
    update: BlockUpdate,
) -> Result<BlockId> {
    let target = resolve_block_id(&core.doc.blocks, locator)
        .with_context(|| format!("找不到目标块：{locator}"))?;

    if let Some(kind_str) = update.turn_into {
        let kind = parse_block_kind(&kind_str)?;
        core.turn_into(&[target], kind);
    }

    let b = core
        .block_mut(target)
        .with_context(|| format!("无法获取块：{locator}"))?;
    if let Some(align) = update.align {
        b.align = align;
    }
    if let Some(text) = update.text {
        b.text = RichText::plain(&text);
    }
    match &mut b.kind {
        BlockKind::Heading { level, .. } => {
            if let Some(l) = update.level {
                *level = l.clamp(1, 6);
            }
        }
        BlockKind::Todo { done, .. } => {
            if let Some(d) = update.done {
                *done = d;
            }
            if update.toggle_done {
                *done = !*done;
            }
        }
        BlockKind::Code { lang, caption, .. } => {
            if let Some(l) = update.lang {
                *lang = l;
            }
            if let Some(c) = update.caption {
                *caption = Some(c);
            }
        }
        BlockKind::Callout { emoji, .. } => {
            if let Some(e) = update.emoji {
                *emoji = e;
            }
        }
        BlockKind::Image { caption, width, height, .. } => {
            if let Some(c) = update.caption {
                *caption = Some(c);
            }
            if let Some(w) = update.width {
                *width = Some(w);
            }
            if let Some(h) = update.height {
                *height = Some(h);
            }
        }
        _ => {}
    }
    Ok(target)
}

pub fn delete_block_in_core(core: &mut Core, locator: &str) -> Result<BlockId> {
    let target = resolve_block_id(&core.doc.blocks, locator)
        .with_context(|| format!("找不到目标块：{locator}"))?;
    core.delete_block(target);
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fdl_core::model::{BlockKind, Document, RichText};

    #[test]
    fn test_resolve_block_id_locators() {
        let b0 = Block::new(BlockKind::Heading { level: 1, seq_auto: false }, RichText::plain("H1"));
        let b1_child = Block::new(BlockKind::Text, RichText::plain("Child"));
        let b1 = Block::new(BlockKind::Quote, RichText::plain("Q")).with_children(vec![b1_child]);
        let b2 = Block::new(BlockKind::Text, RichText::plain("T"));
        let blocks = vec![b0.clone(), b1.clone(), b2.clone()];

        assert_eq!(resolve_block_id(&blocks, "first"), Some(b0.id));
        assert_eq!(resolve_block_id(&blocks, "last"), Some(b2.id));
        assert_eq!(resolve_block_id(&blocks, "0"), Some(b0.id));
        assert_eq!(resolve_block_id(&blocks, "1"), Some(b1.id));
        assert_eq!(resolve_block_id(&blocks, "1.0"), Some(b1.children[0].id));
        assert_eq!(resolve_block_id(&blocks, &format!("#{}", b2.id.0)), Some(b2.id));
        assert_eq!(resolve_block_id(&blocks, &b2.id.0.to_string()), Some(b2.id));
        assert_eq!(resolve_block_id(&blocks, "99"), None);
    }

    #[test]
    fn test_insert_and_update_block() {
        let doc = Document {
            blocks: vec![Block::new(BlockKind::Text, RichText::plain("Hello"))],
            ..Default::default()
        };
        let mut core = Core::new(doc);

        let new_b = Block::new(BlockKind::Text, RichText::plain("World"));
        let inserted = insert_blocks_with_pos(&mut core, vec![new_b], InsertPosition::End).unwrap();
        assert_eq!(core.doc.blocks.len(), 2);
        assert_eq!(inserted.len(), 1);

        update_block_in_core(&mut core, "1", BlockUpdate {
            align: Some(Align::Center),
            text: Some("Quiet".into()),
            ..Default::default()
        }).unwrap();
        assert_eq!(core.doc.blocks[1].align, Align::Center);
        assert_eq!(core.doc.blocks[1].text.plain_text(), "Quiet");

        // Test turn_into and todo toggle
        update_block_in_core(&mut core, "1", BlockUpdate {
            turn_into: Some("todo".into()),
            done: Some(false),
            ..Default::default()
        }).unwrap();
        assert!(matches!(core.doc.blocks[1].kind, BlockKind::Todo { done: false, .. }));

        update_block_in_core(&mut core, "1", BlockUpdate {
            toggle_done: true,
            ..Default::default()
        }).unwrap();
        assert!(matches!(core.doc.blocks[1].kind, BlockKind::Todo { done: true, .. }));

        delete_block_in_core(&mut core, "0").unwrap();
        assert_eq!(core.doc.blocks.len(), 1);
        assert_eq!(core.doc.blocks[0].text.plain_text(), "Quiet");
    }

    #[test]
    fn test_word_and_char_counts() {
        let text = "Hello world! 这是中文字符。";
        // "Hello" (1) + "world" (1) + 6 CJK chars + 1 full-width punctuation = 9 words
        assert_eq!(word_count(text), 9);
        // Visible characters without spaces: 5 + 5 + 1 + 6 + 1 = 18
        assert_eq!(char_count(text), 18);
    }
}
