use core::simd;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct DirectionChangePenalty(pub f32);

impl DirectionChangePenalty {
    pub fn weight_simba<const L: usize>(
        &self,
        v_in: nalgebra::Vector2<simba::simd::Simd<simd::Simd<f32, L>>>,
        v_out: nalgebra::Vector2<simba::simd::Simd<simd::Simd<f32, L>>>,
    ) -> simd::Simd<f32, L>
    where
        simba::simd::Simd<std::simd::Simd<f32, L>>: nalgebra::SimdRealField,
    {
        let mag_in = v_in.magnitude();
        let mag_out = v_out.magnitude();

        use nalgebra::SimdPartialOrd;
        let safe_divisor =
            (mag_in * mag_out).simd_max(simba::simd::Simd(simd::Simd::splat(f32::EPSILON)));

        let dot = v_in.dot(&v_out);
        let cos_theta = dot / safe_divisor;

        (simd::Simd::splat(1.0) - cos_theta.0) * simd::Simd::splat(self.0 * 0.5)
    }

    pub fn weight<const L: usize>(
        &self,
        v_in: nalgebra::Vector2<simba::simd::Simd<simd::Simd<f32, L>>>,
        v_out: nalgebra::Vector2<simba::simd::Simd<simd::Simd<f32, L>>>,
    ) -> simd::Simd<f32, L> {
        fn chunk<const L: usize, const CHUNK_SIZE: usize>(
            dcp: &DirectionChangePenalty,
            v_in: nalgebra::Vector2<simba::simd::Simd<simd::Simd<f32, L>>>,
            v_out: nalgebra::Vector2<simba::simd::Simd<simd::Simd<f32, L>>>,
        ) -> simd::Simd<f32, L>
        where
            simba::simd::Simd<std::simd::Simd<f32, CHUNK_SIZE>>: nalgebra::SimdRealField,
        {
            // 1. Deconstruct wide vectors into standard core::simd arrays
            // We use .0 to get the underlying std::simd type from the simba wrapper
            let in_x: [f32; L] = v_in.x.0.into();
            let in_y: [f32; L] = v_in.y.0.into();
            let out_x: [f32; L] = v_out.x.0.into();
            let out_y: [f32; L] = v_out.y.0.into();

            let mut results = [0.0f32; L];

            // 2. Iterate in Simba-compatible chunks
            for i in (0..L).step_by(CHUNK_SIZE) {
                // Reconstruct small vectors for nalgebra
                let s_in = nalgebra::Vector2::new(
                    simba::simd::Simd(simd::Simd::<_, CHUNK_SIZE>::from_slice(
                        &in_x[i..i + CHUNK_SIZE],
                    )),
                    simba::simd::Simd(simd::Simd::<_, CHUNK_SIZE>::from_slice(
                        &in_y[i..i + CHUNK_SIZE],
                    )),
                );
                let s_out = nalgebra::Vector2::new(
                    simba::simd::Simd(simd::Simd::<_, CHUNK_SIZE>::from_slice(
                        &out_x[i..i + CHUNK_SIZE],
                    )),
                    simba::simd::Simd(simd::Simd::<_, CHUNK_SIZE>::from_slice(
                        &out_y[i..i + CHUNK_SIZE],
                    )),
                );

                // Calculate the penalty for this chunk
                // This inner call now satisfies the SimdRealField bounds
                let penalty_chunk = dcp.weight_simba::<CHUNK_SIZE>(s_in, s_out);

                let chunk_array: [f32; CHUNK_SIZE] = penalty_chunk.into();
                results[i..i + CHUNK_SIZE].copy_from_slice(&chunk_array);
            }

            simd::Simd::from_array(results)
        }

        match L {
            2 => chunk::<_, 2>(self, v_in, v_out),
            4 => chunk::<_, 4>(self, v_in, v_out),
            8 => chunk::<_, 8>(self, v_in, v_out),
            16 => chunk::<_, 16>(self, v_in, v_out),
            _ => chunk::<L, 16>(self, v_in, v_out),
        }
    }
}
