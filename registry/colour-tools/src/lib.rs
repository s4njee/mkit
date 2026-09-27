//! Colour selection and tonal grading controls.
extern crate gpui_pre as gpui;

use gpui_pre::{
    App, Bounds, Context, DispatchPhase, EventEmitter, FocusHandle, Focusable, IntoElement,
    KeyBinding, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels,
    Render, Rgba, StatefulInteractiveElement, Window, actions, canvas, div, point, prelude::*, px,
};
use mkit_core::theme::Theme;

pub const KEY_CONTEXT: &str = "MkitColourTools";
actions!(colour_tools, [HueUp, HueDown, SaturationUp, SaturationDown, LightnessUp, LightnessDown]);

pub fn default_key_bindings() -> [KeyBinding; 6] {
    [
        KeyBinding::new("up", HueUp, Some(KEY_CONTEXT)),
        KeyBinding::new("down", HueDown, Some(KEY_CONTEXT)),
        KeyBinding::new("right", SaturationUp, Some(KEY_CONTEXT)),
        KeyBinding::new("left", SaturationDown, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-up", LightnessUp, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-down", LightnessDown, Some(KEY_CONTEXT)),
    ]
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Srgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsl {
    pub hue: f64,
    pub saturation: f64,
    pub lightness: f64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Oklch {
    pub lightness: f64,
    pub chroma: f64,
    pub hue: f64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colour {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

impl Default for Colour {
    fn default() -> Self {
        Self { red: 0.22, green: 0.45, blue: 0.82 }
    }
}
impl Colour {
    pub fn from_srgb(value: Srgb) -> Self {
        Self {
            red: value.red as f64 / 255.0,
            green: value.green as f64 / 255.0,
            blue: value.blue as f64 / 255.0,
        }
    }
    pub fn to_srgb(self) -> Srgb {
        Srgb {
            red: (unit(self.red) * 255.0).round() as u8,
            green: (unit(self.green) * 255.0).round() as u8,
            blue: (unit(self.blue) * 255.0).round() as u8,
        }
    }
    pub fn from_hsl(value: Hsl) -> Self {
        let h = wrap_hue(value.hue) / 360.0;
        let s = unit(value.saturation / 100.0);
        let l = unit(value.lightness / 100.0);
        if s == 0.0 {
            return Self { red: l, green: l, blue: l };
        }
        let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let p = 2.0 * l - q;
        Self {
            red: hue_channel(p, q, h + 1.0 / 3.0),
            green: hue_channel(p, q, h),
            blue: hue_channel(p, q, h - 1.0 / 3.0),
        }
    }
    pub fn to_hsl(self) -> Hsl {
        let (r, g, b) = (unit(self.red), unit(self.green), unit(self.blue));
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let d = max - min;
        let l = (max + min) / 2.0;
        let (h, s) = if d == 0.0 {
            (0.0, 0.0)
        } else {
            let s = d / (1.0 - (2.0 * l - 1.0).abs());
            let h = if max == r {
                60.0 * (((g - b) / d).rem_euclid(6.0))
            } else if max == g {
                60.0 * ((b - r) / d + 2.0)
            } else {
                60.0 * ((r - g) / d + 4.0)
            };
            (h, s)
        };
        Hsl { hue: h, saturation: s * 100.0, lightness: l * 100.0 }
    }
    pub fn from_oklch(value: Oklch) -> Self {
        let l = unit(value.lightness);
        let c = if value.chroma.is_finite() { value.chroma.clamp(0.0, 0.5) } else { 0.0 };
        let h = wrap_hue(value.hue).to_radians();
        let a = c * h.cos();
        let b = c * h.sin();
        let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
        let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
        let s_ = l - 0.0894841775 * a - 1.2914855480 * b;
        let ll = l_ * l_ * l_;
        let mm = m_ * m_ * m_;
        let ss = s_ * s_ * s_;
        Self {
            red: unit(srgb(4.0767416621 * ll - 3.3077115913 * mm + 0.2309699292 * ss)),
            green: unit(srgb(-1.2684380046 * ll + 2.6097574011 * mm - 0.3413193965 * ss)),
            blue: unit(srgb(-0.0041960863 * ll - 0.7034186147 * mm + 1.7076147010 * ss)),
        }
    }
    pub fn to_oklch(self) -> Oklch {
        let r = linear(unit(self.red));
        let g = linear(unit(self.green));
        let b = linear(unit(self.blue));
        let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
        let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
        let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
        let ll = 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s;
        let aa = 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s;
        let bb = 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s;
        Oklch { lightness: ll, chroma: aa.hypot(bb), hue: wrap_hue(bb.atan2(aa).to_degrees()) }
    }
    fn to_rgba(self) -> Rgba {
        Rgba {
            r: unit(self.red) as f32,
            g: unit(self.green) as f32,
            b: unit(self.blue) as f32,
            a: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GradeOffset {
    pub hue: f64,
    pub saturation: f64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Grading {
    pub shadows: GradeOffset,
    pub midtones: GradeOffset,
    pub highlights: GradeOffset,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ColourChanged(pub Colour);
impl EventEmitter<ColourChanged> for ColourTools {}
#[derive(Clone, Debug, PartialEq)]
pub struct GradingChanged(pub Grading);
impl EventEmitter<GradingChanged> for ColourTools {}

/// A stateful colour editing panel. Values are stored in sRGB and converted at the API boundary.
pub struct ColourTools {
    label: String,
    colour: Colour,
    grading: Grading,
    controlled: bool,
    focus: Option<FocusHandle>,
    wheel_bounds: Option<Bounds<Pixels>>,
    grade_bounds: [Option<Bounds<Pixels>>; 3],
    dragging: bool,
    grade_drag: Option<GradeZone>,
    active_grade: Option<GradeZone>,
    numeric_target: Option<NumericField>,
    numeric_buffer: String,
    numeric_original: Colour,
    numeric_focus: [Option<FocusHandle>; 9],
    grade_focus: [Option<FocusHandle>; 3],
}
impl ColourTools {
    pub fn new(label: impl Into<String>, colour: Colour) -> Self {
        Self::base(label, colour, false)
    }
    pub fn controlled(label: impl Into<String>, colour: Colour) -> Self {
        Self::base(label, colour, true)
    }
    fn base(label: impl Into<String>, colour: Colour, controlled: bool) -> Self {
        Self {
            label: label.into(),
            colour: sanitize(colour),
            grading: Grading::default(),
            controlled,
            focus: None,
            wheel_bounds: None,
            grade_bounds: [None; 3],
            dragging: false,
            grade_drag: None,
            active_grade: None,
            numeric_target: None,
            numeric_buffer: String::new(),
            numeric_original: sanitize(colour),
            numeric_focus: std::array::from_fn(|_| None),
            grade_focus: std::array::from_fn(|_| None),
        }
    }
    pub fn colour(&self) -> Colour {
        self.colour
    }
    pub fn grading(&self) -> Grading {
        self.grading
    }
    pub fn set_colour(&mut self, colour: Colour, cx: &mut Context<Self>) {
        self.colour = sanitize(colour);
        cx.notify();
    }
    pub fn set_grading(&mut self, grading: Grading, cx: &mut Context<Self>) {
        self.grading = sanitize_grading(grading);
        cx.notify();
    }
    pub fn set_srgb(&mut self, value: Srgb, cx: &mut Context<Self>) {
        self.propose_colour(Colour::from_srgb(value), cx);
    }
    pub fn set_hsl(&mut self, value: Hsl, cx: &mut Context<Self>) {
        self.propose_colour(Colour::from_hsl(value), cx);
    }
    pub fn set_oklch(&mut self, value: Oklch, cx: &mut Context<Self>) {
        self.propose_colour(Colour::from_oklch(value), cx);
    }
    pub fn set_grade(&mut self, zone: GradeZone, offset: GradeOffset, cx: &mut Context<Self>) {
        let mut next = self.grading;
        *grade_mut(&mut next, zone) =
            GradeOffset { hue: wrap_hue(offset.hue), saturation: unit(offset.saturation) };
        if next != self.grading {
            if !self.controlled {
                self.grading = next;
                cx.notify();
            }
            cx.emit(GradingChanged(next));
        }
    }
    fn propose_colour(&mut self, next: Colour, cx: &mut Context<Self>) {
        let next = sanitize(next);
        if next == self.colour {
            return;
        }
        if !self.controlled {
            self.colour = next;
            cx.notify();
        }
        cx.emit(ColourChanged(next));
    }
    fn adjust(&mut self, hue: f64, sat: f64, light: f64, cx: &mut Context<Self>) {
        if let Some(zone) = self.active_grade {
            let mut next = self.grading;
            let current = *grade_mut(&mut next, zone);
            *grade_mut(&mut next, zone) = GradeOffset {
                hue: wrap_hue(current.hue + hue),
                saturation: unit(current.saturation + sat / 100.0),
            };
            if next != self.grading {
                if !self.controlled {
                    self.grading = next;
                    cx.notify();
                }
                cx.emit(GradingChanged(next));
            }
            return;
        }
        let hsl = self.colour.to_hsl();
        self.propose_colour(
            Colour::from_hsl(Hsl {
                hue: hsl.hue + hue,
                saturation: hsl.saturation + sat,
                lightness: hsl.lightness + light,
            }),
            cx,
        );
    }
    fn begin_numeric(&mut self, field: NumericField, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(focus) = self.numeric_focus[field.index()].as_ref() {
            window.focus(focus, cx);
        }
        self.activate_numeric(field, cx);
    }
    fn activate_numeric(&mut self, field: NumericField, cx: &mut Context<Self>) {
        self.active_grade = None;
        self.numeric_target = Some(field);
        self.numeric_original = self.colour;
        self.numeric_buffer.clear();
        cx.notify();
    }
    fn on_key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some(field) = self.numeric_target else { return };
        let key = event.keystroke.key.as_str();
        match key {
            "enter" => {
                self.numeric_target = None;
                self.numeric_buffer.clear();
                cx.notify();
            }
            "escape" => {
                let original = self.numeric_original;
                self.numeric_target = None;
                self.numeric_buffer.clear();
                self.propose_colour(original, cx);
                cx.notify();
            }
            "backspace" => {
                self.numeric_buffer.pop();
                self.apply_numeric(field, cx);
            }
            digit if digit.len() == 1 && digit.as_bytes()[0].is_ascii_digit() => {
                self.numeric_buffer.push_str(key);
                self.apply_numeric(field, cx);
            }
            "." | "period" if !self.numeric_buffer.contains('.') => {
                self.numeric_buffer.push('.');
                self.apply_numeric(field, cx);
            }
            "-" if self.numeric_buffer.is_empty() => {
                self.numeric_buffer.push('-');
                cx.notify();
            }
            _ => {}
        }
    }
    fn apply_numeric(&mut self, field: NumericField, cx: &mut Context<Self>) {
        let Ok(value) = self.numeric_buffer.parse::<f64>() else {
            cx.notify();
            return;
        };
        let hsl = self.colour.to_hsl();
        let lch = self.colour.to_oklch();
        let rgb = self.colour.to_srgb();
        let next = match field {
            NumericField::Red => Colour::from_srgb(Srgb {
                red: value.round().clamp(0., 255.) as u8,
                green: rgb.green,
                blue: rgb.blue,
            }),
            NumericField::Green => Colour::from_srgb(Srgb {
                red: rgb.red,
                green: value.round().clamp(0., 255.) as u8,
                blue: rgb.blue,
            }),
            NumericField::Blue => Colour::from_srgb(Srgb {
                red: rgb.red,
                green: rgb.green,
                blue: value.round().clamp(0., 255.) as u8,
            }),
            NumericField::HslHue => Colour::from_hsl(Hsl {
                hue: value,
                saturation: hsl.saturation,
                lightness: hsl.lightness,
            }),
            NumericField::HslSaturation => {
                Colour::from_hsl(Hsl { hue: hsl.hue, saturation: value, lightness: hsl.lightness })
            }
            NumericField::HslLightness => {
                Colour::from_hsl(Hsl { hue: hsl.hue, saturation: hsl.saturation, lightness: value })
            }
            NumericField::OklchLightness => {
                Colour::from_oklch(Oklch { lightness: value, chroma: lch.chroma, hue: lch.hue })
            }
            NumericField::OklchChroma => {
                Colour::from_oklch(Oklch { lightness: lch.lightness, chroma: value, hue: lch.hue })
            }
            NumericField::OklchHue => Colour::from_oklch(Oklch {
                lightness: lch.lightness,
                chroma: lch.chroma,
                hue: value,
            }),
        };
        self.propose_colour(next, cx);
    }
    fn hue_up(&mut self, _: &HueUp, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust(1., 0., 0., cx);
    }
    fn hue_down(&mut self, _: &HueDown, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust(-1., 0., 0., cx);
    }
    fn saturation_up(&mut self, _: &SaturationUp, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust(0., 1., 0., cx);
    }
    fn saturation_down(&mut self, _: &SaturationDown, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust(0., -1., 0., cx);
    }
    fn lightness_up(&mut self, _: &LightnessUp, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust(0., 0., 1., cx);
    }
    fn lightness_down(&mut self, _: &LightnessDown, _: &mut Window, cx: &mut Context<Self>) {
        self.adjust(0., 0., -1., cx);
    }
    fn update_from_pointer(&mut self, position: gpui_pre::Point<Pixels>, cx: &mut Context<Self>) {
        let Some(bounds) = self.wheel_bounds else { return };
        let center = point(
            bounds.origin.x + bounds.size.width / 2.,
            bounds.origin.y + bounds.size.height / 2.,
        );
        let dx = f32::from(position.x - center.x);
        let dy = f32::from(position.y - center.y);
        let radius = f32::from(bounds.size.width.min(bounds.size.height)) / 2.0;
        let hsl = self.colour.to_hsl();
        self.propose_colour(
            Colour::from_hsl(Hsl {
                hue: wheel_hue(dy.atan2(dx) as f64),
                saturation: (dx.hypot(dy) / radius * 100.0).clamp(0., 100.) as f64,
                lightness: hsl.lightness,
            }),
            cx,
        );
    }
    fn update_grade_from_pointer(
        &mut self,
        zone: GradeZone,
        bounds: Bounds<Pixels>,
        position: gpui_pre::Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        let center = point(
            bounds.origin.x + bounds.size.width / 2.,
            bounds.origin.y + bounds.size.height / 2.,
        );
        let dx = f32::from(position.x - center.x);
        let dy = f32::from(position.y - center.y);
        let radius = (f32::from(bounds.size.width.min(bounds.size.height)) / 2.).max(1.);
        self.set_grade(
            zone,
            GradeOffset {
                hue: wheel_hue(dy.atan2(dx) as f64),
                saturation: (dx.hypot(dy) / radius).clamp(0., 1.) as f64,
            },
            cx,
        );
    }
}

impl Focusable for ColourTools {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus.clone().unwrap_or_else(|| cx.focus_handle())
    }
}
impl Render for ColourTools {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let hsl = self.colour.to_hsl();
        let rgb = self.colour.to_srgb();
        let lch = self.colour.to_oklch();
        let focus = self.focus.get_or_insert_with(|| cx.focus_handle()).clone();
        let mut numeric_focus = Vec::with_capacity(9);
        for index in 0..9 {
            if self.numeric_focus[index].is_none() {
                let handle = cx.focus_handle().tab_index(index as isize).tab_stop(true);
                self.numeric_focus[index] = Some(handle);
            }
            numeric_focus.push(self.numeric_focus[index].as_ref().unwrap().clone());
        }
        let wheel_entity = cx.entity();
        let root = div().id("mkit-colour-tools").key_context(KEY_CONTEXT)
            .flex().flex_col().gap(px(theme.spacing.medium)).p(px(theme.spacing.medium)).w_full()
            .rounded(px(theme.radii.medium)).border(px(theme.borders.hairline)).border_color(theme.colors.border).bg(theme.colors.surface)
            .text_color(theme.colors.text).role(gpui_pre::accesskit::Role::Group).aria_label(self.label.clone())
            .aria_description("Hue and saturation wheel with lightness control, colour values, and tonal grading wheels.");
        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .child(div().text_sm().font_weight(gpui_pre::FontWeight::SEMIBOLD).child("Colour"))
            .child(
                div()
                    .text_xs()
                    .text_color(theme.colors.text_muted)
                    .child(format!("#{:02X}{:02X}{:02X}", rgb.red, rgb.green, rgb.blue)),
            );
        let wheel = div()
            .id("colour-hue-saturation-wheel")
            .size(px(theme.controls.large * 4.5))
            .track_focus(&focus)
            .tab_stop(true)
            .on_action(cx.listener(Self::hue_up))
            .on_action(cx.listener(Self::hue_down))
            .on_action(cx.listener(Self::saturation_up))
            .on_action(cx.listener(Self::saturation_down))
            .on_action(cx.listener(Self::lightness_up))
            .on_action(cx.listener(Self::lightness_down))
            .role(gpui_pre::accesskit::Role::Slider)
            .aria_label("Hue and saturation")
            .aria_value(format!(
                "Hue {:.0} degrees, saturation {:.0} percent",
                hsl.hue, hsl.saturation
            ))
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, (), window, cx| {
                        wheel_entity.update(cx, |picker, _| picker.wheel_bounds = Some(bounds));
                        let center = point(
                            bounds.origin.x + bounds.size.width / 2.,
                            bounds.origin.y + bounds.size.height / 2.,
                        );
                        let radius = bounds.size.width.min(bounds.size.height) / 2.0
                            - px(theme.spacing.xsmall);
                        let current_lightness = wheel_entity.read(cx).colour.to_hsl().lightness;
                        for ring in 0..5 {
                            let inner = radius * (ring as f32 / 5.0);
                            let outer = radius * ((ring + 1) as f32 / 5.0);
                            let saturation = (ring as f64 + 0.5) / 5.0 * 100.0;
                            for i in 0..72 {
                                let a0 = i as f32 / 72.0 * std::f32::consts::TAU;
                                let a1 = (i + 1) as f32 / 72.0 * std::f32::consts::TAU;
                                let mut path = gpui_pre::PathBuilder::fill();
                                path.move_to(point(
                                    center.x + inner * a0.cos(),
                                    center.y + inner * a0.sin(),
                                ));
                                path.line_to(point(
                                    center.x + outer * a0.cos(),
                                    center.y + outer * a0.sin(),
                                ));
                                path.line_to(point(
                                    center.x + outer * a1.cos(),
                                    center.y + outer * a1.sin(),
                                ));
                                path.line_to(point(
                                    center.x + inner * a1.cos(),
                                    center.y + inner * a1.sin(),
                                ));
                                path.close();
                                if let Ok(path) = path.build() {
                                    let hue =
                                        wheel_hue((i as f64 + 0.5) / 72.0 * std::f64::consts::TAU);
                                    let color = Colour::from_hsl(Hsl {
                                        hue,
                                        saturation,
                                        lightness: current_lightness,
                                    });
                                    window.paint_path(path, color.to_rgba());
                                }
                            }
                        }
                        let mut marker = gpui_pre::PathBuilder::stroke(px(theme.borders.strong));
                        let at = {
                            let hsl = wheel_entity.read(cx).colour.to_hsl();
                            let a = (hsl.hue - 90.0).to_radians();
                            let rr = radius * (hsl.saturation as f32 / 100.0);
                            point(center.x + rr * a.cos() as f32, center.y + rr * a.sin() as f32)
                        };
                        marker.move_to(point(at.x - px(theme.controls.xsmall / 3.0), at.y));
                        marker.line_to(point(at.x + px(theme.controls.xsmall / 3.0), at.y));
                        marker.move_to(point(at.x, at.y - px(theme.controls.xsmall / 3.0)));
                        marker.line_to(point(at.x, at.y + px(theme.controls.xsmall / 3.0)));
                        if let Ok(path) = marker.build() {
                            window.paint_path(path, theme.colors.text);
                        }
                        let down = wheel_entity.clone();
                        window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                            if phase == DispatchPhase::Capture
                                && event.button == MouseButton::Left
                                && bounds.contains(&event.position)
                            {
                                down.update(cx, |picker, cx| {
                                    picker.dragging = true;
                                    picker.active_grade = None;
                                    window.focus(&picker.focus_handle(cx), cx);
                                    picker.update_from_pointer(event.position, cx);
                                });
                            }
                        });
                        let moved = wheel_entity.clone();
                        window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                            if phase == DispatchPhase::Capture {
                                moved.update(cx, |picker, cx| {
                                    if picker.dragging {
                                        picker.update_from_pointer(event.position, cx);
                                    }
                                });
                            }
                        });
                        let up = wheel_entity.clone();
                        window.on_mouse_event(move |_: &MouseUpEvent, phase, _, cx| {
                            if phase == DispatchPhase::Capture {
                                up.update(cx, |picker, _| picker.dragging = false);
                            }
                        });
                    },
                )
                .size_full(),
            );
        let wheel_row =
            div().flex().items_center().gap(px(theme.spacing.medium)).child(wheel).child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(theme.spacing.xsmall))
                    .child(
                        div().text_xs().text_color(theme.colors.text_muted).child("Numeric entry"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(theme.spacing.xsmall))
                            .child(numeric_cell(
                                "R",
                                rgb.red.to_string(),
                                NumericField::Red,
                                self,
                                &numeric_focus[0],
                                cx,
                                theme,
                            ))
                            .child(numeric_cell(
                                "G",
                                rgb.green.to_string(),
                                NumericField::Green,
                                self,
                                &numeric_focus[1],
                                cx,
                                theme,
                            ))
                            .child(numeric_cell(
                                "B",
                                rgb.blue.to_string(),
                                NumericField::Blue,
                                self,
                                &numeric_focus[2],
                                cx,
                                theme,
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(theme.spacing.xsmall))
                            .child(numeric_cell(
                                "H°",
                                format!("{:.0}", hsl.hue),
                                NumericField::HslHue,
                                self,
                                &numeric_focus[3],
                                cx,
                                theme,
                            ))
                            .child(numeric_cell(
                                "S%",
                                format!("{:.0}", hsl.saturation),
                                NumericField::HslSaturation,
                                self,
                                &numeric_focus[4],
                                cx,
                                theme,
                            ))
                            .child(numeric_cell(
                                "L%",
                                format!("{:.0}", hsl.lightness),
                                NumericField::HslLightness,
                                self,
                                &numeric_focus[5],
                                cx,
                                theme,
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(theme.spacing.xsmall))
                            .child(numeric_cell(
                                "L",
                                format!("{:.3}", lch.lightness),
                                NumericField::OklchLightness,
                                self,
                                &numeric_focus[6],
                                cx,
                                theme,
                            ))
                            .child(numeric_cell(
                                "C",
                                format!("{:.3}", lch.chroma),
                                NumericField::OklchChroma,
                                self,
                                &numeric_focus[7],
                                cx,
                                theme,
                            ))
                            .child(numeric_cell(
                                "H°",
                                format!("{:.0}", lch.hue),
                                NumericField::OklchHue,
                                self,
                                &numeric_focus[8],
                                cx,
                                theme,
                            )),
                    ),
            );
        let mut grades = div().flex().gap(px(theme.spacing.small));
        let grade_focus: Vec<FocusHandle> = self
            .grade_focus
            .iter_mut()
            .map(|slot| slot.get_or_insert_with(|| cx.focus_handle().tab_stop(true)).clone())
            .collect();
        for (index, (name, _offset)) in [
            ("Shadows", self.grading.shadows),
            ("Midtones", self.grading.midtones),
            ("Highlights", self.grading.highlights),
        ]
        .into_iter()
        .enumerate()
        {
            let entity = cx.entity();
            grades = grades.child(
                div()
                    .id(zone_for(index).id())
                    .track_focus(&grade_focus[index])
                    .tab_stop(true)
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(theme.spacing.xsmall))
                    .role(gpui_pre::accesskit::Role::Slider)
                    .aria_label(name)
                    .aria_value(format!(
                        "Hue {:.0} degrees, saturation {:.0} percent",
                        _offset.hue,
                        _offset.saturation * 100.0
                    ))
                    .on_action(cx.listener(move |picker, _: &HueUp, _, cx| {
                        picker.active_grade = Some(zone_for(index));
                        picker.adjust(1., 0., 0., cx);
                    }))
                    .on_action(cx.listener(move |picker, _: &HueDown, _, cx| {
                        picker.active_grade = Some(zone_for(index));
                        picker.adjust(-1., 0., 0., cx);
                    }))
                    .on_action(cx.listener(move |picker, _: &SaturationUp, _, cx| {
                        picker.active_grade = Some(zone_for(index));
                        picker.adjust(0., 1., 0., cx);
                    }))
                    .on_action(cx.listener(move |picker, _: &SaturationDown, _, cx| {
                        picker.active_grade = Some(zone_for(index));
                        picker.adjust(0., -1., 0., cx);
                    }))
                    .child(
                        canvas(
                            move |_, _, _| (),
                            move |bounds, (), window, cx| {
                                entity.update(cx, |picker, _| {
                                    picker.grade_bounds[index] = Some(bounds)
                                });
                                let center = point(
                                    bounds.origin.x + bounds.size.width / 2.,
                                    bounds.origin.y + bounds.size.height / 2.,
                                );
                                let radius = bounds.size.width.min(bounds.size.height) / 2.
                                    - px(theme.borders.regular);
                                for ring_index in 0..4 {
                                    let inner = radius * (ring_index as f32 / 4.0);
                                    let outer = radius * ((ring_index + 1) as f32 / 4.0);
                                    let saturation = (ring_index as f64 + 0.5) / 4.0 * 100.0;
                                    for segment in 0..48 {
                                        let a0 = segment as f32 / 48.0 * std::f32::consts::TAU;
                                        let a1 =
                                            (segment + 1) as f32 / 48.0 * std::f32::consts::TAU;
                                        let mut wedge = gpui_pre::PathBuilder::fill();
                                        wedge.move_to(point(
                                            center.x + inner * a0.cos(),
                                            center.y + inner * a0.sin(),
                                        ));
                                        wedge.line_to(point(
                                            center.x + outer * a0.cos(),
                                            center.y + outer * a0.sin(),
                                        ));
                                        wedge.line_to(point(
                                            center.x + outer * a1.cos(),
                                            center.y + outer * a1.sin(),
                                        ));
                                        wedge.line_to(point(
                                            center.x + inner * a1.cos(),
                                            center.y + inner * a1.sin(),
                                        ));
                                        wedge.close();
                                        if let Ok(path) = wedge.build() {
                                            let hue = wheel_hue(
                                                (segment as f64 + 0.5) / 48.0
                                                    * std::f64::consts::TAU,
                                            );
                                            let color = Colour::from_hsl(Hsl {
                                                hue,
                                                saturation,
                                                lightness: 50.0,
                                            });
                                            window.paint_path(path, color.to_rgba());
                                        }
                                    }
                                }
                                let mut ring =
                                    gpui_pre::PathBuilder::stroke(px(theme.borders.regular));
                                for i in 0..48 {
                                    let a = i as f32 / 48. * std::f32::consts::TAU;
                                    let b = (i + 1) as f32 / 48. * std::f32::consts::TAU;
                                    ring.move_to(point(
                                        center.x + radius * a.cos(),
                                        center.y + radius * a.sin(),
                                    ));
                                    ring.line_to(point(
                                        center.x + radius * b.cos(),
                                        center.y + radius * b.sin(),
                                    ));
                                }
                                if let Ok(path) = ring.build() {
                                    window.paint_path(path, theme.colors.border);
                                }
                                let off = match index {
                                    0 => entity.read(cx).grading.shadows,
                                    1 => entity.read(cx).grading.midtones,
                                    _ => entity.read(cx).grading.highlights,
                                };
                                let a = (off.hue - 90.).to_radians() as f32;
                                let rr = radius * off.saturation as f32;
                                let mut mark =
                                    gpui_pre::PathBuilder::stroke(px(theme.borders.strong));
                                mark.move_to(point(
                                    center.x + rr * a.cos() - px(theme.controls.xsmall / 4.),
                                    center.y + rr * a.sin(),
                                ));
                                mark.line_to(point(
                                    center.x + rr * a.cos() + px(theme.controls.xsmall / 4.),
                                    center.y + rr * a.sin(),
                                ));
                                if let Ok(path) = mark.build() {
                                    window.paint_path(path, theme.colors.accent);
                                }
                                let zone = zone_for(index);
                                let down = entity.clone();
                                window.on_mouse_event(
                                    move |event: &MouseDownEvent, phase, window, cx| {
                                        if phase == DispatchPhase::Capture
                                            && event.button == MouseButton::Left
                                            && bounds.contains(&event.position)
                                        {
                                            down.update(cx, |picker, cx| {
                                                picker.grade_drag = Some(zone);
                                                picker.active_grade = Some(zone);
                                                window.focus(&picker.focus_handle(cx), cx);
                                                picker.update_grade_from_pointer(
                                                    zone,
                                                    bounds,
                                                    event.position,
                                                    cx,
                                                );
                                            });
                                        }
                                    },
                                );
                                let moved = entity.clone();
                                window.on_mouse_event(
                                    move |event: &MouseMoveEvent, phase, _, cx| {
                                        if phase == DispatchPhase::Capture {
                                            moved.update(cx, |picker, cx| {
                                                if picker.grade_drag == Some(zone) {
                                                    picker.update_grade_from_pointer(
                                                        zone,
                                                        bounds,
                                                        event.position,
                                                        cx,
                                                    );
                                                }
                                            });
                                        }
                                    },
                                );
                                let up = entity.clone();
                                window.on_mouse_event(move |_: &MouseUpEvent, phase, _, cx| {
                                    if phase == DispatchPhase::Capture {
                                        up.update(cx, |picker, _| picker.grade_drag = None);
                                    }
                                });
                            },
                        )
                        .size(px(theme.controls.large * 1.6)),
                    )
                    .child(div().text_xs().text_color(theme.colors.text_muted).child(name)),
            );
        }
        let sample_unavailable = div()
            .flex()
            .items_center()
            .justify_between()
            .rounded(px(theme.radii.small))
            .border(px(theme.borders.hairline))
            .border_color(theme.colors.border)
            .px(px(theme.spacing.small))
            .py(px(theme.spacing.xsmall))
            .child(div().text_xs().child("Eyedropper unavailable on this platform"))
            .child(div().text_xs().text_color(theme.colors.disabled).child("Unavailable"));
        root.child(header)
            .child(wheel_row)
            .child(
                div().text_xs().font_weight(gpui_pre::FontWeight::SEMIBOLD).child("Colour grading"),
            )
            .child(grades)
            .child(sample_unavailable)
    }
}

fn numeric_cell(
    label: &'static str,
    value: String,
    field: NumericField,
    picker: &ColourTools,
    focus: &FocusHandle,
    cx: &mut Context<ColourTools>,
    theme: Theme,
) -> impl IntoElement {
    let editing = picker.numeric_target == Some(field);
    let shown = if editing { picker.numeric_buffer.clone() } else { value.clone() };
    div()
        .id(format!("colour-value-{}", field.id()))
        .track_focus(focus)
        .tab_stop(true)
        .flex()
        .flex_col()
        .gap(px(theme.spacing.xsmall))
        .px(px(theme.spacing.xsmall))
        .py(px(theme.spacing.xsmall))
        .rounded(px(theme.radii.small))
        .border(px(theme.borders.hairline))
        .border_color(if editing { theme.colors.focus } else { theme.colors.border })
        .bg(theme.colors.elevated_surface)
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _event, window, cx| this.begin_numeric(field, window, cx)),
        )
        .on_key_down(cx.listener(move |this, event: &KeyDownEvent, window, cx| {
            if this.numeric_target == Some(field) {
                this.on_key_down(event, window, cx);
            } else if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                this.begin_numeric(field, window, cx);
            }
        }))
        .role(gpui_pre::accesskit::Role::SpinButton)
        .aria_label(format!("{label} numeric colour value"))
        .aria_numeric_value(shown.parse().unwrap_or_else(|_| value.parse().unwrap_or(0.0)))
        .aria_min_numeric_value(field.range().0)
        .aria_max_numeric_value(field.range().1)
        .aria_value(if editing { shown.clone() } else { value })
        .child(div().text_xs().text_color(theme.colors.text_muted).child(label))
        .child(div().text_sm().child(if editing && shown.is_empty() {
            "Type value".to_owned()
        } else {
            shown
        }))
}
fn unit(v: f64) -> f64 {
    if v.is_finite() { v.clamp(0., 1.) } else { 0. }
}
fn wrap_hue(h: f64) -> f64 {
    if h.is_finite() { h.rem_euclid(360.) } else { 0. }
}
fn sanitize(c: Colour) -> Colour {
    Colour { red: unit(c.red), green: unit(c.green), blue: unit(c.blue) }
}
fn sanitize_grading(g: Grading) -> Grading {
    Grading {
        shadows: GradeOffset {
            hue: wrap_hue(g.shadows.hue),
            saturation: unit(g.shadows.saturation),
        },
        midtones: GradeOffset {
            hue: wrap_hue(g.midtones.hue),
            saturation: unit(g.midtones.saturation),
        },
        highlights: GradeOffset {
            hue: wrap_hue(g.highlights.hue),
            saturation: unit(g.highlights.saturation),
        },
    }
}
fn grade_mut(g: &mut Grading, z: GradeZone) -> &mut GradeOffset {
    match z {
        GradeZone::Shadows => &mut g.shadows,
        GradeZone::Midtones => &mut g.midtones,
        GradeZone::Highlights => &mut g.highlights,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GradeZone {
    Shadows,
    Midtones,
    Highlights,
}
impl GradeZone {
    fn id(self) -> &'static str {
        match self {
            Self::Shadows => "colour-grade-shadows",
            Self::Midtones => "colour-grade-midtones",
            Self::Highlights => "colour-grade-highlights",
        }
    }
}
fn zone_for(index: usize) -> GradeZone {
    match index {
        0 => GradeZone::Shadows,
        1 => GradeZone::Midtones,
        _ => GradeZone::Highlights,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NumericField {
    Red,
    Green,
    Blue,
    HslHue,
    HslSaturation,
    HslLightness,
    OklchLightness,
    OklchChroma,
    OklchHue,
}
impl NumericField {
    fn range(self) -> (f64, f64) {
        match self {
            Self::Red | Self::Green | Self::Blue => (0.0, 255.0),
            Self::HslHue | Self::OklchHue => (0.0, 360.0),
            Self::HslSaturation | Self::HslLightness => (0.0, 100.0),
            Self::OklchLightness => (0.0, 1.0),
            Self::OklchChroma => (0.0, 0.5),
        }
    }
    fn index(self) -> usize {
        match self {
            Self::Red => 0,
            Self::Green => 1,
            Self::Blue => 2,
            Self::HslHue => 3,
            Self::HslSaturation => 4,
            Self::HslLightness => 5,
            Self::OklchLightness => 6,
            Self::OklchChroma => 7,
            Self::OklchHue => 8,
        }
    }
    fn id(self) -> &'static str {
        match self {
            Self::Red => "srgb-red",
            Self::Green => "srgb-green",
            Self::Blue => "srgb-blue",
            Self::HslHue => "hsl-hue",
            Self::HslSaturation => "hsl-saturation",
            Self::HslLightness => "hsl-lightness",
            Self::OklchLightness => "oklch-lightness",
            Self::OklchChroma => "oklch-chroma",
            Self::OklchHue => "oklch-hue",
        }
    }
}
fn hue_channel(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0. {
        t += 1.
    };
    if t > 1. {
        t -= 1.
    };
    if t < 1. / 6. {
        p + (q - p) * 6. * t
    } else if t < 0.5 {
        q
    } else if t < 2. / 3. {
        p + (q - p) * (2. / 3. - t) * 6.
    } else {
        p
    }
}
fn linear(v: f64) -> f64 {
    if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
}
fn srgb(v: f64) -> f64 {
    if v <= 0.0031308 { 12.92 * v } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 }
}

fn wheel_hue(angle_radians: f64) -> f64 {
    (angle_radians.to_degrees() + 90.0).rem_euclid(360.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wheel_hue_matches_pointer_and_marker_orientation() {
        assert!((wheel_hue(-std::f64::consts::FRAC_PI_2) - 0.0).abs() < 1e-9);
        assert!((wheel_hue(0.0) - 90.0).abs() < 1e-9);
        assert!((wheel_hue(std::f64::consts::FRAC_PI_2) - 180.0).abs() < 1e-9);
    }
    #[test]
    fn srgb_round_trip_is_exact_for_bytes() {
        for c in [
            Srgb { red: 255, green: 0, blue: 0 },
            Srgb { red: 19, green: 87, blue: 221 },
            Srgb { red: 0, green: 0, blue: 0 },
        ] {
            assert_eq!(Colour::from_srgb(c).to_srgb(), c);
        }
    }
    #[test]
    fn hsl_round_trip_preserves_primary_and_gray() {
        for c in [
            Srgb { red: 255, green: 0, blue: 0 },
            Srgb { red: 0, green: 255, blue: 0 },
            Srgb { red: 102, green: 102, blue: 102 },
        ] {
            let out = Colour::from_hsl(Colour::from_srgb(c).to_hsl()).to_srgb();
            assert!((out.red as i16 - c.red as i16).abs() <= 1);
            assert!((out.green as i16 - c.green as i16).abs() <= 1);
            assert!((out.blue as i16 - c.blue as i16).abs() <= 1);
        }
    }
    #[test]
    fn oklch_round_trip_stays_within_one_byte_in_gamut() {
        for c in [
            Srgb { red: 255, green: 0, blue: 0 },
            Srgb { red: 20, green: 100, blue: 220 },
            Srgb { red: 128, green: 128, blue: 128 },
        ] {
            let out = Colour::from_oklch(Colour::from_srgb(c).to_oklch()).to_srgb();
            assert!((out.red as i16 - c.red as i16).abs() <= 1);
            assert!((out.green as i16 - c.green as i16).abs() <= 1);
            assert!((out.blue as i16 - c.blue as i16).abs() <= 1);
        }
    }

    #[test]
    fn out_of_srgb_oklch_is_clipped_per_channel_and_read_back_from_rgb() {
        let requested = Oklch { lightness: 0.6, chroma: 0.5, hue: 40.0 };
        let clipped = Colour::from_oklch(requested);
        assert_eq!(clipped.to_srgb(), Srgb { red: 255, green: 0, blue: 0 });
        let read_back = clipped.to_oklch();
        assert!((read_back.lightness - requested.lightness).abs() > 0.01);
        assert!((read_back.chroma - requested.chroma).abs() > 0.1);
        assert!((read_back.hue - requested.hue).abs() > 1.0);
    }
}

#[cfg(test)]
mod gpui_tests {
    use super::*;
    use gpui_pre::{Modifiers, TestAppContext};

    #[gpui_pre::test]
    fn numeric_entry_accepts_digits_and_emits_colour_proposal(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        let (picker, visual) =
            cx.add_window_view(|_, _| ColourTools::new("Colour", Colour::default()));
        let changes = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let log = changes.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&picker, move |_, event: &ColourChanged, _| {
                log.borrow_mut().push(event.0)
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| {
            let focus = picker.read(cx).numeric_focus[0].clone().unwrap();
            focus.focus(window, cx);
        });
        visual.simulate_keystrokes("enter");
        assert_eq!(
            picker.read_with(visual, |picker, _| picker.numeric_target),
            Some(NumericField::Red)
        );
        visual.simulate_keystrokes("2 0 0 enter");
        let red = picker.read_with(visual, |picker, _| picker.colour.to_srgb().red);
        assert_eq!(red, 200);
        assert_eq!(changes.borrow().last().unwrap().to_srgb().red, 200);
    }

    #[gpui_pre::test]
    fn grading_wheel_pointer_and_keyboard_change_selected_zone(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (picker, visual) =
            cx.add_window_view(|_, _| ColourTools::new("Colour", Colour::default()));
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let bounds = picker.read_with(visual, |picker, _| picker.grade_bounds[0]).unwrap();
        let point = point(bounds.origin.x + bounds.size.width - px(5.), bounds.center().y);
        visual.simulate_mouse_down(point, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_up(point, MouseButton::Left, Modifiers::default());
        assert!(picker.read_with(visual, |picker, _| picker.grading.shadows.saturation) > 0.5);
        let before = picker.read_with(visual, |picker, _| picker.grading.shadows.hue);
        visual.update(|window, cx| {
            let focus = picker.read(cx).grade_focus[0].clone().unwrap();
            focus.focus(window, cx);
        });
        visual.simulate_keystrokes("up");
        let after = picker.read_with(visual, |picker, _| picker.grading.shadows.hue);
        assert_ne!(before, after);
        assert_eq!(
            picker.read_with(visual, |picker, _| picker.grading.midtones),
            GradeOffset::default()
        );
    }

    #[gpui_pre::test]
    fn controlled_numeric_input_proposes_until_owner_echo_and_equal_input_is_noop(
        cx: &mut TestAppContext,
    ) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        let original = Colour::default();
        let (picker, visual) =
            cx.add_window_view(|_, _| ColourTools::controlled("Colour", original));
        let proposals = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let observed = proposals.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&picker, move |_, event: &ColourChanged, _| {
                observed.borrow_mut().push(event.0);
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| {
            let focus = picker.read(cx).numeric_focus[0].as_ref().unwrap().clone();
            focus.focus(window, cx);
        });
        visual.simulate_keystrokes("enter 2 0 0 enter");

        let proposal = *proposals.borrow().last().expect("typed colour proposal");
        assert_eq!(proposal.to_srgb().red, 200);
        assert_eq!(picker.read_with(visual, |picker, _| picker.colour), original);
        let proposal_count = proposals.borrow().len();
        picker.update(visual, |picker, cx| picker.set_colour(proposal, cx));
        assert_eq!(picker.read_with(visual, |picker, _| picker.colour), proposal);
        picker.update(visual, |picker, cx| picker.set_srgb(proposal.to_srgb(), cx));
        assert_eq!(
            proposals.borrow().len(),
            proposal_count,
            "equal owner echo is not a new proposal"
        );
    }

    #[gpui_pre::test]
    fn controlled_grading_keyboard_proposes_then_owner_echoes_and_equal_set_is_noop(
        cx: &mut TestAppContext,
    ) {
        cx.update(mkit_core::theme::set_shadcn_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (picker, visual) =
            cx.add_window_view(|_, _| ColourTools::controlled("Colour", Colour::default()));
        let proposals = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let observed = proposals.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&picker, move |_, event: &GradingChanged, _| {
                observed.borrow_mut().push(event.0);
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.update(|window, cx| {
            let focus = picker.read(cx).grade_focus[0].as_ref().unwrap().clone();
            focus.focus(window, cx);
        });
        visual.simulate_keystrokes("up");

        let requested = *proposals.borrow().last().expect("grading proposal");
        assert_eq!(requested.shadows, GradeOffset { hue: 1.0, saturation: 0.0 });
        assert_eq!(picker.read_with(visual, |picker, _| picker.grading), Grading::default());
        let proposal_count = proposals.borrow().len();
        picker.update(visual, |picker, cx| picker.set_grading(requested, cx));
        assert_eq!(picker.read_with(visual, |picker, _| picker.grading), requested);
        picker.update(visual, |picker, cx| {
            picker.set_grade(GradeZone::Shadows, requested.shadows, cx)
        });
        assert_eq!(
            proposals.borrow().len(),
            proposal_count,
            "equal grading echo is not a new proposal"
        );

        visual.simulate_keystrokes("up");
        assert_eq!(proposals.borrow().last().unwrap().shadows.hue, 2.0);
        assert_eq!(picker.read_with(visual, |picker, _| picker.grading), requested);
    }
}
