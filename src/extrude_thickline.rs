use picture::*;
use std::num::Float;


pub fn extrude_thickline(line: &Points, thickness: f32) -> Points {
	let len = line.len();
	let mut pts = Vec::with_capacity(len * 2);

	for i in 0..len {
		let normal = if i == 0 {get_normal(line[i], line[i+1])}
					 else if i == len-1 {get_normal(line[i-1], line[i])}
					 else {get_miter(line[i-1], line[i], line[i+1])};

		let extrusion = mul(normal, thickness / 2.0);
		pts.push(add(line[i], extrusion));
		pts.push(sub(line[i], extrusion));
	}

	pts
}


fn get_normal(prev: Point, cur: Point) -> Point {
	let tangent = normalize(sub(prev, cur));
	point(-tangent.y, tangent.x)
}

fn get_miter(prev: Point, cur: Point, next: Point) -> Point {
	let tangent1 = normalize(sub(prev, cur));
	let tangent2 = normalize(sub(next, cur));
	mul( add(tangent1, tangent2), 1.0 / (tangent1.x * tangent2.y - tangent1.y * tangent2.x) )
}


fn normalize(vector: Point) -> Point {
    match vector {
        Point {x: 0.0, y: 0.0} => vector,
        Point {x, y} => mul(vector, (x * x + y * y).rsqrt())
    }
}

fn mul(vector: Point, scalar: f32) -> Point {
    point(vector.x * scalar, vector.y * scalar)
}

fn add(vector1: Point, vector2: Point) -> Point {
    point(vector1.x + vector2.x, vector1.y + vector2.y)
}

fn sub(vector1: Point, vector2: Point) -> Point {
    point(vector1.x - vector2.x, vector1.y - vector2.y)
}
