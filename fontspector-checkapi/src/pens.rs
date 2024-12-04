use skrifa::outline::OutlinePen;

#[derive(Debug, Default)]
pub struct XDeltaPen {
    highest_point: Option<(f32, f32)>,
    lowest_point: Option<(f32, f32)>,
}

impl XDeltaPen {
    pub fn new() -> Self {
        Self::default()
    }
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
pub struct HasInkPen(bool);
impl HasInkPen {
    pub fn new() -> Self {
        Self::default()
    }
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
pub struct AnythingPen {
    anything: bool,
}
impl AnythingPen {
    pub fn new() -> Self {
        Self::default()
    }
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
pub struct ContourCountPen {
    contour_count: usize,
}
impl ContourCountPen {
    pub fn new() -> Self {
        Self::default()
    }
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
pub struct AreaPen {
    area: f32,
    start_point: Option<(f32, f32)>,
    p0: Option<(f32, f32)>,
}

impl AreaPen {
    pub fn new() -> Self {
        Self::default()
    }
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
