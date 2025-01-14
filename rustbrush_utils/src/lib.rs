pub use ecolor::{Color32, Rgba};

pub mod operations;

pub const RED_CHANNEL: usize = 0;
pub const GREEN_CHANNEL: usize = 1;
pub const BLUE_CHANNEL: usize = 2;
pub const ALPHA_CHANNEL: usize = 3;

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
    pub fn compute_stamp(&self) -> Stamp {
        match self {
            Brush::SoftCircle { inner_radius, base } => soft_circle(base.radius, *inner_radius),
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
            Brush::SoftCircle { inner_radius, mut base } => {
                base.spacing = spacing;
                Brush::SoftCircle { inner_radius, base }
            }
        }
    }

    pub fn with_radius(self, radius: f32) -> Self {
        match self {
            Brush::SoftCircle { inner_radius, mut base } => {
                base.radius = radius;
                Brush::SoftCircle { inner_radius, base }
            }
        }
    }

    pub fn with_strength(self, strength: f32) -> Self {
        match self {
            Brush::SoftCircle { inner_radius, mut base } => {
                base.strength = strength;
                Brush::SoftCircle { inner_radius, base }
            }
        }
    }
}

pub trait RgbaExtensions {
    fn overlay(&self, other: &Self) -> Self;
}

impl RgbaExtensions for Rgba {
    fn overlay(&self, other: &Self) -> Self {
        const BIAS : f32 = 1.3;

        let src_alpha = self.a();
        let dst_alpha = other.a();

        let new_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);

        // bias the blend slightly to preserve more color
        let blend = src_alpha * BIAS;

        let r = self.r() * blend + other.r() * (1.0 - blend);
        let g = self.g() * blend + other.g() * (1.0 - blend);
        let b = self.b() * blend + other.b() * (1.0 - blend);

        Rgba::from_rgba_premultiplied(
            r.min(1.0),
            g.min(1.0),
            b.min(1.0),
            new_alpha.min(1.0),
        )
    }
}

fn soft_circle(radius: f32, inner_radius: f32) -> Stamp {
    let mut pixels = Vec::new();
    let radius_squared = radius * radius;
    let inner_radius_squared = inner_radius * inner_radius;

    for x in -radius as i32..=radius as i32 {
        for y in -radius as i32..=radius as i32 {
            let distance_squared = (x * x + y * y) as f32;
            if distance_squared <= radius_squared {
                let distance = distance_squared.sqrt();
                let alpha = if distance_squared <= inner_radius_squared {
                    1.0
                } else {
                    let t = ((distance - inner_radius) / (radius - inner_radius)).min(1.0);
                    0.5 * (1.0 + f32::cos(t * std::f32::consts::PI))
                };

                pixels.push(Pixel {
                    x,
                    y,
                    color: Rgba::from_rgba_unmultiplied(1.0, 1.0, 1.0, alpha),
                });
            }
        }
    }

    Stamp { pixels }
}