use std::collections::HashMap;

use super::common::{FuzzyError, FuzzyResult, Term};

#[derive(Debug, Clone, PartialEq)]
pub struct FuzzySet {
    terms: HashMap<Term, Vec<(f64, f64)>>
}

impl FuzzySet {
    pub fn new(
    ) -> Self {
        Self {
            terms: HashMap::new()
        }
    }

    pub fn term(
        mut self,
        key: impl Into<Term>,
        mut points: Vec<(f64, f64)>
    ) -> FuzzyResult<Self> {
        if points.len() < 2 {
            Err(FuzzyError::InvalidPoints)?
        }
        // Order points by x axis.
        points.sort_by(|(x1, _), (x2, _)| x1.partial_cmp(x2).unwrap());
        self.terms.insert(key.into(), points);
        Ok(self)
    }

    pub fn terms(
        &self
    ) -> impl Iterator<Item=(&Term, &Vec<(f64, f64)>)> {
        self.terms.iter()
    }

    pub fn points(
        &self,
        term: impl Into<Term>
    ) -> FuzzyResult<&Vec<(f64, f64)>> {
        let key = term.into();
        self.terms.get(&key)
            .ok_or(FuzzyError::InvalidTerm(key.into()))
    }

    /// Aplies maximum threshold for given term.
    pub fn apply_threshold(
        &mut self,
        term: impl Into<Term>,
        value: f64
    ) -> FuzzyResult<()> {
        let key = term.into();
        let mut points = self.terms.remove(&key)
            .ok_or(FuzzyError::InvalidTerm(key.clone()))?;

        // Three cases:
        // 1. Threshold above maximum y -> Do nothing
        // 2. Threshold exceeded only once. -> Replace interval with one point.
        // 3. Whole set above the threshold -> Replace with two bounadry points with same y.
        // 4. Threshold exceeded and returned to valid range -> Replace interval with two points.

        // Solution idea
        // 1. Find all adjacent points above the threshold. This is going to be our interval.
        // 2. Solve for y=value(boundaries). Result: (x1, y), (x2, y), or (x1, y) for case 2).
        // 3. Replace interval with newly calculated points.

        let points_copy = points.clone();

        let find_x = |i: usize, y: f64| -> FuzzyResult<f64> {
            let p1 = points_copy.get(i).unwrap();
            let p2 = points_copy.get(i+1).unwrap_or_else(|| p1);
            println!("Y: {} | P1: {:?} | P2: {:?}", y, p1, p2);
            let (x1, y1) = p1;
            let (x2, y2) = p2;
            // Check if valid range.
            let (min_y, max_y) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
            if !(y >= *min_y && y <= *max_y) {
                panic!("Internal error. y out of bounds.")
            }
            // I assume (x, y) lies on the same line as p1 and p2, so we can find x by comparing slopes.
            let slope = if x1 == x2  { 0.0 } else { (y1-y2)/(x1-x2) };
            let x = if slope == 0.0 { *x1 } else { (y-y1)/slope+x1 };
            Ok(x)
        };

        // Do actual replacement.
        let mut replace_interval = |from: i32, to: i32| -> FuzzyResult<()> {
            println!("REPLACE: {}/{}", from, to);
            match (from, to) {
                (-1, -1) => {},
                // Whole set above threshold -> Replace whole set with two points.
                (-1, j) | (0, j) if j as usize == points_copy.len()-1 => {
                    let (x1, _) = *points.first().unwrap();
                    let (x2, _) = *points.last().unwrap();
                    points.clear();
                    points.push((x1, value));
                    points.push((x2, value));
                },
                // Right side of the set above threshold -> Replace with one point.
                (i, j) if j as usize == points_copy.len()-1 => {
                    let i = i as usize;
                    let j = j as usize;
                    let x1 = find_x(i-1, value)?;
                    points.drain(j..points_copy.len());
                    points.push((x1, value));
                },
                // Left side of the set above threshold -> Replace with one point
                (0, j) | (-1, j) => {
                    let j = j as usize;
                    let x1 = find_x(j, value)?;
                    points.drain(0..j+1);
                    points.insert(0, (x1, value));
                },
                // Middle of set above threshold -> Replace interval with two points.
                (i, j) => {
                    let (i, j) = (i as usize, j as usize);
                    let x1 = find_x(i-1, value)?;
                    let x2 = find_x(j, value)?;
                    points.drain(i..j+1);
                    points.insert(i, (x2, value));
                    points.insert(i, (x1, value));
                }
            };
            Ok(())
        };

        // Pointers to interval positions.
        let (mut int_start, mut int_end) = (-1, -1);
        // Start from the end to prevent indexing issues after element removal.
        for (i, (_x, y)) in points_copy.iter().enumerate().rev() {
            println!("{}/({}, {})", i, _x, y);
            match (int_start, int_end) {
                // No interval processed.
                (-1, -1) => {
                    if *y >= value {
                        int_end = i as i32;
                        println!("HERE 1 {} -> {}", i, int_end);
                    }
                },
                // Start of the interval
                (-1, _) => {
                    if *y < value {
                        int_start = i as i32+1;
                        println!("HERE 2 {} -> ({}, {})", i, int_start, int_end);
                        replace_interval(int_start, int_end)?;
                        int_start = -1;
                        int_end = -1;
                    }
                },
                _ =>
                    panic!("Internal error. Unexpected interval state.")
            }
        }
        replace_interval(int_start, int_end)?;

        self.terms.insert(key, points);
        Ok(())
    }

    pub fn call_single(
        &self,
        term: impl Into<String>,
        x: f64
    ) -> FuzzyResult<f64> {
        let key = term.into();
        let values = self.terms.get(&key)
            .ok_or(FuzzyError::InvalidTerm(key))?;

        let values = &values[..];
        let (mut lx, mut ly) = values.first().unwrap();
        let (mut rx, mut ry) = values.last().unwrap();
        if x < lx {
            rx = lx; ry = ly;
        } else if x > rx {
            lx = rx; ly = ry;
        } else {
            for window in values.windows(2) {
                let (px, py) = window[0];
                let (cx, cy) = window[1];
                if px <= x && cx >= x {
                    lx = px; ly = py;
                    rx = cx; ry = cy;
                    break;
                }
            }
        }
        // Linear interpolation to find y.
        let slope = if lx == rx  { 0.0 } else { (ly-ry)/(lx-rx) };
        Ok(ly+(x-lx)*slope)
    }

    pub fn call(
        &self,
        x: f64
    ) -> FuzzyResult<HashMap<String, f64>> {
        let mut result = HashMap::new();
        for (key, _) in self.terms() {
            let y = self.call_single(key.clone(), x)?;
            result.insert(key.clone(), y);
        }
        Ok(result)
    }
}


#[macro_export]
macro_rules! fuzzy {
    ($($term:expr => $(($x:expr,$y:expr)),* $(,)*);* $(;)*) => {{
        Result::<$crate::FuzzySet, $crate::FuzzyError>::Ok($crate::FuzzySet::new())
            $(.and_then(|set| set.term($term, vec![$(($x, $y),)*])))*
    }}
}

#[test]
fn test_fuzzy_macro(
) -> () {
    assert_eq!(fuzzy!{
        "term1" => (1.0, 1.0), (2.0, 2.0);
        "term2" => (0.0, 0.5)
    }, Err(FuzzyError::InvalidPoints));
    assert_eq!(fuzzy!{
        "term1" => (1.0, 1.0), (0.0, 2.0)
    }, FuzzySet::new().term("term1", vec![(0.0, 2.0), (1.0, 1.0)]));
    assert_eq!(fuzzy!{}, Ok(FuzzySet::new()))
}
