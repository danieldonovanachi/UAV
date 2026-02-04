use core::simd;

/// Costs of a translation
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SpecificEnergyCost {
    pub weights: nalgebra::Matrix2<f32>,
    pub asymmetry: nalgebra::Matrix2<f32>,
}

impl std::default::Default for SpecificEnergyCost {
    fn default() -> Self {
        Self::identity()
    }
}

impl SpecificEnergyCost {
    /// Creates the metric matrices from directional penalties.
    ///
    /// # Arguments
    /// * `up` - Cost multiplier for moving exactly [0, -1]
    /// * `down` - Cost multiplier for moving exactly [0, 1]
    /// * `sideways` - Cost multiplier for moving exactly [1, 0] or [-1, 0]
    pub fn from_penalties(up: f32, down: f32, sideways: f32) -> Self {
        // Symmetric Component (Ms)
        // We store weights as the squares because our metric uses vᵀ * Ms * v
        let w_xx = sideways.powi(2);
        let w_yy = ((up + down) / 2.0).powi(2);

        // Asymmetric Component (Ma)
        // This is a vector representing the "flow" field.
        // If down > up, bias is positive (gravity helps move down).
        let a_y = (down - up) / 2.0;
        let a_x = 0.0; // Assuming symmetry on the horizontal axis

        Self {
            weights: nalgebra::Matrix2::new(w_xx, 0.0, 0.0, w_yy),
            asymmetry: nalgebra::Matrix2::new(a_x, 0.0, 0.0, a_y),
        }
    }

    pub fn identity() -> Self {
        Self {
            weights: nalgebra::Matrix2::identity(),
            asymmetry: nalgebra::Matrix2::identity(),
        }
    }

    const fn min_spec_e(&self) -> f32 {
        todo!()
    }

    /// Computes the weight for a given translation
    #[inline(always)]
    pub fn weight<const L: usize>(
        &self,
        translation: nalgebra::Vector2<simba::simd::Simd<core::simd::Simd<f32, L>>>,
    ) -> core::simd::Simd<f32, L>
    where
        simba::simd::Simd<std::simd::Simd<f32, L>>: simba::simd::SimdSigned
            + core::ops::AddAssign
            + core::ops::MulAssign
            + core::cmp::PartialEq
            + nalgebra::SimdComplexField<SimdRealField = simba::simd::Simd<std::simd::Simd<f32, L>>>
            + nalgebra::SimdValue<Element = f32>,
    {
        // Splat & multiply with the weight matrices (both the assymetric & the symmetric component)
        // That gets us the weight of the displacement, still componentwise (& splatted)

        // Symmetric Metric (Euclidean/Riemannian part)
        // vᵀ * Ms * v
        // If Ms is a 2x2 matrix, this is (v.x, v.y) * [Wxx, Wxy; Wyx, Wyy] * (v.x, v.y)ᵀ
        let absed = (&translation).map(|v| v.simd_abs());
        let symmetric_cost = absed.transpose() * self.weights.cast() * absed;

        // Asymmetric Drift (Linear part)
        // Ma * v
        // This is a simple dot product: (Ax * v.x) + (Ay * v.y)
        let asymmetric_cost = (translation.transpose() * self.asymmetry.cast()).x;

        use nalgebra::SimdComplexField;
        let combined_cost = symmetric_cost.x.simd_sqrt() + asymmetric_cost;

        combined_cost.0
    }

    /// SIMD version of the directional weighting
    pub fn weight_points_simba<const L: usize>(
        &self,
        src_points: crate::generation::Point<L>,
        dst_points: crate::generation::Point<L>,
    ) -> simd::Simd<f32, L>
    where
        simba::simd::Simd<std::simd::Simd<f32, L>>: simba::simd::SimdSigned
            + core::ops::AddAssign
            + core::ops::MulAssign
            + core::cmp::PartialEq
            + nalgebra::SimdRealField
            + nalgebra::SimdValue<Element = f32>,
    {
        let diff = <crate::generation::Point<L> as Into<nalgebra::Point2<_>>>::into(dst_points)
            - <crate::generation::Point<L> as Into<nalgebra::Point2<_>>>::into(src_points);
        self.weight(diff)
    }

    pub fn weight_points<const L: usize>(
        &self,
        src_points: crate::generation::Point<L>,
        dst_points: crate::generation::Point<L>,
    ) -> simd::Simd<f32, L> {
        fn chunk<const L: usize, const CHUNK_SIZE: usize>(
            sec: &SpecificEnergyCost,
            src_points: crate::generation::Point<L>,
            dst_points: crate::generation::Point<L>,
        ) -> simd::Simd<f32, L>
        where
            simba::simd::Simd<std::simd::Simd<f32, CHUNK_SIZE>>: core::ops::AddAssign
                + core::ops::MulAssign
                + core::cmp::PartialEq
                + nalgebra::SimdRealField
                + nalgebra::SimdValue<Element = f32>,
        {
            // 1. Calculate the raw difference using standard core::simd (which supports any L)
            let diff_x = dst_points.x - src_points.x;
            let diff_y = dst_points.y - src_points.y;

            // 2. Prepare output buffer
            let mut results = [0.0f32; L];

            // We use arrays to "escape" the trait bound limitation of the generic L
            let x_array: [f32; L] = diff_x.into();
            let y_array: [f32; L] = diff_y.into();

            for i in (0..L).step_by(CHUNK_SIZE) {
                // Load a 4-lane Simba-compatible vector
                let sx =
                    core::simd::Simd::<f32, CHUNK_SIZE>::from_slice(&x_array[i..i + CHUNK_SIZE]);
                let sy =
                    core::simd::Simd::<f32, CHUNK_SIZE>::from_slice(&y_array[i..i + CHUNK_SIZE]);

                // Wrap in the nalgebra-friendly Simd type
                let translation = nalgebra::Vector2::new(
                    simba::simd::Simd::<core::simd::Simd<f32, CHUNK_SIZE>>(sx),
                    simba::simd::Simd::<core::simd::Simd<f32, CHUNK_SIZE>>(sy),
                );

                // This call now satisfies the trait bounds because it is explicitly implemented
                let cost_chunk = sec.weight::<CHUNK_SIZE>(translation);

                // Store back to array
                let chunk_array: [f32; CHUNK_SIZE] = cost_chunk.into();
                results[i..i + CHUNK_SIZE].copy_from_slice(&chunk_array);
            }

            core::simd::Simd::from_array(results)
        }

        match L {
            2 => chunk::<_, 2>(self, src_points, dst_points),
            4 => chunk::<_, 4>(self, src_points, dst_points),
            8 => chunk::<_, 8>(self, src_points, dst_points),
            16 => chunk::<_, 16>(self, src_points, dst_points),
            _ => chunk::<L, 16>(self, src_points, dst_points),
        }
    }
}
