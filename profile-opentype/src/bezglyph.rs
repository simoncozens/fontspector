#![allow(dead_code)]
use kurbo::BezPath;
#[derive(Default, Debug)]
pub struct BezGlyph(pub Vec<BezPath>);

impl BezGlyph {
    pub fn new_from_paths(b: Vec<BezPath>) -> Self {
        BezGlyph(b)
    }
    pub fn next(&mut self) -> &mut BezPath {
        self.0.push(BezPath::new());
        #[allow(clippy::unwrap_used)] // We just added it
        self.0.last_mut().unwrap()
    }
    pub fn current(&mut self) -> &mut BezPath {
        if self.0.is_empty() {
            self.0.push(BezPath::new());
        }
        #[allow(clippy::unwrap_used)] // We know it's not empty
        self.0.last_mut().unwrap()
    }

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
