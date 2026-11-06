use fdl_core::model::{Align, Block, BlockKind};
use serde::Serialize;

#[derive(Serialize)]
pub struct BlockNode {
    pub path: String,
    pub id: u64,
    pub kind: String,
    pub align: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<BlockNode>,
}

pub fn build_block_tree(blocks: &[Block], prefix: &str) -> Vec<BlockNode> {
    blocks
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let path = if prefix.is_empty() { format!("{i}") } else { format!("{prefix}.{i}") };
            let (kind, text, caption, src, width, height) = match &b.kind {
                BlockKind::Text => ("Paragraph".into(), Some(b.text.plain_text()), None, None, None, None),
                BlockKind::Heading { level, .. } => (format!("Heading {level}"), Some(b.text.plain_text()), None, None, None, None),
                BlockKind::Bullet => ("Bullet".into(), Some(b.text.plain_text()), None, None, None, None),
                BlockKind::Ordered { seq } => {
                    let k = seq.map(|s| format!("Ordered ({s})")).unwrap_or_else(|| "Ordered".into());
                    (k, Some(b.text.plain_text()), None, None, None, None)
                }
                BlockKind::Todo { done, .. } => {
                    let k = if *done { "Todo [x]" } else { "Todo [ ]" };
                    (k.into(), Some(b.text.plain_text()), None, None, None, None)
                }
                BlockKind::Code { lang, caption, .. } => {
                    let k = if lang.is_empty() { "Code".into() } else { format!("Code ({lang})") };
                    (k, Some(b.text.plain_text()), caption.clone(), None, None, None)
                }
                BlockKind::Quote => ("Quote".into(), Some(b.text.plain_text()), None, None, None, None),
                BlockKind::Callout { emoji, .. } => (format!("Callout ({emoji})"), Some(b.text.plain_text()), None, None, None, None),
                BlockKind::Divider => ("Divider".into(), None, None, None, None, None),
                BlockKind::Image { src, caption, width, height, .. } => {
                    ("Image".into(), None, caption.clone(), Some(src.clone()), *width, *height)
                }
                BlockKind::Table(_) => ("Table".into(), None, None, None, None, None),
                BlockKind::Grid => ("Grid".into(), None, None, None, None, None),
                BlockKind::Column { ratio } => (format!("Column ({ratio})"), None, None, None, None, None),
                BlockKind::SubPageList => ("SubPageList".into(), None, None, None, None, None),
                BlockKind::Countdown(_) => ("Countdown".into(), None, None, None, None, None),
                BlockKind::File(f) => (format!("File ({})", f.name), None, None, Some(f.src.clone()), None, None),
                BlockKind::Raw { tag, .. } => (format!("Raw ({tag})"), None, None, None, None, None),
            };
            let align = match b.align {
                Align::Left => "left",
                Align::Center => "center",
                Align::Right => "right",
            }
            .into();
            let children = build_block_tree(&b.children, &path);
            BlockNode { path, id: b.id.0, kind, align, text, caption, src, width, height, children }
        })
        .collect()
}

pub fn print_block_tree(nodes: &[BlockNode], indent_level: usize) {
    let indent = "  ".repeat(indent_level);
    for n in nodes {
        let mut meta = vec![];
        if n.align != "left" {
            meta.push(format!("align: {}", n.align));
        }
        if let Some(w) = n.width {
            meta.push(format!("width: {w}"));
        }
        if let Some(h) = n.height {
            meta.push(format!("height: {h}"));
        }
        let meta_str = if meta.is_empty() { String::new() } else { format!(" ({})", meta.join(", ")) };

        let mut desc = String::new();
        if let Some(src) = &n.src {
            desc.push_str(src);
        }
        if let Some(c) = &n.caption {
            if !desc.is_empty() {
                desc.push(' ');
            }
            desc.push_str(&format!("\"{}\"", c));
        }
        if let Some(t) = &n.text {
            let snippet: String = t.lines().next().unwrap_or("").chars().take(60).collect();
            if !snippet.is_empty() {
                if !desc.is_empty() {
                    desc.push(' ');
                }
                desc.push_str(&snippet);
            }
        }
        let desc_str = if desc.is_empty() { String::new() } else { format!(": {desc}") };

        println!("{indent}[{}] #{} {}{}{}", n.path, n.id, n.kind, meta_str, desc_str);
        if !n.children.is_empty() {
            print_block_tree(&n.children, indent_level + 1);
        }
    }
}
