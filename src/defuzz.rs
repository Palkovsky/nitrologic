use super::common::{FuzzyError, FuzzyResult};

///              / - - -
///             /
///      - - - /
///    /
///   /
///- -
pub fn cog(
    points: impl AsRef<[(f64, f64)]>
) -> FuzzyResult<f64> {
    let points = points.as_ref();
    if points.len() < 2 {
        Err(FuzzyError::InvalidPoints)?
    }
    let (mut a, mut b) = (0.0, 0.0);
    for pair in points.windows(2) {
        let (x1, y1) = pair[0];
        let (x2, y2) = pair[1];
        let (x_diff, y_diff) = (x2-x1, (y2-y1).abs());
        // Height of base rectangle.
        let height = f64::min(y1, y2);
        // Area of base rectangle + right traingle on the top.
        let area = height*x_diff + 0.5*y_diff*x_diff;
        let centroid = if y_diff == 0.0 {
            // | - - - - |
            // |         |
            // |         |
            // |         |
            // | - - - - |
            (x1+x2)/2.0
        } else {
            // |      -  |
            // |   / |   |
            // | -   |   |
            // |  |  |   |
            // | - - - - |
            (2.0*x1+x2)/3.0
        };
        a += area*centroid;
        b += area;
    }
    Ok(a/b)
}
