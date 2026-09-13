<div align="center">

# 󰎆 tune

<p align="center">
  <strong>现代 · 极简 · 高性能的网易云音乐终端 (TUI) 播放器</strong>
</p>

<p align="center">
  <a href="https://github.com/aimy1/tune/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square" alt="License MIT" /></a>
  <img src="https://img.shields.io/badge/Rust-2024%20Edition-orange.svg?style=flat-square" alt="Rust Edition" />
  <img src="https://img.shields.io/badge/Platform-Linux-lightgrey.svg?style=flat-square" alt="Platform Linux" />
  <img src="https://img.shields.io/badge/Ratatui-v0.30-red.svg?style=flat-square" alt="Ratatui Version" />
  <img src="https://img.shields.io/badge/Theme-Catppuccin-blueviolet.svg?style=flat-square" alt="Catppuccin" />
</p>

<p align="center">
  <a href="#-特性亮点">特性亮点</a> •
  <a href="#-快速开始">快速开始</a> •
  <a href="#-快捷键指南">快捷键</a> •
  <a href="#-配置与主题">配置</a> •
  <a href="#-开源协议">开源协议</a>
</p>

</div>

---

`tune` 是一款专为终端极客与音乐爱好者打造的网易云音乐终端播放器。基于 **Rust** 与 **Ratatui** 异步渲染引擎构建，追求极致轻量、亚毫秒级输入响应，融入 **Catppuccin** 全色系主题、**Kitty / Sixel** 高清封面渲染以及沉浸式全屏微动效，让终端听歌体验兼具极客速度与现代美学。

---

## ✨ 特性亮点

| 模块 | 特性说明 |
| :--- | :--- |
| 🎨 **设计美学** | 原生搭载 **Catppuccin** 官方四色系（Mocha、Macchiato、Frappe、Latte）与 Hyprland 配色；支持背景透明化，与终端壁纸自然融为一体。 |
| 🖼️ **高清封面** | 基于 `ratatui-image` 支持 Kitty 与 Sixel 图形协议，终端内无损直出超清专辑封面；不支持图形协议的终端智能优雅降级为 ASCII 字符画。 |
| 🎛️ **全屏沉浸** | 双栏沉浸式播放视窗；**单行极简悬浮音量胶囊**（不遮挡进度条，支持滚轮/点击无级调节）；平滑滚动的实时卡拉 OK 歌词。 |
| ⚡ **极速性能** | 纯异步 Tokio 事件驱动，内存开销仅数十兆，启动瞬时秒开，无任何 Electron 或 Webview 臃肿负担。 |
| 🎧 **高质音源** | 支持网易云从标准 MP3 至无损 FLAC、Hi-Res、超清母带等全部音质；支持 AcoustID 音频指纹识别与离线歌词匹配。 |
| 🔌 **桌面集成** | Linux 桌面 **MPRIS** 协议深度集成，支持媒体快捷按键、D-Bus 系统总线控制与桌面切歌通知。 |

---

## 🚀 快速开始

> [!TIP]
> 推荐终端使用支持 [Nerd Fonts](https://www.nerdfonts.com/) 图标集的等宽字体（如 *JetBrainsMono Nerd Font*、*FiraCode Nerd Font*），以获得最佳图标渲染效果。

### 1. 系统依赖安装

编译本项目需要 `ALSA` 音频开发库和 `D-Bus` 消息总线库：

- **Arch Linux**:
  ```bash
  sudo pacman -S alsa-lib dbus pkgconf chromaprint
  ```

- **Fedora / RHEL**:
  ```bash
  sudo dnf install -y alsa-lib-devel dbus-devel pkgconf-pkg-config chromaprint-devel
  ```

- **Ubuntu / Debian**:
  ```bash
  sudo apt-get update && sudo apt-get install -y libasound2-dev libdbus-1-dev pkg-config libchromaprint-dev
  ```

---

### 2. 构建与运行

确保已安装 [Rust](https://www.rust-lang.org/) 环境（推荐使用 Rust 2024 Edition / 1.85+）：

```bash
# 1. 克隆代码仓库
git clone https://github.com/aimy1/tune.git
cd tune

# 2. 进行 Release 优化编译
cargo build --release

# 3. 安装到用户可执行路径（可选）
install -m 755 target/release/tune ~/.local/bin/tune
```

安装完成后直接运行即可：
```bash
tune
```

---

## ⌨️ 快捷键指南

### 全局与导航
| 按键 | 功能描述 |
| :---: | :--- |
| <kbd>s</kbd> / <kbd>/</kbd> | 唤出全局搜索弹窗 |
| <kbd>,</kbd> | 打开系统设置与快捷键偏好菜单 |
| <kbd>b</kbd> | 展开 / 折叠主页侧边栏 |
| <kbd>f</kbd> | 进入 / 退出沉浸式全屏播放模式 |
| <kbd>q</kbd> / <kbd>Ctrl+C</kbd> | 优雅退出播放器 |

### 播放与控制
| 按键 | 功能描述 |
| :---: | :--- |
| <kbd>Space</kbd> | 播放 / 暂停当前曲目 |
| <kbd>Enter</kbd> | 播放选中歌曲、进入歌单、确认选中项 |
| <kbd>Esc</kbd> | 返回上一级 / 关闭当前弹出浮层 |
| <kbd>←</kbd> / <kbd>→</kbd> | 后退 / 快进播放进度 |
| <kbd>↑</kbd> / <kbd>↓</kbd> | 在曲目列表、推荐卡片或菜单间移动焦点 |
| <kbd>Ctrl+↑</kbd> / <kbd>Ctrl+↓</kbd> | 侧边栏“创建歌单”与“收藏歌单”分区快捷跳转 |

> [!NOTE]
> 在全屏播放模式下，点击控制栏音量按钮即可唤出**单行极简音量胶囊条**，支持滚轮、方向键及鼠标点击无级调整音量。

---

## ⚙️ 配置与主题

首次运行后，`tune` 将在本地生成标准化 TOML 配置文件：

- **主配置文件**：`~/.config/tune/config/default.toml`
- **预设主题**：`~/.config/tune/themes/`
  - `catppuccin_mocha.toml`（默认深色调）
  - `catppuccin_macchiato.toml`
  - `catppuccin_frappe.toml`
  - `catppuccin_latte.toml`（清爽浅色调）
  - `hyprland.toml`
  - `system.toml`（系统终端原生配色）

在 `default.toml` 中配置 `theme = "mocha"` 即可快速切换主题风格。

---

## 👏 特别致谢

*   **架构与灵感**：感谢 [@professor-lee](https://github.com/professor-lee) 的开源贡献与底层架构灵感。
*   **渲染引擎**：感谢 [Ratatui](https://github.com/ratatui/ratatui) 团队卓越的终端 UI 开发生态。

---

## 📄 开源协议

本项目基于 [MIT License](LICENSE) 协议开源。欢迎提交 Issue 反馈建议或发起 Pull Request 共同改进！
