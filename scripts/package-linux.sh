#!/bin/sh
# Build and package Quiet.md and quiet CLI for Linux distribution.
#
#   scripts/package-linux.sh [version]
#
# Produces:
#   dist/quiet-md-$version-linux-$arch.tar.gz
#   dist/quiet-md-$version-linux-$arch.tar.gz.sha256
set -e
cd "$(dirname "$0")/.."

version=${1:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}
arch=$(uname -m)
[ "$arch" = "x86_64" ] && arch="x64"

echo "== 编译 Quiet.md ($version, $arch)"
cargo build --release --locked --workspace

out="dist"
bundle_name="quiet-md-$version-linux-$arch"
bundle_dir="$out/$bundle_name"
tarball="$out/$bundle_name.tar.gz"

echo "== 组装 Linux 发布包: $bundle_dir"
rm -rf "$bundle_dir" "$tarball"
mkdir -p "$bundle_dir/bin" "$bundle_dir/share/applications" "$bundle_dir/share/icons/hicolor/512x512/apps"

# 二进制文件
cp target/release/quiet_md "$bundle_dir/bin/quiet_md"
cp target/release/quiet "$bundle_dir/bin/quiet"
chmod +x "$bundle_dir/bin/quiet_md" "$bundle_dir/bin/quiet"

# 桌面图标与 .desktop
cp assets/branding/app-icon.png "$bundle_dir/share/icons/hicolor/512x512/apps/quiet-md.png"

cat > "$bundle_dir/share/applications/quiet-md.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Quiet.md
GenericName=Markdown Notes
Comment=A fast, quiet, local rich Markdown notes app
Exec=quiet_md %U
Icon=quiet-md
Terminal=false
Categories=Office;TextEditor;Utility;
StartupWMClass=quiet-md
MimeType=x-scheme-handler/quiet-md;
DESKTOP

# 安装与卸载脚本
cat > "$bundle_dir/install.sh" <<'INSTALL'
#!/bin/sh
set -e
PREFIX="${PREFIX:-$HOME/.local}"
echo "安装 Quiet.md 到 $PREFIX ..."

mkdir -p "$PREFIX/bin" "$PREFIX/share/applications" "$PREFIX/share/icons/hicolor/512x512/apps"

cp bin/quiet_md "$PREFIX/bin/quiet_md"
ln -sf "$PREFIX/bin/quiet_md" "$PREFIX/bin/quiet-md"
cp bin/quiet "$PREFIX/bin/quiet"
cp share/icons/hicolor/512x512/apps/quiet-md.png "$PREFIX/share/icons/hicolor/512x512/apps/quiet-md.png"

# 替换 Desktop Entry 中的 Exec 路径
sed "s|Exec=quiet_md|Exec=$PREFIX/bin/quiet_md|g" share/applications/quiet-md.desktop > "$PREFIX/share/applications/quiet-md.desktop"
chmod +x "$PREFIX/bin/quiet_md" "$PREFIX/bin/quiet" "$PREFIX/share/applications/quiet-md.desktop"

command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$PREFIX/share/applications" || true
echo "安装完成！可在应用菜单中启动 Quiet.md，或终端运行 quiet / quiet-md。"
INSTALL
chmod +x "$bundle_dir/install.sh"

cat > "$bundle_dir/uninstall.sh" <<'UNINSTALL'
#!/bin/sh
set -e
PREFIX="${PREFIX:-$HOME/.local}"
echo "卸载 Quiet.md ($PREFIX) ..."

rm -f "$PREFIX/bin/quiet_md" "$PREFIX/bin/quiet-md" "$PREFIX/bin/quiet"
rm -f "$PREFIX/share/applications/quiet-md.desktop"
rm -f "$PREFIX/share/icons/hicolor/512x512/apps/quiet-md.png"

command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$PREFIX/share/applications" || true
echo "卸载完成（用户笔记数据不受影响）。"
UNINSTALL
chmod +x "$bundle_dir/uninstall.sh"

cp README.md "$bundle_dir/README.md"
cp LICENSE "$bundle_dir/LICENSE"

echo "== 打包 $tarball"
tar -czf "$tarball" -C "$out" "$bundle_name"
rm -rf "$bundle_dir"

sha256sum "$tarball" | awk '{print $1}' > "$tarball.sha256"
echo "== 生成完毕: $tarball (SHA256: $(cat "$tarball.sha256"))"
