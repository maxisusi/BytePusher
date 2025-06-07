use std::ops::Sub;

use raylib::prelude::*;

/** Vertex Type alias */
type Vt = i32;

#[derive(Copy, Clone)]
struct Vertex {
    x: Vt,
    y: Vt,
}

impl Vertex {
    fn sub(a: Vertex, b: Vertex) -> Vertex {
        Vertex {
            x: b.x - a.x,
            y: b.y - a.y,
        }
    }
    fn cross_product(&self, start_vertex: Vertex, target: Vertex) -> i32 {
        let target = Vertex::sub(start_vertex, target);
        self.x * target.y - self.y * target.x
    }
}

impl Sub for Vertex {
    type Output = Vertex;
    fn sub(self, b: Self) -> Self::Output {
        Vertex {
            x: b.x - self.x,
            y: b.y - self.y,
        }
    }
}

struct Triangle {
    v0: Vertex,
    v1: Vertex,
    v2: Vertex,
    color: Option<Color>,
}

struct BoundingBox {
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
}

impl Triangle {
    fn new(v1: Vertex, v2: Vertex, v3: Vertex, color: Option<Color>) -> Self {
        Self {
            v0: v1,
            v1: v2,
            v2: v3,
            color,
        }
    }

    fn get_bounding_box(&self) -> BoundingBox {
        BoundingBox {
            min_x: i32::min(self.v0.x, i32::min(self.v1.x, self.v2.x)),
            min_y: i32::min(self.v0.y, i32::min(self.v1.y, self.v2.y)),
            max_x: i32::max(self.v0.x, i32::max(self.v1.x, self.v2.x)),
            max_y: i32::max(self.v0.y, i32::max(self.v1.y, self.v2.y)),
        }
    }

    pub fn buffer(&self) -> Vec<(i32, i32)> {
        let bb = self.get_bounding_box();
        let mut buffer: Vec<(i32, i32)> = vec![];

        for x in bb.min_x..bb.max_x {
            for y in bb.min_y..bb.max_y {
                let target = Vertex { x, y };

                let v0 = (self.v0 - self.v1).cross_product(self.v0, target);
                let v1 = (self.v1 - self.v2).cross_product(self.v1, target);
                let v2 = (self.v2 - self.v0).cross_product(self.v2, target);

                if v0 >= 0 && v1 >= 0 && v2 >= 0 {
                    buffer.push((target.x, target.y));
                }
            }
        }
        buffer
    }
}

fn main() {
    let tr_1 = Triangle::new(
        Vertex { x: 320, y: 100 },
        Vertex { x: 400, y: 200 },
        Vertex { x: 250, y: 250 },
        Some(Color::RED),
    );

    let tr_2 = Triangle::new(
        Vertex { x: 100, y: 300 },
        Vertex { x: 200, y: 400 },
        Vertex { x: 50, y: 450 },
        Some(Color::PURPLE),
    );

    let triangles = vec![tr_1, tr_2];

    let (mut rl, thread) = raylib::init().size(640, 480).title("Rasterizer").build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);

        for triangle in triangles.iter() {
            for v in triangle.buffer() {
                d.draw_pixel(v.0, v.1, triangle.color.unwrap_or(Color::WHITE));
            }
        }
    }
}
