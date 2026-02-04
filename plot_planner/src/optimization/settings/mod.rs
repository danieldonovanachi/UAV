mod direction_change;
pub use direction_change::DirectionChangePenalty;

mod specific_energy;
pub use specific_energy::SpecificEnergyCost;

use crate::generation::Point;

/// When creating a path from the generated primitives, and
/// for a physical vehicle (pen, robot, UAV), we want
/// our path to be optimized (if not optimal).
///
/// This means minimising some notion of cost, which can in general
/// be reduced to time & energy.
///
/// As time & energy are by themselves very dependent on the actual run,
/// we use principles as a proxy. These principles are:
///
/// A. Some directions are more costly than overs
///     as e.g in a vertical robot or UAV, up is more expensive than down
/// B. Changing direction (i.e. fighting inertia) is costly.
///     it is also relative to the amount of change
///     as e.g speed has to be reduced to take a turn
///
/// Inspirations:
/// - https://fis.tu-dresden.de/portal/files/44138491/SCITECH2024_Rienecker_Paper.pdf
#[derive(Clone, Debug)]
pub struct OptimizationSettings {
    pub specific_energy: SpecificEnergyCost,
    pub penalty: DirectionChangePenalty,
    pub start: nalgebra::Point2<f32>,
    pub include_start: bool,
}

impl crate::optimization::gryf_algo::MetricSequenceCostFunction for OptimizationSettings {
    type Weight = f32;

    #[inline(always)]
    fn compute_metric_sequence_weight<const L: usize>(
        &self,
        prev: Option<Point<L>>,
        from: Point<L>,
        to: Point<L>,
    ) -> (
        core::simd::Simd<Self::Weight, L>,
        core::simd::Mask<isize, L>,
    ) {
        // The weight of the branch
        let weight = self.specific_energy.weight_points(from, to);

        // And the penalty of the switch
        let penalty = match prev {
            None => core::simd::Simd::splat(0.0),
            Some(prev) => {
                let v_out = to - from;
                let v_out = <_ as Into<
                    nalgebra::Point2<simba::simd::Simd<core::simd::Simd<f32, L>>>,
                >>::into(v_out)
                .coords;

                let v_in = from - prev;
                let v_in = <_ as Into<
                    nalgebra::Point2<simba::simd::Simd<core::simd::Simd<f32, L>>>,
                >>::into(v_in)
                .coords;

                self.penalty.weight(v_in, v_out)
            }
        };

        (weight + penalty, unimplemented!())
    }
}
