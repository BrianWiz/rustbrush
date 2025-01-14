pub use ecolor::Color32;

pub mod operations;

pub const RED_CHANNEL: usize = 0;
pub const GREEN_CHANNEL: usize = 1;
pub const BLUE_CHANNEL: usize = 2;
pub const ALPHA_CHANNEL: usize = 3;

/// A pixel is a single point in a pixel buffer with an RGBA color value.
pub struct Pixel {
    pub x: i32,
    pub y: i32,
    pub color: Color32,
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
        hardness: f32,
        base: BrushBaseSettings,
    },
}

impl Default for Brush {
    fn default() -> Self {
        Brush::SoftCircle {
            hardness: 0.1,
            base: BrushBaseSettings {
                id: "soft-circle".to_string(),
                radius: 10.0,
                spacing: 0.1,
                strength: 1.0,
            },
        }
    }
}

impl Brush {
    /// Gets a stamp for the current brush settings
    pub fn compute_stamp(&self, color: Color32) -> Stamp {
        match self {
            Brush::SoftCircle { hardness, base } => {
                soft_circle_flat(base.radius, *hardness, color)
            }
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
            Brush::SoftCircle { hardness, mut base } => {
                base.spacing = spacing;
                Brush::SoftCircle { hardness, base }
            }
        }
    }

    pub fn with_radius(self, radius: f32) -> Self {
        match self {
            Brush::SoftCircle { hardness, mut base } => {
                base.radius = radius;
                Brush::SoftCircle { hardness, base }
            }
        }
    }

    pub fn with_strength(self, strength: f32) -> Self {
        match self {
            Brush::SoftCircle { hardness, mut base } => {
                base.strength = strength;
                Brush::SoftCircle { hardness, base }
            }
        }
    }
}

fn soft_circle_flat(radius: f32, hardness: f32, color: Color32) -> Stamp {
    let mut pixels = Vec::new();
    for y in -radius as i32..=radius as i32 {
        for x in -radius as i32..=radius as i32 {
            let distance = (x as f32 * x as f32 + y as f32 * y as f32).sqrt();
            let alpha = (1.0 - (distance / radius).powf(hardness)).max(0.0);
            let alpha = (alpha * 255.0).round() as u8;
            if alpha > 0 {
                pixels.push(Pixel {
                    x,
                    y,
                    color: Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), alpha),
                });
            }
        }
    }

    Stamp { pixels }
}
