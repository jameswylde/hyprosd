use crate::OsdEvent;
use crate::config::Config;
use gtk4::glib::object::Cast;
use gtk4::prelude::*;
use gtk4_layer_shell::{self as layer_shell, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

pub struct OsdUi {
    window: gtk4::Window,
    icon: gtk4::Image,
    bar_area: gtk4::DrawingArea,
    drawing: gtk4::DrawingArea,
    overlay: gtk4::Overlay,
    hide_source: Rc<RefCell<Option<gtk4::glib::SourceId>>>,
    draw_state: Rc<RefCell<DrawState>>,
    config: Config,
}

impl OsdUi {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let window = gtk4::Window::new();
        window.set_decorated(false);
        window.set_resizable(false);
        window.set_visible(false);
        window.set_default_size(config.osd.width, config.osd.height);
        window.set_widget_name("hyprosd-window");
        window.set_css_classes(&["hyprosd-transparent"]);

        window.init_layer_shell();
        // this is what hyprctl layers shows, and what users can target with layer rules
        window.set_namespace("hyprosd");
        window.set_layer(layer_shell::Layer::Overlay);
        window.set_anchor(layer_shell::Edge::Bottom, true);
        window.set_anchor(layer_shell::Edge::Left, true);
        window.set_anchor(layer_shell::Edge::Right, false);
        window.set_exclusive_zone(0);
        window.set_margin(layer_shell::Edge::Bottom, config.osd.offset_y);
        window.connect_realize(|window| {
            // the rounded corners need the compositor to treat the surface as non-opaque
            let surface = window.surface();
            surface.set_opaque_region(None);
        });
        if let Some(display) = gtk4::gdk::Display::default() {
            let monitors = display.monitors();
            if let Some(monitor) = monitors
                .item(0)
                .and_then(|obj| obj.downcast::<gtk4::gdk::Monitor>().ok())
            {
                window.set_monitor(&monitor);
                let geometry = monitor.geometry();
                // layer-shell anchors to edges, so we center by setting the left margin ourselves
                let margin_left = ((geometry.width() - config.osd.width).max(0)) / 2;
                window.set_margin(layer_shell::Edge::Left, margin_left);
            }
        }

        let root = gtk4::CenterBox::new();
        root.set_hexpand(true);
        root.set_vexpand(true);
        root.set_halign(gtk4::Align::Fill);
        root.set_widget_name("hyprosd-root");
        root.set_css_classes(&["hyprosd-transparent"]);

        let drawing = gtk4::DrawingArea::new();
        drawing.set_content_width(config.osd.width);
        drawing.set_content_height(config.osd.height);
        drawing.set_widget_name("hyprosd-surface");
        drawing.set_css_classes(&["hyprosd-transparent"]);

        let draw_state = Rc::new(RefCell::new(DrawState::new()));
        let draw_state_ref = draw_state.clone();
        let config_ref = config.clone();
        drawing.set_draw_func(move |_, cr, width, height| {
            let state = draw_state_ref.borrow();
            // clear the whole drawing area first so only our rounded shape remains
            cr.set_operator(gtk4::cairo::Operator::Source);
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            let _ = cr.paint();
            cr.set_operator(gtk4::cairo::Operator::Over);

            let (bg_r, bg_g, bg_b, bg_a) =
                parse_rgba(&config_ref.theme.background).unwrap_or((0.17, 0.17, 0.17, 0.94));
            let radius = config_ref.theme.corner_radius as f64;
            match state.mode {
                DrawMode::Lock => {
                    rounded_rect(cr, 0.0, 0.0, width as f64, height as f64, radius);
                    cr.set_source_rgba(bg_r, bg_g, bg_b, bg_a);
                    let _ = cr.fill();
                }
                DrawMode::Value { level } => {
                    rounded_rect(cr, 0.0, 0.0, width as f64, height as f64, radius);
                    cr.set_source_rgba(bg_r, bg_g, bg_b, bg_a);
                    let _ = cr.fill();
                    let _ = level;
                }
            }
            let (fg_r, fg_g, fg_b, _) =
                parse_rgba(&config_ref.theme.foreground).unwrap_or((1.0, 1.0, 1.0, 1.0));
            cr.set_source_rgba(fg_r, fg_g, fg_b, 1.0);
            rounded_rect(
                cr,
                0.5,
                0.5,
                width as f64 - 1.0,
                height as f64 - 1.0,
                radius,
            );
            let _ = cr.stroke();
        });

        let icon = gtk4::Image::from_icon_name("audio-volume-high-symbolic");
        icon.set_pixel_size(config.theme.icon_size);
        icon.set_widget_name("hyprosd-icon");
        icon.set_halign(gtk4::Align::Center);
        icon.set_valign(gtk4::Align::Center);

        let bar_area = gtk4::DrawingArea::new();
        let bar_width =
            (config.osd.width - (config.theme.padding * 2) - config.theme.icon_size - 18).max(80);
        let bar_height = config.osd.bar_height.max(1);
        bar_area.set_content_width(bar_width);
        bar_area.set_content_height(bar_height);
        bar_area.set_size_request(bar_width, bar_height);
        bar_area.set_widget_name("hyprosd-bar");

        let draw_state_ref = draw_state.clone();
        let config_ref = config.clone();
        bar_area.set_draw_func(move |_, cr, width, height| {
            let state = draw_state_ref.borrow();
            let DrawMode::Value { level } = state.mode else {
                return;
            };
            let (fg_r, fg_g, fg_b, fg_a) =
                parse_rgba(&config_ref.theme.foreground).unwrap_or((1.0, 1.0, 1.0, 1.0));
            let bar_height = config_ref.osd.bar_height.max(1) as f64;
            let y = ((height as f64) - bar_height) / 2.0;
            cr.rectangle(0.0, y, width as f64, bar_height);
            cr.set_source_rgba(fg_r, fg_g, fg_b, 0.35);
            let _ = cr.fill();

            let fill_width = (width as f64) * (level as f64 / 100.0);
            cr.rectangle(0.0, y, fill_width, bar_height);
            cr.set_source_rgba(fg_r, fg_g, fg_b, fg_a);
            let _ = cr.fill();
        });

        let content = gtk4::Box::new(gtk4::Orientation::Horizontal, 18);
        content.set_halign(gtk4::Align::Center);
        content.set_valign(gtk4::Align::Center);
        content.set_widget_name("hyprosd-content");
        content.set_css_classes(&["hyprosd-transparent"]);
        content.set_margin_start(config.theme.padding);
        content.set_margin_end(config.theme.padding);
        content.append(&icon);
        content.append(&bar_area);

        let overlay = gtk4::Overlay::new();
        overlay.set_size_request(config.osd.width, config.osd.height);
        overlay.set_child(Some(&drawing));
        overlay.add_overlay(&content);
        overlay.set_halign(gtk4::Align::Center);
        overlay.set_valign(gtk4::Align::Center);
        overlay.set_css_classes(&["hyprosd-transparent"]);

        root.set_center_widget(Some(&overlay));
        window.set_child(Some(&root));

        apply_css(config)?;

        Ok(Self {
            window,
            icon,
            bar_area,
            drawing,
            overlay,
            hide_source: Rc::new(RefCell::new(None)),
            draw_state,
            config: config.clone(),
        })
    }

    pub fn show_event(&mut self, event: OsdEvent) {
        match event {
            OsdEvent::Volume { level, muted } => {
                self.set_layout(LayoutKind::Value);
                let icon = if muted || level == 0 {
                    "audio-volume-muted-symbolic"
                } else if level < 35 {
                    "audio-volume-low-symbolic"
                } else if level < 70 {
                    "audio-volume-medium-symbolic"
                } else {
                    "audio-volume-high-symbolic"
                };
                self.icon.set_from_icon_name(Some(icon));
                if muted || level == 0 {
                    self.icon.add_css_class("hyprosd-muted");
                } else {
                    self.icon.remove_css_class("hyprosd-muted");
                }
                self.draw_state.borrow_mut().set_level(level);
                self.bar_area.queue_draw();
                self.drawing.queue_draw();
            }
            OsdEvent::Brightness { level } => {
                self.set_layout(LayoutKind::Value);
                self.icon
                    .set_from_icon_name(Some("display-brightness-symbolic"));
                self.icon.remove_css_class("hyprosd-muted");
                self.draw_state.borrow_mut().set_level(level);
                self.bar_area.queue_draw();
                self.drawing.queue_draw();
            }
            OsdEvent::CapsLock { on } => {
                self.set_layout(LayoutKind::Lock);
                set_icon_with_fallback(&self.icon, "caps-lock-symbolic", "input-keyboard-symbolic");
                if on {
                    self.icon.remove_css_class("hyprosd-muted");
                } else {
                    self.icon.add_css_class("hyprosd-muted");
                }
                self.draw_state.borrow_mut().set_lock();
                self.bar_area.queue_draw();
                self.drawing.queue_draw();
            }
            OsdEvent::NumLock { on } => {
                self.set_layout(LayoutKind::Lock);
                self.icon
                    .set_from_icon_name(Some("input-keyboard-symbolic"));
                if on {
                    self.icon.remove_css_class("hyprosd-muted");
                } else {
                    self.icon.add_css_class("hyprosd-muted");
                }
                self.draw_state.borrow_mut().set_lock();
                self.bar_area.queue_draw();
                self.drawing.queue_draw();
            }
        }

        if let Some(source) = self.hide_source.borrow_mut().take() {
            source.remove();
        }

        self.window.set_visible(true);
        let window = self.window.clone();
        let hide_source = self.hide_source.clone();
        let timeout = self.config.osd.timeout_ms;
        // each event replaces the old timer so repeated key presses keep the osd visible
        let source_id =
            gtk4::glib::timeout_add_local(std::time::Duration::from_millis(timeout), move || {
                window.set_visible(false);
                *hide_source.borrow_mut() = None;
                gtk4::glib::ControlFlow::Break
            });
        *self.hide_source.borrow_mut() = Some(source_id);
    }

    fn set_layout(&mut self, kind: LayoutKind) {
        let (width, height) = match kind {
            LayoutKind::Value => (self.config.osd.width, self.config.osd.height),
            LayoutKind::Lock => (self.config.osd.lock_size, self.config.osd.lock_size),
        };
        self.window.set_default_size(width, height);
        self.drawing.set_content_width(width);
        self.drawing.set_content_height(height);
        self.overlay.set_size_request(width, height);
        if matches!(kind, LayoutKind::Lock) {
            self.bar_area.set_visible(false);
        } else {
            self.bar_area.set_visible(true);
        }
        if let Some(display) = gtk4::gdk::Display::default() {
            let monitors = display.monitors();
            if let Some(monitor) = monitors
                .item(0)
                .and_then(|obj| obj.downcast::<gtk4::gdk::Monitor>().ok())
            {
                self.window.set_monitor(&monitor);
                let geometry = monitor.geometry();
                // lock osds are narrower, so recenter whenever the layout changes
                let margin_left = ((geometry.width() - width).max(0)) / 2;
                self.window.set_margin(layer_shell::Edge::Left, margin_left);
            }
        }
    }
}

fn set_icon_with_fallback(icon: &gtk4::Image, primary: &str, fallback: &str) {
    let display = match gtk4::gdk::Display::default() {
        Some(display) => display,
        None => {
            icon.set_from_icon_name(Some(fallback));
            return;
        }
    };
    let theme = gtk4::IconTheme::for_display(&display);
    // some icon themes do not ship a caps-lock symbol
    if theme.has_icon(primary) {
        icon.set_from_icon_name(Some(primary));
    } else {
        icon.set_from_icon_name(Some(fallback));
    }
}

fn apply_css(config: &Config) -> anyhow::Result<()> {
    let css = format!(
        r#"
        /* keep gtk's window background from showing behind the rounded cairo shape */
        window#hyprosd-window,
        window#hyprosd-window.background,
        window#hyprosd-window.hyprosd-transparent,
        window#hyprosd-window.hyprosd-transparent.background,
        window.hyprosd-transparent,
        window.hyprosd-transparent.background,
        window.hyprosd-transparent > *,
        window#hyprosd-window > decoration,
        window#hyprosd-window > decoration > widget,
        .hyprosd-transparent,
        #hyprosd-root {{
            background: transparent;
            background-color: transparent;
            box-shadow: none;
            border: none;
            outline: none;
        }}
        #hyprosd-surface {{
            background: transparent;
            background-color: transparent;
        }}
        #hyprosd-content {{
            padding: {}px;
            background: transparent;
            background-color: transparent;
        }}
        #hyprosd-icon {{
            color: {};
        }}
        #hyprosd-icon.hyprosd-muted {{
            color: rgba(245, 245, 245, 0.55);
        }}
        "#,
        config.theme.padding, config.theme.foreground,
    );

    let provider = gtk4::CssProvider::new();
    provider.load_from_data(&css);
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("display"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    Ok(())
}

fn rounded_rect(cr: &gtk4::cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    let radius = r.min(w / 2.0).min(h / 2.0);
    cr.new_sub_path();
    cr.arc(
        x + w - radius,
        y + radius,
        radius,
        -std::f64::consts::FRAC_PI_2,
        0.0,
    );
    cr.arc(
        x + w - radius,
        y + h - radius,
        radius,
        0.0,
        std::f64::consts::FRAC_PI_2,
    );
    cr.arc(
        x + radius,
        y + h - radius,
        radius,
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
    );
    cr.arc(
        x + radius,
        y + radius,
        radius,
        std::f64::consts::PI,
        std::f64::consts::PI * 1.5,
    );
    cr.close_path();
}

fn parse_rgba(value: &str) -> Option<(f64, f64, f64, f64)> {
    let value = value.trim().trim_start_matches('#');
    let (r, g, b, a) = match value.len() {
        6 => {
            let r = u8::from_str_radix(&value[0..2], 16).ok()?;
            let g = u8::from_str_radix(&value[2..4], 16).ok()?;
            let b = u8::from_str_radix(&value[4..6], 16).ok()?;
            (r, g, b, 255)
        }
        8 => {
            let r = u8::from_str_radix(&value[0..2], 16).ok()?;
            let g = u8::from_str_radix(&value[2..4], 16).ok()?;
            let b = u8::from_str_radix(&value[4..6], 16).ok()?;
            let a = u8::from_str_radix(&value[6..8], 16).ok()?;
            (r, g, b, a)
        }
        _ => return None,
    };
    Some((
        r as f64 / 255.0,
        g as f64 / 255.0,
        b as f64 / 255.0,
        a as f64 / 255.0,
    ))
}

#[derive(Clone, Copy)]
enum DrawMode {
    Lock,
    Value { level: u8 },
}

struct DrawState {
    mode: DrawMode,
}

impl DrawState {
    fn new() -> Self {
        Self {
            mode: DrawMode::Value { level: 0 },
        }
    }

    fn set_lock(&mut self) {
        self.mode = DrawMode::Lock;
    }

    fn set_level(&mut self, level: u8) {
        self.mode = DrawMode::Value {
            level: level.min(100),
        };
    }
}

#[derive(Clone, Copy)]
enum LayoutKind {
    Lock,
    Value,
}
