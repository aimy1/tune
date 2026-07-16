# tune

一个基于 Rust 和 Ratatui 构建的现代、高性能网易云音乐终端 (TUI) 播放器。

![tune TUI](https://raw.githubusercontent.com/aimy1/tune/main/logo.svg) *(或替换为您自己的运行截图)*

---

## ✨ 特性

- **现代终端界面 (Modern TUI)**：采用多栏网格布局，内置主页发现、歌单列表、搜索、作者专辑及沉浸式歌词页面。
- **丰富的个性化主题**：原生集成经典的 Catppuccin 主题家族（Latte 浅色、Frappe、Macchiato、Mocha）及系统默认高对比度配色。支持自定义 TOML 主题配置。
- **高阶专辑封面显示**：利用 `ratatui-image` 在支持 Kitty / Sixel 协议的终端上直接渲染超高清专辑封面图，对于不支持图形的终端则能优雅地降级渲染为 ASCII 字符画或文本。
- **平滑视觉交互**：
  - 侧边栏（Home Sidebar）折叠/展开引入 `EaseOutQuad` 自然弹性缓动动画。
  - 歌单及搜索列表奇偶行交替渲染（Zebra Striping），且当前选中行使用醒目的 `color_buff` 底色，层级分明。
  - 支持鼠标盲区测试（Hit-testing），可通过鼠标进行切歌、跳转进度和选择列表。
- **实时的 Braille 盲文音乐频谱**：底部播放条集成实时的音频波形频谱分析。
- **多平台 MPRIS 控制器集成**：深度融入 Linux 桌面生态，支持系统级媒体按键及通知控制。

---

## 🛠️ 安装与编译

### 1. 安装系统依赖

编译本项目所需的系统级库，请在对应的终端运行：

#### **Fedora (RHEL / CentOS)**
```bash
sudo dnf install -y dbus-devel pkgconf-pkg-config alsa-lib-devel chromaprint-devel
```

#### **Ubuntu / Debian (Mint / Pop!_OS)**
```bash
sudo apt-get update
sudo apt-get install -y libdbus-1-dev pkg-config libasound2-dev libchromaprint-dev
```

### 2. 编译并运行

克隆本项目后，使用 Cargo 进行清理并以 Release 模式进行优化编译：

```bash
# 清理旧构建缓存
cargo clean

# 构建优化版二进制程序
cargo build --release

# 运行播放器
./target/release/tune
```

如果您希望可以在系统的任何路径下通过直接输入 `tune` 启动播放器，可将二进制文件复制到系统 PATH 中：
```bash
sudo cp target/release/tune /usr/local/bin/
```

---

## ⚙️ 配置文件与主题

首次运行程序后，播放器会自动在您的系统配置路径中生成默认的配置文件及预设主题文件：

*   **默认配置路径**：`~/.config/tune/config/default.toml`
*   **主题文件目录**：`~/.config/tune/themes/`
    *   `catppuccin_latte.toml` (浅色主题)
    *   `catppuccin_frappe.toml`
    *   `catppuccin_macchiato.toml`
    *   `catppuccin_mocha.toml`
    *   `system.toml` (系统默认高反差)

可以通过修改 `default.toml` 中的 `theme = "mocha"` 配置项来快速切换您心仪的主题。

---

## ⌨️ 常用快捷键

### 全局操作
- `s` 或 `/`：唤出顶部搜索框
- `,`：打开系统设置菜单
- `b`：折叠/展开主页侧边栏
- `f`：开启/关闭全屏模式
- `q` 或 `Ctrl + C`：退出播放器

### 播放控制
- `Space` (空格)：播放 / 暂停
- `Enter`：播放选中的歌曲或进入歌单
- `Left` / `Right` (左右方向键)：快退 / 快进播放进度
- `Up` / `Down` (上下方向键)：在列表中上下移动光标
- `Ctrl + Up` / `Ctrl + Down`：在侧边栏中切换不同的歌单分区

---

## 📄 开源协议

本项目采用 [MIT License](LICENSE) 开源协议。欢迎提交 PR 和 Issue！
