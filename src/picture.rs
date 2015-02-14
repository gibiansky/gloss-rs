pub enum Picture {
    /// A blank picture, with nothing in it.
    Blank,

    /// A convex polygon filled with a solid color.
    Polygon(Points),

    /// A line along an arbitrary path.
    Line(Points),

    /// A circle with the given radius.
    Circle(f32),

    /// A circle with the given thickness and radius. If the thickness is 0 then this is equivalent to Circle.
    ThickCircle(f32, f32),

    /// A circular arc drawn counter-clockwise between two angles (in degrees) at the given radius.
    Arc(f32, f32, f32),

    /// A circular arc drawn counter-clockwise between two angles (in degrees), with the given
    /// radius and thickness. If the thickness is 0 then this is equivalent to Arc.
    ThickArc(f32, f32, f32, f32),

    /// Some text to draw with a vector font.
    Text(String),

    /// A bitmap image with a width, height and some 32-bit RGBA bitmap data.
    ///
    /// The boolean flag controls whether Gloss should cache the data between frames for speed. If
    /// you are programatically generating the image for each frame then use False. If you have
    /// loaded it from a file then use True.
    Bitmap(u32, u32, BitmapData, bool),

    /// A picture drawn with this color.
    Colored(Color, Box<Picture>),

    /// A picture translated by the given x and y coordinates.
    Translate(f32, f32, Box<Picture>),

    /// A picture rotated clockwise by the given angle (in degrees).
    Rotate(f32, Box<Picture>),

    /// A picture scaled by the given x and y factors.
    Scale(f32, f32, Box<Picture>),

    /// A picture consisting of several others.
    Pictures(Vec<Picture>),
}

/// A color used for drawing pictures.
pub enum Color {
    Black,
    Blue,
    Green,
    Red,
    White,
    RGB(f32, f32, f32),
    RGBA(f32, f32, f32, f32),
}
impl Copy for Color {}

/// A point in 2D space.
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// A list of points.
pub type Points = Vec<Point>;

/// UNDEFINED. FIXME
pub type BitmapData = bool;

pub fn point(x: f32, y: f32) -> Point {
    Point { x: x, y: y }
}

pub fn color_to_rgba(color: Color) -> (f32, f32, f32, f32) {
    match color {
        Color::RGBA(r, g, b, a) => (r, g, b, a),
        Color::RGB(r, g, b)     => (r, g, b, 1.0),
        Color::Black            => (0.0, 0.0, 0.0, 1.0),
        Color::Blue             => (0.0, 0.0, 1.0, 1.0),
        Color::Green            => (0.0, 1.0, 0.0, 1.0),
        Color::Red              => (1.0, 0.0, 0.0, 1.0),
        Color::White            => (1.0, 1.0, 1.0, 1.0),
    }
}
