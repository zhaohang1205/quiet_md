use anyhow::{Context, Result};
use fdl_core::model::{Align, Block, BlockKind, RichText};
use std::path::Path;

pub struct ImageOptions {
    pub caption: Option<String>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub align: Align,
    pub copy: bool,
}

pub fn create_image_block(
    image_path: &Path,
    assets_dir: &Path,
    opts: ImageOptions,
) -> Result<(Block, String)> {
    anyhow::ensure!(image_path.exists(), "图片文件不存在：{}", image_path.display());
    let bytes = std::fs::read(image_path)
        .with_context(|| format!("无法读取图片文件：{}", image_path.display()))?;

    let (nat_w, nat_h) = image::ImageReader::new(std::io::Cursor::new(&bytes))
        .with_guessed_format()
        .ok()
        .and_then(|r| r.into_dimensions().ok())
        .map(|(w, h)| (w as f32, h as f32))
        .unwrap_or((0., 0.));

    let ext = image_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png")
        .to_ascii_lowercase();

    let stored_src = if opts.copy {
        let file_name = format!("{}.{ext}", uuid::Uuid::new_v4());
        let dest = assets_dir.join(&file_name);
        std::fs::create_dir_all(assets_dir)?;
        std::fs::write(&dest, &bytes)?;
        format!("assets/{file_name}")
    } else {
        std::fs::canonicalize(image_path)
            .unwrap_or_else(|_| image_path.to_path_buf())
            .to_string_lossy()
            .to_string()
    };

    let width = opts.width.or(if nat_w > 0. { Some(nat_w) } else { None });
    let height = opts.height.or(if nat_h > 0. { Some(nat_h) } else { None });

    let mut block = Block::new(
        BlockKind::Image {
            src_kind: "src".into(),
            src: stored_src.clone(),
            width,
            height,
            caption: opts.caption,
            crop: None,
        },
        RichText::default(),
    );
    block.align = opts.align;

    Ok((block, stored_src))
}
