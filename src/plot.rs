// Plotlib
use plotlib::{
    page::Page,
    repr::Plot,
    view::{View, ContinuousView},
    style::LineStyle
};

use super::set::FuzzySet;
use super::common::{
    FuzzyError,
    FuzzyResult
};

static COLORS: &'static [&'static str] = &[
    "#ffbe0b",
    "#fb5607",
    "#ff006e",
    "#8338ec",
    "#3a86ff"
];

pub struct FuzzyPlot<V> {
    view: V
}

impl<V: View> FuzzyPlot<V> {
    pub fn new(
        view: V
    ) -> Self {
        Self { view }
    }

    pub fn to_svg(
        &self,
        path: impl AsRef<std::path::Path>
    ) -> FuzzyResult<()> {
        let path = path.as_ref();
        Page::single(&self.view)
            .save(path)
            .map_err(|err| {
                let msg = format!("Error saving to {:?} with '{:?}'.", path, err);
                FuzzyError::Misc(msg)
            })
    }

    pub fn to_string(
        &self
    ) -> FuzzyResult<String> {
        Page::single(&self.view)
            .to_text()
            .map_err(|err| FuzzyError::Misc(format!("{:?}", err)))
    }
}

pub fn set(
    set: &FuzzySet,
    x_label: impl Into<String>,
    y_label: impl Into<String>
) -> FuzzyPlot<ContinuousView> {
    let mut view = ContinuousView::new()
        .y_range(0.0, 1.0)
        .x_label(x_label)
        .y_label(y_label);

    for (i, (_, points)) in set.terms().enumerate() {
        let color = COLORS[i%COLORS.len()];
        let style = LineStyle::new().colour(color);
        let plot = Plot::new(points.to_vec()).line_style(style);
        view = view.add(plot);
    }

    FuzzyPlot::new(view)
}
