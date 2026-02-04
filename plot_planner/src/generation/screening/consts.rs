pub struct QuadritoneAngles {
    c: f32,
    m: f32,
    y: f32,
    k: f32,
}

impl QuadritoneAngles {
    pub const EUROPEAN: Self = Self {
        c: 15.0f32.to_radians(),
        m: 75.0f32.to_radians(),
        y: 0.0f32.to_radians(),
        k: 45.0f32.to_radians(),
    };

    pub const AMERICAN: Self = Self {
        c: 15.0f32.to_radians(),
        m: 75.0f32.to_radians(),
        y: 0.0f32.to_radians(),
        k: 45.0f32.to_radians(),
    };
}

pub struct TritoneAngles {
    darkest: f32,
    medium: f32,
    lightest: f32,
}

impl TritoneAngles {
    pub const DEFAULT: Self = Self {
        darkest: 45.0f32.to_radians(),
        medium: 75.0f32.to_radians(),
        lightest: 15.0f32.to_radians(),
    };
}

pub struct BitoneAngles {
    dark: f32,
    light: f32,
}

impl BitoneAngles {
    pub const DEFAULT: Self = Self {
        dark: 45.0f32.to_radians(),
        light: 75.0f32.to_radians(),
    };
}
