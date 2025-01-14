pub use ecolor::Color32;
use ecolor::Rgba;

pub mod operations;

pub const RED_CHANNEL: usize = 0;
pub const GREEN_CHANNEL: usize = 1;
pub const BLUE_CHANNEL: usize = 2;
pub const ALPHA_CHANNEL: usize = 3;

#[derive(Clone, Copy, Debug)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }

    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    pub fn with_alpha(&self, alpha: f32) -> Self {
        Self::new(self.r, self.g, self.b, alpha)
    }

    pub fn to_color32(&self) -> Color32 {
        let r = (self.r * self.a * 255.0) as u8;
        let g = (self.g * self.a * 255.0) as u8;
        let b = (self.b * self.a * 255.0) as u8;
        let a = (self.a * 255.0) as u8;
        Color32::from_rgba_premultiplied(r, g, b, a)
    }

    pub fn blend(&self, other: &Color) -> Self {
        let out_alpha = other.a + self.a * (1.0 - other.a);
        if out_alpha == 0.0 {
            return Self::new(0.0, 0.0, 0.0, 0.0);
        }

        let sr = other.r * other.a;
        let sg = other.g * other.a;
        let sb = other.b * other.a;

        let dr = self.r * self.a * (1.0 - other.a);
        let dg = self.g * self.a * (1.0 - other.a);
        let db = self.b * self.a * (1.0 - other.a);

        Self {
            r: (sr + dr) / out_alpha,
            g: (sg + dg) / out_alpha,
            b: (sb + db) / out_alpha,
            a: out_alpha,
        }
    }
}

/// A pixel is a single point in a pixel buffer with an RGBA color value.
pub struct Pixel {
    pub x: i32,
    pub y: i32,
    pub color: Rgba,
}

/// A stamp is a collection of pixels that represent a brush shape.
pub struct Stamp {
    pub pixels: Vec<Pixel>,
}

#[derive(Clone)]
pub struct BrushBaseSettings {
    pub id: String,
    pub radius: f32,
    pub spacing: f32,
    pub strength: f32,
}

#[derive(Clone)]
pub enum Brush {
    SoftCircle {
        inner_radius: f32,
        base: BrushBaseSettings,
    },
}

impl Default for Brush {
    fn default() -> Self {
        Brush::SoftCircle {
            inner_radius: 1.0,
            base: BrushBaseSettings {
                id: "soft-circle".to_string(),
                radius: 10.0,
                spacing: 1.0,
                strength: 1.0,
            },
        }
    }
}

impl Brush {
    /// Gets a stamp for the current brush settings
    pub fn compute_stamp(&self, color: Rgba) -> Stamp {
        match self {
            Brush::SoftCircle { inner_radius: hardness, base } => soft_circle(base.radius, *hardness, color),
        }
    }

    //==========================================================================
    // accessor methods
    //==========================================================================

    pub fn spacing(&self) -> f32 {
        match self {
            Brush::SoftCircle { base, .. } => base.spacing,
        }
    }

    pub fn radius(&self) -> f32 {
        match self {
            Brush::SoftCircle { base, .. } => base.radius,
        }
    }

    pub fn strength(&self) -> f32 {
        match self {
            Brush::SoftCircle { base, .. } => base.strength,
        }
    }

    //==========================================================================
    // mutator methods
    //==========================================================================
    pub fn set_spacing(&mut self, spacing: f32) {
        match self {
            Brush::SoftCircle { base, .. } => base.spacing = spacing,
        }
    }

    pub fn set_radius(&mut self, radius: f32) {
        match self {
            Brush::SoftCircle { base, .. } => base.radius = radius,
        }
    }

    pub fn set_strength(&mut self, strength: f32) {
        match self {
            Brush::SoftCircle { base, .. } => base.strength = strength,
        }
    }

    //==========================================================================
    // builder methods
    //==========================================================================

    pub fn with_spacing(self, spacing: f32) -> Self {
        match self {
            Brush::SoftCircle { inner_radius: hardness, mut base } => {
                base.spacing = spacing;
                Brush::SoftCircle { inner_radius: hardness, base }
            }
        }
    }

    pub fn with_radius(self, radius: f32) -> Self {
        match self {
            Brush::SoftCircle { inner_radius: hardness, mut base } => {
                base.radius = radius;
                Brush::SoftCircle { inner_radius: hardness, base }
            }
        }
    }

    pub fn with_strength(self, strength: f32) -> Self {
        match self {
            Brush::SoftCircle { inner_radius: hardness, mut base } => {
                base.strength = strength;
                Brush::SoftCircle { inner_radius: hardness, base }
            }
        }
    }
}

fn soft_circle(radius: f32, inner_radius: f32, color: Rgba) -> Stamp {
    let mut pixels = Vec::new();

    let radius_squared = radius * radius;
    let inner_radius_squared = inner_radius * inner_radius;

    for x in -radius as i32..=radius as i32 {
        for y in -radius as i32..=radius as i32 {
            let distance_squared = (x * x + y * y) as f32;
            if distance_squared <= radius_squared {
                let alpha_factor = if distance_squared <= inner_radius_squared {
                    1.0
                } else {
                    let t = ((distance_squared.sqrt() - inner_radius) / (radius - inner_radius)).min(1.0);
                    0.5 * (1.0 + f32::cos(t * std::f32::consts::PI))
                };

                pixels.push(Pixel {
                    x,
                    y,
                    color: color * alpha_factor
                });
            }
        }
    }

    Stamp { pixels }
}
