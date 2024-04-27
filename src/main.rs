use std::{f32::consts::PI, thread, time::Duration};

use rand::prelude::*;

type Coordinate = (usize, usize);
type Size = (usize, usize);
type Point = (f32, f32);

#[derive(Copy, Clone, Debug)]
struct Line(Point, Point);

impl Line {
    pub fn intersect(self, other: Self) -> Option<Point> {
        let (x1, y1) = self.0;
        let (x2, y2) = self.1;
        let (x3, y3) = other.0;
        let (x4, y4) = other.1;

        let denominator = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
        if denominator == 0.0 {
            return None; // Lines are parallel
        }

        let t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denominator;
        let u = -((x1 - x2) * (y1 - y3) - (y1 - y2) * (x1 - x3)) / denominator;

        if t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0 {
            Some((x1 + t * (x2 - x1), y1 + t * (y2 - y1)))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
struct Velocity {
    direction: f32, // radians
    distance: f32,
}

struct Node {
    position: Point,
    velocity: Velocity,
}

impl Node {
    fn update_position(&mut self) {
        (self.position, self.velocity) = get_new_position(self.position, self.velocity);
    }
}

fn main() {
    let mut rng = rand::thread_rng();

    let mut nodes: Vec<Node> = (0..100)
        .map(|_| Node {
            position: (
                rng.gen::<f32>() * PLAY_AREA_SIZE.0 as f32,
                rng.gen::<f32>() * PLAY_AREA_SIZE.1 as f32,
            ),
            velocity: Velocity {
                direction: rng.gen::<f32>() * 2.0 * PI,
                distance: rng.gen::<f32>(),
            },
        })
        .collect();
    for _ in 0..100000 {
        nodes.iter_mut().for_each(Node::update_position);
        render(&nodes);
        thread::sleep(Duration::from_millis(15));
    }
}

const BOARD_RESOLUTION: Size = (20, 100);
const PLAY_AREA_SIZE: Size = (20, 100);
const DAMPENING: f32 = 0.8;

fn render(nodes: &Vec<Node>) {
    let mut board = [['.'; BOARD_RESOLUTION.1]; BOARD_RESOLUTION.0];
    for node in nodes {
        let coord = point_to_board_coord(node.position, BOARD_RESOLUTION, PLAY_AREA_SIZE);
        board[coord.0][coord.1] = 'O';
    }
    let board: String = board.map(|col| col.iter().collect::<String>()).join("\n");
    print!("{}[2J", 27 as char);
    println!("{}", board);
}

const RIGHT_WALL: Line = Line(
    (PLAY_AREA_SIZE.0 as f32, 0.0),
    (PLAY_AREA_SIZE.0 as f32, PLAY_AREA_SIZE.1 as f32),
);
const LEFT_WALL: Line = Line((0.0, 0.0), (0.0, PLAY_AREA_SIZE.1 as f32));
const TOP_WALL: Line = Line((0.0, 0.0), (PLAY_AREA_SIZE.0 as f32, 0.0));
const BOTTOM_WALL: Line = Line(
    (0.0, PLAY_AREA_SIZE.1 as f32),
    (PLAY_AREA_SIZE.0 as f32, PLAY_AREA_SIZE.1 as f32),
);

fn get_new_position(position: Point, velocity: Velocity) -> (Point, Velocity) {
    let mut point_0 = position.0 + velocity.distance * velocity.direction.cos();
    let mut point_1 = position.1 + velocity.distance * velocity.direction.sin();
    let mut new_velocity = velocity.clone();

    let mut x_contact: Option<Line> = None;
    let mut y_contact: Option<Line> = None;

    if point_0 < 0.0 {
        x_contact = Some(LEFT_WALL);
    } else if point_0 >= PLAY_AREA_SIZE.0 as f32 {
        x_contact = Some(RIGHT_WALL);
    }
    if point_1 < 0.0 {
        y_contact = Some(TOP_WALL);
    } else if point_1 >= PLAY_AREA_SIZE.1 as f32 {
        y_contact = Some(BOTTOM_WALL);
    }

    let mut intersections: Vec<(Point, bool)> = Vec::with_capacity(2);
    let traveled_line = Line(position, (point_0, point_1));
    if let Some(x_line) = x_contact {
        if let Some(intersect) = traveled_line.intersect(x_line) {
            intersections.push((intersect, true));
        }
    }
    if let Some(y_line) = y_contact {
        if let Some(intersect) = traveled_line.intersect(y_line) {
            intersections.push((intersect, false));
        }
    }

    // intersected a wall and need to calculate new velocity and position
    if let Some(((intersect_point, x_intersect), distance)) = intersections
        .into_iter()
        .map(|p| (p, calc_distance(position, p.0)))
        .min_by_key(|p| (p.1 * 10_000.0) as u32)
    {
        let mut multiplier = 2.0;
        if x_intersect {
            multiplier = 1.0;
        }
        let (new_point, velocity) = get_new_position(
            intersect_point,
            Velocity {
                distance: velocity.distance - distance,
                direction: multiplier * PI - velocity.direction,
            },
        );
        new_velocity.direction = velocity.direction;
        point_0 = new_point.0;
        point_1 = new_point.1;
    }

    ((point_0, point_1), new_velocity)
}

fn calc_distance(p1: Point, p2: Point) -> f32 {
    let dx = p2.0 - p1.0;
    let dy = p2.1 - p1.1;
    (dx.powi(2) + dy.powi(2)).sqrt()
}

fn point_to_board_coord(point: Point, resolution: Size, play_area: Size) -> Coordinate {
    let point_0 = point.0 / play_area.0 as f32 * resolution.0 as f32;
    let point_1 = point.1 / play_area.1 as f32 * resolution.1 as f32;
    (point_0 as usize, point_1 as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_intersection() {
        let line1 = Line((0.0, 1.0), (2.0, 1.0));
        let line2 = Line((1.0, 0.0), (1.0, 2.0));
        let intersect = line1.intersect(line2);
        assert_eq!(Some((1.0, 1.0)), intersect);

        let line1 = Line((0.0, 1.0), (2.0, 1.0));
        let line2 = Line((0.0, 0.0), (2.0, 2.0));
        let intersect = line1.intersect(line2);
        assert_eq!(Some((1.0, 1.0)), intersect);

        let line1 = Line((0.0, 1.0), (2.0, 1.0));
        let line2 = Line((3.0, 0.0), (3.0, 2.0));
        let intersect = line1.intersect(line2);
        assert_eq!(None, intersect);
    }
}
