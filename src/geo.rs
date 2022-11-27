/// Utils and objects for geography and coordinate work
#[allow(unused_imports)]
use num_traits::float::Float;

/// A 2-dimensional point with x and y coordinates
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x: x, y: y }
    }

    /// Find the bearing from this point to the given point
    pub fn bearing(&self, point: Point2D) -> f32 {
        let lng_a = self.x.to_radians();
        let lat_a = self.y.to_radians();
        let lng_b = point.x.to_radians();
        let lat_b = point.y.to_radians();
        let delta_lng = lng_b - lng_a;

        let s = lat_b.cos() * delta_lng.sin();
        let c = lat_a.cos() * lat_b.sin() - lat_a.sin() * lat_b.cos() * delta_lng.cos();

        f32::atan2(s, c).to_degrees()
    }

    /// Find the bearing from this point to north
    pub fn bearing_north(&self) -> f32 {
        f32::atan2(self.x, self.y).to_degrees()
    }
}
