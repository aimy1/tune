# 󰎆 tune

<p align="center">
  <img src="https://raw.githubusercontent.com/aimy1/tune/main/logo.svg" alt="tune TUI Logo" width="120" />
</p>

<h3 align="center">tune — 现代、高性能的网易云音乐终端 (TUI) 播放器</h3>

<p align="center">
  <a href="https://github.com/aimy1/tune/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License MIT" />
  </a>
  <img src="https://img.shields.io/badge/Rust-2024%20Edition-orange.svg" alt="Rust Edition" />
  <img src="https://img.shields.io/badge/Platform-Linux-lightgrey.svg" alt="Platform Linux" />
  <img src="https://img.shields.io/badge/TUI-Ratatui%20v0.30-red.svg" alt="Ratatui Version" />
</p>

---

`tune` 是一款专为音乐爱好者和终端极客量身定制的网易云音乐终端播放器。它基于 **Rust** 与 **Ratatui** 渲染引擎构建，在保留终端极致轻量、高效体验的同时，融入了丰富细腻的视觉效果与交互体验。

---

## 📖 目录

- [🌟 核心特性](#-核心特性)
- [📸 界面预览](#-界面预览)
- [🚀 快速开始](#-快速开始)
  - [1. 安装系统依赖](#1-安装系统依赖)
  - [2. 编译与安装](#2-编译与安装)
- [⚙️ 配置说明](#-配置说明)
  - [默认配置文件](#默认配置文件)
  - [Catppuccin 主题家族](#catppuccin-主题家族)
- [⌨️ 快捷键指南](#-快捷键指南)
- [👏 特别致谢](#-特别致谢)
- [📄 开源协议](#-开源协议)

---

## 🌟 核心特性

*   **📱 现代分栏终端布局**
    *   **左侧导航栏**：展示自建与收藏歌单，完美支持用户状态同步。
    *   **中央主视图**：包含精美网格卡片样式的“发现”推荐、歌单列表、搜索页及歌手专辑库。
    *   **右侧/悬浮歌词**：支持平滑滚动的沉浸式歌词视窗。
*   **🎨 Catppuccin 优雅主题**
    *   原生预设四款经典的 Catppuccin 色系（Latte、Frappe、Macchiato、Mocha）及 System 主题。
    *   支持 `transparent_background`，完美融于您原本的终端透明效果。
*   **🖼️ 高阶专辑封面渲染**
    *   通过 `ratatui-image` 在支持 Kitty / Sixel 协议的终端上渲染超高清专辑封面。
    *   针对不支持图像的终端，能够无损优雅降级为 ASCII 字符画或文本布局。
*   **✨ 细节拉满的动效与视觉反馈**
    *   **弹性动效**：侧边栏采用 `EaseOutQuad` 非线性阻尼过渡动画，拉出/收回顺滑优雅。
    *   **多维边框**：聚焦的活动块拥有双线框（Double Borders）并带强调色激活指示，非活动块自动降级为圆角框，聚焦感知极强。
    *   **精美列表**：歌单及搜索列表支持奇偶交替斑马纹配色，当前选中行高亮渲染，拒绝视觉疲劳。
*   **📊 音频波形频谱分析**
    *   播放条集成实时响应的 Braille（盲文编码）动感音频频谱指示器。
*   **🔌 桌面环境 MPRIS 深度集成**
    *   支持 Linux 桌面媒体快捷按键，能够接收系统全局控制命令并发送通知。

---

## 📸 界面预览

> [!NOTE]  
> 推荐安装支持 [Nerd Fonts](https://www.nerdfonts.com/) 图标集的字体（如 *JetBrainsMono Nerd Font*），以获得最佳的界面图标显示效果。

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  󰎆  NetEase Cloud Music  │ 传递音乐的力量                         󰀄 Guest Mode (游客)  │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ 📂 Created Playlists      │ 🔍 [ 🔍 搜索你喜欢的音乐...                             ] │
│   ▎ 󰓇  我的红心歌单 [30首]│ ─────────────────────────────────────────────────────── │
│     本地音乐    [12首]  │  ┌──────────────┐                                        │
│     秋日私语    [18首]  │  │  [Album Art] │  󰎆 晴天 - 周杰伦                         │
│ 📂 Collected Playlists    │  │ (专辑封面图片)│  💿 专辑: 叶惠美                          │
│     周杰伦精选集 [50首]  │  │              │  📅 发行: 2003                            │
│     摇滚经典   [100首]  │  └──────────────┘                                        │
│                           │  [ 󰐊 立即播放 ]   [ 󰓇 收藏歌单 ]                         │
│                           │                                                        │
│                           │ 🎵 播放列表 (奇偶交替斑马纹配色)                        │
│                           │ 1. 晴天 ───────────────────────── 周杰伦 ───────── 04:29 │
│                           │ 2. 轨迹 ───────────────────────── 周杰伦 ───────── 05:22 │
├───────────────────────────┴────────────────────────────────────────────────────────┤
│ 󰎆 现在播放: 晴天 - 周杰伦                [  ]                  ▅▆▇▆▅▃ 01:23 / 04:29 │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━●─────────────────────────────────── │
│ [s] 搜索   [,] 设置   [b] 侧边栏   [f] 全屏   [q] 退出                                   │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 🚀 快速开始

### 1. 安装系统依赖

编译本项目依赖于系统的 `ALSA` 音频开发库和 `D-Bus` 消息总线开发库，请根据您的发行版选择并执行：

#### **Fedora (RHEL / CentOS)**
```bash
sudo dnf install -y dbus-devel pkgconf-pkg-config alsa-lib-devel chromaprint-devel
```

#### **Ubuntu / Debian (Pop!_OS / Linux Mint)**
```bash
sudo apt-get update
sudo apt-get install -y libdbus-1-dev pkg-config libasound2-dev libchromaprint-dev
```

### 2. 编译与安装

克隆本项目，清理缓存后进行 Release 优化编译：

```bash
# 克隆仓库并进入目录
git clone https://github.com/aimy1/tune.git
cd tune

# 清除旧的编译残留并执行完整构建
cargo clean
cargo build --release
```

**运行播放器：**
```bash
./target/release/tune
```

**系统级全局安装 (可选)：**
```bash
# 复制二进制到系统可执行路径
sudo cp target/release/tune /usr/local/bin/
```

---

## ⚙️ 配置说明

### 默认配置文件

首次启动程序后，`tune` 将在您的个人配置目录下自动写入一套默认配置，可通过修改这些文件自定义您的播放器体验：

*   **默认配置路径**：`~/.config/tune/config/default.toml`

### Catppuccin 主题家族

主题文件将保存在 `~/.config/tune/themes/` 路径下，包含了以下配色配置：
- `catppuccin_latte.toml` (温暖明亮的浅色调)
- `catppuccin_frappe.toml`
- `catppuccin_macchiato.toml`
- `catppuccin_mocha.toml` (默认极客深色调)
- `system.toml` (终端自带配色回退)

在 `default.toml` 中，通过调整 `theme = "mocha"` 选择您心仪的主题配色。

---

## ⌨️ 快捷键指南

| 按键 (Key) | 功能描述 (Description) |
| :--- | :--- |
| **`s`** / **`/`** | 唤出顶部搜索输入框 (Global Search Box) |
| **`,`** | 打开系统设置与快捷键更改菜单 (Settings) |
| **`b`** | 展开 / 折叠主页侧边栏 (Toggle Sidebar) |
| **`f`** | 开启 / 关闭全屏沉浸式播放状态 (Toggle Fullscreen) |
| **`Space` (空格)** | 播放 / 暂停当前音轨 (Play / Pause) |
| **`Enter` (回车)** | 播放选中歌曲、进入歌单、选中选项 (Confirm / Play) |
| **`Esc`** | 返回上一级、隐藏当前弹窗 (Go Back / Hide Popup) |
| **`Left`** / **`Right`** | 后退 / 前进播放进度 (Seek Backward / Forward) |
| **`Up`** / **`Down`** | 在歌曲列表、推荐卡片或设置中移动光标 (Navigate Up / Down) |
| **`Ctrl + Up`** / **`Down`** | 在侧边栏的“创建歌单”和“收藏歌单”分区之间快捷跳转 |
| **`q`** / **`Ctrl + C`** | 优雅退出播放器 (Quit App) |

---

## 👏 特别致谢

本项目的灵感来源、底层架构与初始版本核心代码源自原作者的优秀开源库，在此表达诚挚的谢意与敬意：

*   **原作者仓库**：[@professor-lee](https://github.com/professor-lee)

---

## 📄 开源协议

本项目基于 [MIT License](LICENSE) 协议开源，欢迎广大开发者提交 Issue 反馈或发起 Pull Request 参与共建！
