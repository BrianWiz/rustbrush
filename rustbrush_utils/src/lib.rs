pub mod operations;

pub const RED_CHANNEL: usize = 0;
pub const GREEN_CHANNEL: usize = 1;
pub const BLUE_CHANNEL: usize = 2;
pub const ALPHA_CHANNEL: usize = 3;

/// A pixel is a single point in a pixel buffer with an RGBA color value.
pub struct Pixel {
    pub x: i32,
    pub y: i32,
    pub color: [u8; 4],
}

/// A stamp is a collection of pixels that represent a brush shape.
pub struct Stamp {
    pub pixels: Vec<Pixel>,
}

pub struct BrushBaseSettings {
    pub id: String,
    pub radius: f32,
    pub spacing: f32,
    pub opacity: f32,
}

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
                opacity: 1.0,
            },
        }
    }
}

impl Brush {
    /// Gets a stamp for the current brush settings
    pub fn compute_stamp(&self, color: [u8; 3]) -> Stamp {
        match self {
            Brush::SoftCircle { hardness, base } => {
                soft_circle(base.radius, *hardness, base.opacity, color)
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

    pub fn opacity(&self) -> f32 {
        match self {
            Brush::SoftCircle { base, .. } => base.opacity,
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

    pub fn with_opacity(self, opacity: f32) -> Self {
        match self {
            Brush::SoftCircle { hardness, mut base } => {
                base.opacity = opacity;
                Brush::SoftCircle { hardness, base }
            }
        }
    }
}

/// Generates a soft circle brush stamp which you can use to merge in with a pixel buffer. 
/// Generally you wouldn't call this directly and instead would use `PaintOperation::process`.
pub fn soft_circle(radius: f32, hardness: f32, opacity: f32, color: [u8; 3]) -> Stamp {
    let mut pixels = Vec::new();
    let radius_int = radius.ceil() as i32;
    
    for y in -radius_int..=radius_int {
        for x in -radius_int..=radius_int {
            let distance = ((x * x + y * y) as f32).sqrt();
            if distance <= radius {
                let normalized_dist = distance / radius;
                let alpha = if normalized_dist < hardness {
                    1.0
                } else {
                    let t = (normalized_dist - hardness) / (1.0 - hardness);
                    (1.0 - t * t * (3.0 - 2.0 * t)).max(0.0)
                };
                
                let alpha = (alpha * opacity * 255.0) as u8;
                pixels.push(Pixel {
                    x,
                    y,
                    color: [color[0], color[1], color[2], alpha],
                });
            }
        }
    }
    
    Stamp { pixels }
}
