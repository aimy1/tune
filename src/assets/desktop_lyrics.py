#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Tune Desktop Lyrics (桌面歌词)
A sleek, transparent, always-on-top, click-through desktop lyrics overlay.
Supports Wayland (via GtkLayerShell on Hyprland/Sway) and X11.
"""

import os
import sys
import json
import time
import signal

try:
    import gi
    gi.require_version('Gtk', '3.0')
    from gi.repository import Gtk, Gdk, GLib
    import cairo
except ImportError as e:
    sys.stderr.write(f"[tune-desktop-lyrics] Required library missing: {e}\n")
    sys.exit(1)

# Check GtkLayerShell for Wayland
has_layer_shell = False
if os.environ.get('WAYLAND_DISPLAY'):
    try:
        gi.require_version('GtkLayerShell', '0.1')
        from gi.repository import GtkLayerShell
        has_layer_shell = True
    except Exception:
        has_layer_shell = False


def get_default_state_path():
    runtime_dir = os.environ.get('XDG_RUNTIME_DIR')
    if runtime_dir:
        p = os.path.join(runtime_dir, 'tune', 'desktop_lyric.json')
        if os.path.exists(p) or os.path.isdir(os.path.dirname(p)):
            return p
    cache_dir = os.path.expanduser('~/.cache/tune')
    return os.path.join(cache_dir, 'desktop_lyric.json')


class DesktopLyricsWindow(Gtk.Window):
    def __init__(self, state_path):
        super().__init__(title='Tune Desktop Lyrics')
        self.state_path = state_path
        self.last_mtime = 0
        self.current_accent = "#33ccff"
        self.current_subtext = "#949cbb"
        self.last_text1 = ""
        self.last_text2 = ""

        self.set_decorated(False)
        self.set_resizable(False)
        self.set_app_paintable(True)
        self.set_skip_taskbar_hint(True)
        self.set_skip_pager_hint(True)

        # Transparent RGBA visual
        screen = self.get_screen()
        visual = screen.get_rgba_visual()
        if visual and screen.is_composited():
            self.set_visual(visual)

        if has_layer_shell:
            GtkLayerShell.init_for_window(self)
            GtkLayerShell.set_layer(self, GtkLayerShell.Layer.OVERLAY)
            GtkLayerShell.set_anchor(self, GtkLayerShell.Edge.BOTTOM, True)
            GtkLayerShell.set_margin(self, GtkLayerShell.Edge.BOTTOM, 54)
            GtkLayerShell.set_keyboard_mode(self, GtkLayerShell.KeyboardMode.NONE)
            GtkLayerShell.set_exclusive_zone(self, 0)
        else:
            self.set_type_hint(Gdk.WindowTypeHint.DOCK)
            self.set_keep_above(True)
            self.stick()

        # UI Layout
        self.box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=3)
        self.box.set_name('lyrics-container')

        self.label_main = Gtk.Label(label='♪ Tune 桌面歌词 ♪')
        self.label_main.set_name('lyric-main')
        self.label_main.set_ellipsize(3) # PANGO_ELLIPSIZE_END
        self.label_main.set_max_width_chars(60)

        self.label_sub = Gtk.Label(label='等待播放...')
        self.label_sub.set_name('lyric-sub')
        self.label_sub.set_ellipsize(3)
        self.label_sub.set_max_width_chars(65)

        self.box.pack_start(self.label_main, True, True, 0)
        self.box.pack_start(self.label_sub, True, True, 0)
        self.add(self.box)

        # CSS setup
        self.css_provider = Gtk.CssProvider()
        Gtk.StyleContext.add_provider_for_screen(
            screen, self.css_provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )
        self.update_style(self.current_accent, self.current_subtext)

        # Handle realize to enable click-through
        self.connect('realize', self.on_realize)
        self.connect('size-allocate', self.on_size_allocate)

        self.show_all()

        # Poll state every 50ms
        GLib.timeout_add(50, self.check_state)

    def on_realize(self, widget):
        self.enable_click_through()

    def on_size_allocate(self, widget, allocation):
        self.enable_click_through()

    def enable_click_through(self):
        gdk_win = self.get_window()
        if gdk_win:
            try:
                # An empty cairo.Region allows all pointer events to pass through
                empty_region = cairo.Region()
                gdk_win.input_shape_combine_region(empty_region, 0, 0)
            except Exception:
                pass

    def update_style(self, accent, subtext):
        css = f"""
        #lyrics-container {{
            background: rgba(18, 18, 26, 0.78);
            border-radius: 18px;
            padding: 8px 30px;
            border: 1px solid rgba(51, 204, 255, 0.35);
            margin: 0px 16px;
        }}
        #lyric-main {{
            color: {accent};
            font-family: 'JetBrainsMono Nerd Font', 'Noto Sans CJK SC', 'PingFang SC', 'Microsoft YaHei', sans-serif;
            font-size: 20px;
            font-weight: 800;
            text-shadow: 0 2px 4px rgba(0, 0, 0, 0.75);
            letter-spacing: 0.5px;
        }}
        #lyric-sub {{
            color: {subtext};
            font-family: 'JetBrainsMono Nerd Font', 'Noto Sans CJK SC', 'PingFang SC', 'Microsoft YaHei', sans-serif;
            font-size: 14px;
            font-weight: 500;
            text-shadow: 0 1px 3px rgba(0, 0, 0, 0.75);
        }}
        """
        self.css_provider.load_from_data(css.encode('utf-8'))

    def check_state(self):
        if not os.path.exists(self.state_path):
            return True

        try:
            st = os.stat(self.state_path)
            if st.st_mtime == self.last_mtime:
                return True
            self.last_mtime = st.st_mtime

            with open(self.state_path, 'r', encoding='utf-8') as f:
                data = json.load(f)

            line1 = data.get('line1', '').strip()
            line2 = data.get('line2', '').strip()
            state = data.get('state', 'Playing')
            accent = data.get('accent_color', '#33ccff')
            subtext = data.get('subtext_color', '#949cbb')
            parent_pid = data.get('tune_pid')

            # Parent process liveness check
            if parent_pid and parent_pid > 0:
                try:
                    os.kill(parent_pid, 0)
                except OSError:
                    # Parent process has exited
                    Gtk.main_quit()
                    return False

            # Update colors if changed
            if accent != self.current_accent or subtext != self.current_subtext:
                self.current_accent = accent
                self.current_subtext = subtext
                self.update_style(accent, subtext)

            # Display text logic
            if state == 'Stopped' or (not line1 and not line2):
                title = data.get('title', '')
                artist = data.get('artist', '')
                if title:
                    line1 = f"♪ {title}"
                    line2 = artist if artist else "Tune"
                else:
                    line1 = "♪ Tune 桌面歌词 ♪"
                    line2 = ""

            if line1 != self.last_text1:
                self.label_main.set_text(line1)
                self.last_text1 = line1

            if line2:
                if not self.label_sub.get_visible():
                    self.label_sub.show()
                if line2 != self.last_text2:
                    self.label_sub.set_text(line2)
                    self.last_text2 = line2
            else:
                if self.label_sub.get_visible():
                    self.label_sub.hide()
                    self.last_text2 = ""

        except Exception:
            # Tolerant against partial writes
            pass

        return True


def main():
    state_path = None
    if len(sys.argv) > 1:
        state_path = sys.argv[1]
    if not state_path:
        state_path = get_default_state_path()

    signal.signal(signal.SIGINT, lambda s, f: Gtk.main_quit())
    signal.signal(signal.SIGTERM, lambda s, f: Gtk.main_quit())

    win = DesktopLyricsWindow(state_path)

    # Position fallback for non-layer-shell (X11)
    if not has_layer_shell:
        screen = win.get_screen()
        w = 600
        h = 80
        x = (screen.get_width() - w) // 2
        y = screen.get_height() - h - 50
        win.move(x, y)

    Gtk.main()


if __name__ == '__main__':
    main()
