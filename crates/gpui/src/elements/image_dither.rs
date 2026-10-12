/// Ordered GPU dithering applied after image sampling, before opacity and clipping.
#[derive(Clone, Copy, Debug)]
pub struct ImageDither {
    /// Number of quantization levels per color channel (2 through 256).
    pub levels: u16,
    /// Bayer threshold strength (0 through 1). Zero gives plain quantization.
    pub strength: f32,
    /// Width of a Bayer cell in logical pixels; must be finite and positive.
    pub pixel_size: f32,
}

impl Default for ImageDither {
    fn default() -> Self {
        Self {
            levels: 4,
            strength: 1.0,
            pixel_size: 2.0,
        }
    }
}

impl ImageDither {
    pub(crate) fn shader_params(self, scale_factor: f32) -> [f32; 4] {
        if !self.strength.is_finite() || !self.pixel_size.is_finite() || self.pixel_size <= 0.0 {
            return [0.0; 4];
        }
        [
            self.levels.clamp(2, 256) as f32,
            self.strength.clamp(0.0, 1.0),
            (self.pixel_size * scale_factor).max(1.0),
            0.0,
        ]
    }
}

#[cfg(test)]
mod dither_tests {
    use super::ImageDither;

    #[test]
    fn parameters_are_bounded_and_scaled() {
        assert_eq!(
            ImageDither {
                levels: 0,
                strength: 2.0,
                pixel_size: 2.0
            }
            .shader_params(2.0),
            [2.0, 1.0, 4.0, 0.0]
        );
        assert_eq!(
            ImageDither {
                levels: 1000,
                strength: -1.0,
                pixel_size: 0.1
            }
            .shader_params(1.0),
            [256.0, 0.0, 1.0, 0.0]
        );
    }

    #[test]
    fn invalid_parameters_disable_dithering() {
        for pixel_size in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(
                ImageDither {
                    pixel_size,
                    ..ImageDither::default()
                }
                .shader_params(1.0),
                [0.0; 4]
            );
        }
        assert_eq!(
            ImageDither {
                strength: f32::NAN,
                ..ImageDither::default()
            }
            .shader_params(1.0),
            [0.0; 4]
        );
    }
}
