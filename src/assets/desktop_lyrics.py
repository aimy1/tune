#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Tune Desktop Lyrics (桌面歌词)
A sleek, transparent, always-on-top, draggable desktop lyrics overlay.
Supports Wayland (via GtkLayerShell on Hyprland/Sway) and X11.
"""

import os
import sys
import json
import socket
import signal

try:
    import gi
    gi.require_version('Gtk', '3.0')
    from gi.repository import Gtk, Gdk, GLib, Pango
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


def get_hyprland_cursor_pos():
    sig = os.environ.get('HYPRLAND_INSTANCE_SIGNATURE')
    if not sig:
        return None
    runtime_dir = os.environ.get('XDG_RUNTIME_DIR', f"/run/user/{os.getuid()}")
    sock_path = os.path.join(runtime_dir, 'hypr', sig, '.socket.sock')
    if not os.path.exists(sock_path):
        sock_path = f"/tmp/hypr/{sig}/.socket.sock"
    if not os.path.exists(sock_path):
        return None
    try:
        s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        s.settimeout(0.05)
        s.connect(sock_path)
        s.sendall(b'cursorpos')
        data = s.recv(128).decode('utf-8').strip()
        s.close()
        parts = data.split(',')
        if len(parts) == 2:
            return int(parts[0].strip()), int(parts[1].strip())
    except Exception:
        pass
    return None


def find_monitor_for_point(display, x, y):
    n = display.get_n_monitors()
    if n == 0:
        return None, None

    # 1. Exact match
    for i in range(n):
        m = display.get_monitor(i)
        geom = m.get_geometry()
        if geom.x <= x < geom.x + geom.width and geom.y <= y < geom.y + geom.height:
            return m, geom

    # 2. Closest monitor center
    best_monitor = None
    best_geom = None
    min_dist_sq = float('inf')
    for i in range(n):
        m = display.get_monitor(i)
        geom = m.get_geometry()
        cx = geom.x + geom.width / 2.0
        cy = geom.y + geom.height / 2.0
        dist_sq = (x - cx) ** 2 + (y - cy) ** 2
        if dist_sq < min_dist_sq:
            min_dist_sq = dist_sq
            best_monitor = m
            best_geom = geom

    return best_monitor, best_geom


def get_total_screen_bounds(display):
    n = display.get_n_monitors()
    if n == 0:
        return 0, 0, 1920, 1080
    min_x = float('inf')
    min_y = float('inf')
    max_x = float('-inf')
    max_y = float('-inf')
    for i in range(n):
        geom = display.get_monitor(i).get_geometry()
        min_x = min(min_x, geom.x)
        min_y = min(min_y, geom.y)
        max_x = max(max_x, geom.x + geom.width)
        max_y = max(max_y, geom.y + geom.height)
    return int(min_x), int(min_y), int(max_x), int(max_y)


class DesktopLyricsWindow(Gtk.Window):
    def __init__(self, state_path):
        super().__init__(title='Tune Desktop Lyrics')
        self.state_path = state_path
        self.pos_path = os.path.join(os.path.dirname(state_path), 'desktop_lyric_pos.json')

        self.last_mtime = 0
        self.current_accent = "#33ccff"
        self.current_subtext = "#949cbb"
        self.last_text1 = ""
        self.last_text2 = ""

        # Settings
        self.locked = True
        self.font_size = 20
        self.dual_line = True
        self.align = "center"
        self.bg_style = "translucent"
        self.custom_pos_x = None
        self.custom_pos_y = None

        # Drag state & monitor
        self.current_monitor = None
        self.dragging = False
        self.drag_start_mouse = (0, 0)
        self.drag_start_win = (0, 0)
        self.current_win_x = 0
        self.current_win_y = 0

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
            display = Gdk.Display.get_default()
            self.current_monitor = display.get_primary_monitor() or (
                display.get_monitor(0) if display.get_n_monitors() > 0 else None
            )
            if self.current_monitor:
                GtkLayerShell.set_monitor(self, self.current_monitor)
            GtkLayerShell.set_anchor(self, GtkLayerShell.Edge.BOTTOM, True)
            GtkLayerShell.set_margin(self, GtkLayerShell.Edge.BOTTOM, 54)
            GtkLayerShell.set_keyboard_mode(self, GtkLayerShell.KeyboardMode.NONE)
            GtkLayerShell.set_exclusive_zone(self, 0)
        else:
            self.set_type_hint(Gdk.WindowTypeHint.DOCK)
            self.set_keep_above(True)
            self.stick()

        self.add_events(
            Gdk.EventMask.BUTTON_PRESS_MASK
            | Gdk.EventMask.BUTTON_RELEASE_MASK
            | Gdk.EventMask.BUTTON1_MOTION_MASK
            | Gdk.EventMask.POINTER_MOTION_MASK
        )
        self.connect('button-press-event', self.on_button_press)
        self.connect('button-release-event', self.on_button_release)
        self.connect('motion-notify-event', self.on_motion_notify)

        # Main Box container
        self.box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        self.box.set_name('lyrics-container')

        # Header bar for unlocked/dragging mode
        self.header_bar = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
        self.header_bar.set_name('header-bar')
        self.header_bar.set_no_show_all(True)

        self.drag_hint = Gtk.Label(label='⠿ 拖动调整位置')
        self.drag_hint.set_name('drag-hint')
        self.drag_hint.set_xalign(0.0)

        self.btn_reset = Gtk.Button(label='⟲ 居中')
        self.btn_reset.set_name('btn-reset')
        self.btn_reset.connect('clicked', self.on_reset_clicked)

        self.btn_lock = Gtk.Button(label='🔒 锁定')
        self.btn_lock.set_name('btn-lock')
        self.btn_lock.connect('clicked', self.on_lock_clicked)

        self.header_bar.pack_start(self.drag_hint, True, True, 0)
        self.header_bar.pack_end(self.btn_lock, False, False, 0)
        self.header_bar.pack_end(self.btn_reset, False, False, 0)
        self.box.pack_start(self.header_bar, False, False, 0)

        # Lyrics labels
        self.label_main = Gtk.Label(label='♪ Tune 桌面歌词 ♪')
        self.label_main.set_name('lyric-main')
        self.label_main.set_ellipsize(Pango.EllipsizeMode.END)
        self.label_main.set_max_width_chars(65)
        self.label_main.set_xalign(0.5)

        self.label_sub = Gtk.Label(label='等待播放...')
        self.label_sub.set_name('lyric-sub')
        self.label_sub.set_ellipsize(Pango.EllipsizeMode.END)
        self.label_sub.set_max_width_chars(70)
        self.label_sub.set_xalign(0.5)

        self.box.pack_start(self.label_main, True, True, 0)
        self.box.pack_start(self.label_sub, True, True, 0)
        self.add(self.box)

        # CSS setup
        self.css_provider = Gtk.CssProvider()
        Gtk.StyleContext.add_provider_for_screen(
            screen, self.css_provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )
        self.update_style()

        # Handle realize & allocation to configure click-through
        self.connect('realize', self.on_realize)
        self.connect('size-allocate', self.on_size_allocate)

        self.show_all()
        self.header_bar.hide()

        # Poll state every 50ms
        GLib.timeout_add(50, self.check_state)

    def on_realize(self, widget):
        self.apply_click_through_state()

    def on_size_allocate(self, widget, allocation):
        self.apply_click_through_state()

    def apply_click_through_state(self):
        gdk_win = self.get_window()
        if not gdk_win:
            return
        try:
            if self.locked:
                empty_region = cairo.Region()
                gdk_win.input_shape_combine_region(empty_region, 0, 0)
            else:
                gdk_win.input_shape_combine_region(None, 0, 0)
                display = self.get_display()
                cursor = Gdk.Cursor.new_from_name(display, 'grab') or Gdk.Cursor.new_from_name(display, 'fleur')
                if cursor:
                    gdk_win.set_cursor(cursor)
        except Exception:
            pass

    def on_lock_clicked(self, btn):
        self.set_locked_state(True)
        self.save_client_updates()

    def on_reset_clicked(self, btn):
        self.reset_to_default_position()
        self.save_client_updates(reset_pos=True)

    def set_locked_state(self, locked):
        self.locked = locked
        if self.locked:
            self.header_bar.hide()
        else:
            self.header_bar.show()
        self.update_style()
        self.apply_click_through_state()

    def reset_to_default_position(self, target_monitor=None):
        self.custom_pos_x = None
        self.custom_pos_y = None
        display = Gdk.Display.get_default()
        monitor = target_monitor or self.current_monitor or display.get_primary_monitor() or (
            display.get_monitor(0) if display.get_n_monitors() > 0 else None
        )
        if has_layer_shell:
            if monitor:
                GtkLayerShell.set_monitor(self, monitor)
                self.current_monitor = monitor
            GtkLayerShell.set_anchor(self, GtkLayerShell.Edge.TOP, False)
            GtkLayerShell.set_anchor(self, GtkLayerShell.Edge.LEFT, False)
            GtkLayerShell.set_anchor(self, GtkLayerShell.Edge.RIGHT, False)
            GtkLayerShell.set_anchor(self, GtkLayerShell.Edge.BOTTOM, True)
            GtkLayerShell.set_margin(self, GtkLayerShell.Edge.BOTTOM, 54)
            GtkLayerShell.set_margin(self, GtkLayerShell.Edge.LEFT, 0)
            GtkLayerShell.set_margin(self, GtkLayerShell.Edge.TOP, 0)
        else:
            geom = monitor.get_geometry() if monitor else Gdk.Rectangle()
            alloc = self.get_allocation()
            w = alloc.width if alloc.width > 50 else 600
            h = alloc.height if alloc.height > 20 else 80
            mw = geom.width if geom.width > 100 else 1920
            mh = geom.height if geom.height > 100 else 1080
            x = geom.x + (mw - w) // 2
            y = geom.y + mh - h - 54
            self.move(x, y)

    def apply_custom_position(self, x, y, target_monitor=None):
        self.custom_pos_x = x
        self.custom_pos_y = y
        self.current_win_x = x
        self.current_win_y = y

        display = Gdk.Display.get_default()
        if target_monitor is not None:
            monitor = target_monitor
            geom = monitor.get_geometry()
        else:
            alloc = self.get_allocation()
            w = alloc.width if alloc.width > 50 else 600
            h = alloc.height if alloc.height > 20 else 80
            monitor, geom = find_monitor_for_point(display, x + w // 2, y + h // 2)

        if has_layer_shell:
            if monitor and monitor != self.current_monitor:
                GtkLayerShell.set_monitor(self, monitor)
                self.current_monitor = monitor

            rel_x = x - (geom.x if geom else 0)
            rel_y = y - (geom.y if geom else 0)

            GtkLayerShell.set_anchor(self, GtkLayerShell.Edge.BOTTOM, False)
            GtkLayerShell.set_anchor(self, GtkLayerShell.Edge.RIGHT, False)
            GtkLayerShell.set_anchor(self, GtkLayerShell.Edge.TOP, True)
            GtkLayerShell.set_anchor(self, GtkLayerShell.Edge.LEFT, True)
            GtkLayerShell.set_margin(self, GtkLayerShell.Edge.LEFT, int(rel_x))
            GtkLayerShell.set_margin(self, GtkLayerShell.Edge.TOP, int(rel_y))
        else:
            self.move(int(x), int(y))

    def on_button_press(self, widget, event):
        if self.locked:
            return False

        # Don't intercept drag if clicking on buttons
        for btn in (self.btn_lock, self.btn_reset):
            if btn.get_visible():
                alloc = btn.get_allocation()
                coords = btn.translate_coordinates(self, 0, 0)
                if coords:
                    bx, by = coords
                    if bx <= event.x <= bx + alloc.width and by <= event.y <= by + alloc.height:
                        return False

        if event.button == 1:
            self.dragging = True
            self.update_style()

            hypr_pos = get_hyprland_cursor_pos()
            if hypr_pos:
                self.drag_start_mouse = hypr_pos
            else:
                self.drag_start_mouse = (int(event.x_root), int(event.y_root))

            display = Gdk.Display.get_default()
            cur_monitor, cur_geom = find_monitor_for_point(
                display, self.drag_start_mouse[0], self.drag_start_mouse[1]
            )

            # Determine initial window position
            if self.custom_pos_x is not None and self.custom_pos_y is not None:
                self.drag_start_win = (self.custom_pos_x, self.custom_pos_y)
            else:
                alloc = self.get_allocation()
                w = alloc.width if alloc.width > 50 else 600
                h = alloc.height if alloc.height > 20 else 80
                mw = cur_geom.width if cur_geom and cur_geom.width > 100 else 1920
                mh = cur_geom.height if cur_geom and cur_geom.height > 100 else 1080
                gx = cur_geom.x if cur_geom else 0
                gy = cur_geom.y if cur_geom else 0
                est_x = gx + (mw - w) // 2
                est_y = gy + mh - h - 54
                self.drag_start_win = (est_x, est_y)

            if cur_monitor:
                self.current_monitor = cur_monitor

            gdk_win = self.get_window()
            if gdk_win:
                display = self.get_display()
                cursor = Gdk.Cursor.new_from_name(display, 'grabbing')
                if cursor:
                    gdk_win.set_cursor(cursor)
            return True
        return False

    def on_motion_notify(self, widget, event):
        if not self.dragging or self.locked:
            return False

        hypr_pos = get_hyprland_cursor_pos()
        if hypr_pos:
            cur_mouse = hypr_pos
        else:
            cur_mouse = (int(event.x_root), int(event.y_root))

        dx = cur_mouse[0] - self.drag_start_mouse[0]
        dy = cur_mouse[1] - self.drag_start_mouse[1]

        target_x = self.drag_start_win[0] + dx
        target_y = self.drag_start_win[1] + dy

        # Clamp within multi-monitor total screen bounds
        display = Gdk.Display.get_default()
        min_x, min_y, max_x, max_y = get_total_screen_bounds(display)

        target_x = max(min_x, min(max_x - 120, target_x))
        target_y = max(min_y, min(max_y - 40, target_y))

        # Find target monitor for the cursor position
        monitor, _ = find_monitor_for_point(display, cur_mouse[0], cur_mouse[1])
        self.apply_custom_position(target_x, target_y, target_monitor=monitor)
        return True

    def on_button_release(self, widget, event):
        if event.button == 1 and self.dragging:
            self.dragging = False
            self.update_style()
            gdk_win = self.get_window()
            if gdk_win:
                display = self.get_display()
                cursor = Gdk.Cursor.new_from_name(display, 'grab') or Gdk.Cursor.new_from_name(display, 'fleur')
                if cursor:
                    gdk_win.set_cursor(cursor)
            self.save_client_updates()
            return True
        return False

    def save_client_updates(self, reset_pos=False):
        try:
            data = {
                "locked": self.locked,
                "pos_x": self.custom_pos_x,
                "pos_y": self.custom_pos_y,
                "reset_pos": reset_pos,
            }
            tmp = self.pos_path + ".tmp"
            with open(tmp, 'w', encoding='utf-8') as f:
                json.dump(data, f)
            os.replace(tmp, self.pos_path)
        except Exception as e:
            sys.stderr.write(f"[tune-desktop-lyrics] Failed to write pos file: {e}\n")

    def update_style(self):
        if self.bg_style == "dark":
            bg_css = "background: rgba(10, 10, 15, 0.94); border: 1px solid rgba(80, 80, 110, 0.45); border-radius: 18px;"
        elif self.bg_style == "light":
            bg_css = "background: rgba(22, 22, 34, 0.42); border: 1px solid rgba(255, 255, 255, 0.18); border-radius: 18px;"
        elif self.bg_style == "transparent":
            bg_css = "background: transparent; border: none; box-shadow: none;"
        else: # translucent
            bg_css = f"background: rgba(18, 18, 26, 0.78); border: 1px solid rgba(51, 204, 255, 0.35); border-radius: 18px;"

        if self.dragging:
            drag_border_css = f"""
            #lyrics-container {{
                border: 2px solid {self.current_accent} !important;
                box-shadow: 0 0 20px {self.current_accent}, inset 0 0 10px rgba(255, 255, 255, 0.15), 0 8px 32px rgba(0, 0, 0, 0.85) !important;
                background: rgba(16, 16, 26, 0.95) !important;
            }}
            #drag-hint {{
                color: {self.current_accent} !important;
                font-weight: 700 !important;
            }}
            """
        elif not self.locked:
            drag_border_css = f"""
            #lyrics-container {{
                border: 1.5px dashed {self.current_accent} !important;
                box-shadow: 0 4px 16px rgba(0, 0, 0, 0.6);
            }}
            """
        else:
            drag_border_css = ""

        main_fs = self.font_size
        sub_fs = max(11, int(main_fs * 0.70))

        css = f"""
        #lyrics-container {{
            {bg_css}
            padding: 8px 28px;
            margin: 0px 16px;
        }}
        {drag_border_css}
        #header-bar {{
            padding-bottom: 4px;
            margin-bottom: 2px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.12);
        }}
        #drag-hint {{
            color: {self.current_subtext};
            font-size: 11px;
            font-weight: 600;
        }}
        #btn-lock, #btn-reset {{
            background: rgba(40, 40, 60, 0.7);
            color: #ffffff;
            border: 1px solid rgba(255, 255, 255, 0.2);
            border-radius: 6px;
            padding: 2px 8px;
            font-size: 11px;
            margin-left: 4px;
        }}
        #btn-lock:hover, #btn-reset:hover {{
            background: {self.current_accent};
            color: #11111b;
        }}
        #lyric-main {{
            color: {self.current_accent};
            font-family: 'JetBrainsMono Nerd Font', 'Noto Sans CJK SC', 'PingFang SC', 'Microsoft YaHei', sans-serif;
            font-size: {main_fs}px;
            font-weight: 800;
            text-shadow: 0 2px 4px rgba(0, 0, 0, 0.85);
            letter-spacing: 0.5px;
        }}
        #lyric-sub {{
            color: {self.current_subtext};
            font-family: 'JetBrainsMono Nerd Font', 'Noto Sans CJK SC', 'PingFang SC', 'Microsoft YaHei', sans-serif;
            font-size: {sub_fs}px;
            font-weight: 500;
            text-shadow: 0 1px 3px rgba(0, 0, 0, 0.85);
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
                    Gtk.main_quit()
                    return False

            # Lock state
            locked = data.get('locked', True)
            if locked != self.locked:
                self.set_locked_state(locked)

            # Font size, dual line, align, bg
            font_size = data.get('font_size', 20)
            dual_line = data.get('dual_line', True)
            align = data.get('align', 'center')
            bg = data.get('bg', 'translucent')
            pos_x = data.get('pos_x')
            pos_y = data.get('pos_y')

            style_changed = False
            if accent != self.current_accent or subtext != self.current_subtext:
                self.current_accent = accent
                self.current_subtext = subtext
                style_changed = True

            if font_size != self.font_size or bg != self.bg_style:
                self.font_size = font_size
                self.bg_style = bg
                style_changed = True

            if style_changed:
                self.update_style()

            self.dual_line = dual_line

            # Align
            if align != self.align:
                self.align = align
                if align == 'left':
                    self.label_main.set_xalign(0.0)
                    self.label_sub.set_xalign(0.0)
                elif align == 'right':
                    self.label_main.set_xalign(1.0)
                    self.label_sub.set_xalign(1.0)
                else:
                    self.label_main.set_xalign(0.5)
                    self.label_sub.set_xalign(0.5)

            # Position from host if not currently dragging
            if not self.dragging:
                if pos_x is None or pos_y is None:
                    if self.custom_pos_x is not None or self.custom_pos_y is not None:
                        self.reset_to_default_position()
                elif pos_x != self.custom_pos_x or pos_y != self.custom_pos_y:
                    self.apply_custom_position(pos_x, pos_y)

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

            if self.dual_line and line2:
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
        win.reset_to_default_position()

    Gtk.main()


if __name__ == '__main__':
    main()
