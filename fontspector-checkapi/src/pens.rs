use skrifa::outline::OutlinePen;

#[derive(Debug, Default)]
/// A pen for determining the delta between the highest and lowest points in an outline
pub struct XDeltaPen {
    /// The highest point in the outline
    highest_point: Option<(f32, f32)>,
    /// The lowest point in the outline
    lowest_point: Option<(f32, f32)>,
}

impl XDeltaPen {
    /// Create a new XDeltaPen
    pub fn new() -> Self {
        Self::default()
    }

    /// Given a point on the outline, update internal data
    fn update(&mut self, x: f32, y: f32) {
        if let Some((_hx, hy)) = self.highest_point {
            if y > hy {
                self.highest_point = Some((x, y));
            }
        } else {
            self.highest_point = Some((x, y));
        }
        if let Some((_lx, ly)) = self.lowest_point {
            if y < ly {
                self.lowest_point = Some((x, y));
            }
        } else {
            self.lowest_point = Some((x, y));
        }
    }

    /// Horizontal delta between the highest and lowest points
    pub fn x_delta(&self) -> f32 {
        if let (Some((hx, _)), Some((lx, _))) = (self.highest_point, self.lowest_point) {
            hx - lx
        } else {
            0.0
        }
    }
}

impl OutlinePen for XDeltaPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.update(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.update(x, y);
    }
    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.update(cx0, cy0);
        self.update(x, y);
    }
    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.update(cx0, cy0);
        self.update(cx1, cy1);
        self.update(x, y);
    }
    fn close(&mut self) {}
}

#[derive(Default)]
/// A pen to determine if an outline has any ink
pub struct HasInkPen(bool);
impl HasInkPen {
    /// Create a new HasInkPen
    pub fn new() -> Self {
        Self::default()
    }
    /// Does the outline have any ink?
    pub fn has_ink(&self) -> bool {
        self.0
    }
}
impl OutlinePen for HasInkPen {
    fn move_to(&mut self, _x: f32, _y: f32) {}

    fn line_to(&mut self, _x: f32, _y: f32) {
        self.0 = true;
    }

    fn quad_to(&mut self, _cx0: f32, _cy0: f32, _x: f32, _y: f32) {
        self.0 = true;
    }

    fn curve_to(&mut self, _cx0: f32, _cy0: f32, _cx1: f32, _cy1: f32, _x: f32, _y: f32) {
        self.0 = true;
    }

    fn close(&mut self) {}
}

#[derive(Default)]
/// A pen to determine if an outline has any contours
pub struct AnythingPen {
    /// Does the outline have anything?
    anything: bool,
}
impl AnythingPen {
    /// Create a new AnythingPen
    pub fn new() -> Self {
        Self::default()
    }
    /// Does the outline have anything?
    pub fn anything(&self) -> bool {
        self.anything
    }
}
impl OutlinePen for AnythingPen {
    fn move_to(&mut self, _x: f32, _y: f32) {}
    fn line_to(&mut self, _x: f32, _y: f32) {
        self.anything = true;
    }
    fn curve_to(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _x3: f32, _y3: f32) {
        self.anything = true;
    }
    fn quad_to(&mut self, _cx0: f32, _cy0: f32, _x: f32, _y: f32) {
        self.anything = true;
    }
    fn close(&mut self) {}
}

#[derive(Default)]
/// A pen to count the number of contours in an outline
pub struct ContourCountPen {
    /// The number of contours in the outline
    contour_count: usize,
}
impl ContourCountPen {
    /// Create a new ContourCountPen
    pub fn new() -> Self {
        Self::default()
    }
    /// The number of contours in the outline
    pub fn contour_count(&self) -> usize {
        self.contour_count
    }
}
impl OutlinePen for ContourCountPen {
    fn move_to(&mut self, _x: f32, _y: f32) {}
    fn line_to(&mut self, _x: f32, _y: f32) {}
    fn curve_to(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _x3: f32, _y3: f32) {}
    fn quad_to(&mut self, _cx0: f32, _cy0: f32, _x: f32, _y: f32) {}
    fn close(&mut self) {
        self.contour_count += 1;
    }
}

#[derive(Default)]
/// A pen to determine the area of an outline
pub struct AreaPen {
    /// Result area
    area: f32,
    /// Start point of the current curve
    start_point: Option<(f32, f32)>,
    /// On-curve point of the current segment
    p0: Option<(f32, f32)>,
}

impl AreaPen {
    /// Create a new AreaPen
    pub fn new() -> Self {
        Self::default()
    }
    /// The area of the outline
    pub fn area(&self) -> f32 {
        self.area
    }
}

impl OutlinePen for AreaPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.p0 = Some((x, y));
        self.start_point = Some((x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        if let Some((x0, y0)) = self.p0 {
            self.area -= (x - x0) * (y + y0) * 0.5;
        }
        self.p0 = Some((x, y));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        if let Some((x0, y0)) = self.p0 {
            let (x1, y1) = (cx0 - x0, cy0 - y0);
            let (x2, y2) = (x - x0, y - y0);
            self.area -= (x2 * y1 - x1 * y2) / 3.0;
        }
        self.line_to(x, y);
        self.p0 = Some((x, y));
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        if let Some((x0, y0)) = self.p0 {
            let (x1, y1) = (cx0 - x0, cy0 - y0);
            let (x2, y2) = (cx1 - x0, cy1 - y0);
            let (x3, y3) = (x - x0, y - y0);
            self.area -= (x1 * (-y2 - y3) + x2 * (y1 - 2.0 * y3) + x3 * (y1 + 2.0 * y2)) * 0.15;
        }
        self.line_to(x, y);
        self.p0 = Some((x, y));
    }

    fn close(&mut self) {
        if let Some((x, y)) = self.start_point {
            self.line_to(x, y);
        }
    }
}

#[cfg(feature = "kurbo")]
use kurbo::BezPath;

#[derive(Default, Debug)]
#[cfg(feature = "kurbo")]
/// A pen for converting an outline to a series of Kurbo paths
pub struct BezGlyph(pub Vec<BezPath>);

#[cfg(feature = "kurbo")]
impl BezGlyph {
    /// Create a new BezGlyph if we know the paths already
    pub fn new_from_paths(b: Vec<BezPath>) -> Self {
        BezGlyph(b)
    }
    /// Add a new, empty path to the glyph
    fn next(&mut self) -> &mut BezPath {
        self.0.push(BezPath::new());
        #[allow(clippy::unwrap_used)] // We just added it
        self.0.last_mut().unwrap()
    }
    /// Get the current path, in preparation for adding segments to it
    fn current(&mut self) -> &mut BezPath {
        if self.0.is_empty() {
            self.0.push(BezPath::new());
        }
        #[allow(clippy::unwrap_used)] // We know it's not empty
        self.0.last_mut().unwrap()
    }

    /// Iterate over the paths in the glyph
    pub fn iter(&self) -> impl Iterator<Item = &BezPath> {
        self.0.iter()
    }
}

impl skrifa::outline::OutlinePen for BezGlyph {
    fn move_to(&mut self, x: f32, y: f32) {
        self.next().move_to((x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.current().line_to((x, y));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.current().quad_to((cx0, cy0), (x, y));
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.current().curve_to((cx0, cy0), (cx1, cy1), (x, y));
    }

    fn close(&mut self) {
        self.current().close_path();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_area1() {
        let mut pen = AreaPen::new();
        pen.move_to(254.0, 360.0);
        pen.line_to(771.0, 367.0);
        pen.curve_to(800.0, 393.0, 808.0, 399.0, 819.0, 412.0);
        pen.curve_to(818.0, 388.0, 774.0, 138.0, 489.0, 145.0);
        pen.curve_to(188.0, 145.0, 200.0, 398.0, 200.0, 421.0);
        pen.curve_to(209.0, 409.0, 220.0, 394.0, 254.0, 360.0);
        pen.close();
        assert_eq!(-104561.0, pen.area().round());
    }
}
