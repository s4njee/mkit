//! Read-only histogram visualization for image and signal editing tools.
extern crate gpui_pre as gpui;

use gpui::{App, IntoElement, RenderOnce, Rgba, Window, div, point, prelude::*, px, relative};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::{ShadowToken, Theme},
};

/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight, ..foreground }, Rgba { a: 1.0, ..base })
}
/// The card border and plot baseline (the shadcn "border" role); see the spec's theme table.
fn border_role(t: &Theme) -> Rgba {
    let c = t.colors;
    if t.name != "high-contrast" && relative_luminance(c.background) < 0.5 {
        mix(c.text, c.background, 0.1)
    } else {
        c.border
    }
}
fn box_shadow(shadow: ShadowToken) -> gpui::BoxShadow {
    gpui::BoxShadow {
        color: shadow.color.into(),
        offset: point(px(shadow.x), px(shadow.y)),
        blur_radius: px(shadow.blur),
        spread_radius: px(shadow.spread),
        inset: false,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Series {
    label: String,
    values: Vec<f32>,
    color: SeriesColor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SeriesColor {
    Accent,
    Success,
    Warning,
    Danger,
}

/// Stateless image-value distribution chart.
#[derive(IntoElement)]
pub struct Histogram {
    label: String,
    summary: String,
    series: Vec<Series>,
    shadows_clipped: bool,
    highlights_clipped: bool,
    height: f32,
}

impl Histogram {
    /// Create an empty histogram with an accessible name and textual summary.
    pub fn new(label: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            summary: summary.into(),
            series: Vec::new(),
            shadows_clipped: false,
            highlights_clipped: false,
            height: 78.0,
        }
    }

    /// Replace the visualization with one luminance distribution.
    pub fn luminance(mut self, values: impl Into<Vec<f32>>) -> Self {
        self.series = vec![Series {
            label: "Luminance".into(),
            values: values.into(),
            color: SeriesColor::Accent,
        }];
        self
    }

    /// Replace the visualization with red, green, and blue distributions.
    pub fn rgb(
        mut self,
        red: impl Into<Vec<f32>>,
        green: impl Into<Vec<f32>>,
        blue: impl Into<Vec<f32>>,
    ) -> Self {
        self.series = vec![
            Series { label: "Red".into(), values: red.into(), color: SeriesColor::Danger },
            Series { label: "Green".into(), values: green.into(), color: SeriesColor::Success },
            Series { label: "Blue".into(), values: blue.into(), color: SeriesColor::Warning },
        ];
        self
    }

    /// Set shadow and highlight clipping indicators.
    pub fn clipping(mut self, shadows: bool, highlights: bool) -> Self {
        self.shadows_clipped = shadows;
        self.highlights_clipped = highlights;
        self
    }

    /// Set chart plot height in logical pixels. Defaults to 78px to match the established Laika
    /// histogram footprint; hosts may choose a different size for their layout.
    pub fn height(mut self, height: f32) -> Self {
        if height.is_finite() && height > 0.0 {
            self.height = height;
        }
        self
    }
}

fn safe_bin(value: f32) -> f32 {
    if value.is_finite() { value.clamp(0.0, 1.0) } else { 0.0 }
}

impl RenderOnce for Histogram {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let line = border_role(&theme);
        let mut panel = div()
            .id("mkit-histogram")
            .role(gpui::accesskit::Role::Image)
            .aria_label(self.label)
            .aria_description(self.summary)
            .w_full()
            .flex()
            .flex_col()
            .gap(px(theme.spacing.xsmall))
            .p(px(theme.spacing.small))
            .rounded(px(theme.radii.large))
            .border(px(theme.borders.hairline))
            .border_color(line)
            .bg(theme.colors.background)
            .shadow(vec![box_shadow(theme.shadows.small)]);

        if self.shadows_clipped || self.highlights_clipped {
            let indicator = |color: Rgba, label: &'static str| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(theme.spacing.xsmall))
                    .child(
                        div()
                            .flex_none()
                            .size(px(theme.spacing.small))
                            .rounded(px(theme.radii.pill))
                            .bg(color),
                    )
                    .child(
                        div()
                            .text_color(theme.colors.text)
                            .text_size(px(theme.typography.caption))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(label),
                    )
            };
            let mut status = div().flex().items_center().gap(px(theme.spacing.medium));
            if self.shadows_clipped {
                status = status.child(indicator(theme.colors.warning, "Shadows clipped"));
            }
            if self.highlights_clipped {
                status = status.child(indicator(theme.colors.danger, "Highlights clipped"));
            }
            panel = panel.child(status);
        }

        let chart_height = if self.series.len() > 1 {
            (self.height - theme.spacing.small * 2.0).max(1.0) / self.series.len() as f32
        } else {
            self.height
        };
        if self.series.is_empty() {
            panel = panel.child(div().h(px(self.height)).w_full());
        }
        for series in self.series {
            let color = match series.color {
                SeriesColor::Accent => theme.colors.accent,
                SeriesColor::Success => theme.colors.success,
                SeriesColor::Warning => theme.colors.warning,
                SeriesColor::Danger => theme.colors.danger,
            };
            let max = series.values.iter().copied().map(safe_bin).fold(0.0_f32, f32::max);
            let mut bars = div()
                .h(px(chart_height))
                .w_full()
                .flex()
                .items_end()
                .gap(px(theme.borders.hairline))
                .border_b(px(theme.borders.hairline))
                .border_color(line)
                .when(series.values.is_empty(), |el| el.items_center());
            for value in series.values {
                let normalized = if max > 0.0 { safe_bin(value) / max } else { 0.0 };
                bars = bars.child(div().flex_1().h(relative(normalized)).bg(color));
            }
            let row = div()
                .flex()
                .items_center()
                .gap(px(theme.spacing.small))
                .child(
                    div()
                        .min_w(px(theme.typography.caption * 6.5))
                        .text_size(px(theme.typography.caption))
                        .text_color(theme.colors.text_muted)
                        .child(series.label),
                )
                .child(bars);
            panel = panel.child(row);
        }
        panel
    }
}

#[cfg(test)]
mod tests {
    use super::safe_bin;

    #[test]
    fn bins_are_finite_and_clamped() {
        assert_eq!(safe_bin(f32::NAN), 0.0);
        assert_eq!(safe_bin(f32::INFINITY), 0.0);
        assert_eq!(safe_bin(-1.0), 0.0);
        assert_eq!(safe_bin(0.4), 0.4);
        assert_eq!(safe_bin(2.0), 1.0);
    }
}
