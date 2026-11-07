//! The isometric projection shared by the app's drawn art: a floor seen from
//! above, turned by -28° and flattened to 64% height. It is a flat 2D transform,
//! so GTK draws it directly and edges stay sharp.

use gtk::graphene;
use gtk::prelude::*;

pub const TURN_DEGREES: f32 = -28.0;
pub const FLATTEN: f32 = 0.64;

/// Makes everything drawn after it land on the floor, until the snapshot is restored.
pub fn apply_floor_transform(snapshot: &gtk::Snapshot) {
    snapshot.scale(1.0, FLATTEN);
    snapshot.rotate(TURN_DEGREES);
}

/// Where a point on the floor ends up on screen, relative to the floor's origin.
pub fn floor_to_screen(x: f32, y: f32) -> graphene::Point {
    let angle = TURN_DEGREES.to_radians();
    let turned_x = x * angle.cos() - y * angle.sin();
    let turned_y = x * angle.sin() + y * angle.cos();
    graphene::Point::new(turned_x, turned_y * FLATTEN)
}

/// The move on the floor that shows up as `drop` pixels straight down on screen.
/// Drawing a darker copy there gives slabs and windows a visible edge.
pub fn screen_drop_in_floor(drop: f32) -> (f32, f32) {
    let unflattened_y = drop / FLATTEN;
    let angle = (-TURN_DEGREES).to_radians();
    (-unflattened_y * angle.sin(), unflattened_y * angle.cos())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_points_straight_down_on_screen() {
        let (dx, dy) = screen_drop_in_floor(10.0);
        let screen = floor_to_screen(dx, dy);

        assert!(screen.x().abs() < 0.01, "screen x was {}", screen.x());
        assert!((screen.y() - 10.0).abs() < 0.01, "screen y was {}", screen.y());
    }
}
