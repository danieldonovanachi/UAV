#[derive(Debug, Clone)]
pub enum OptimizationControlFlow<E> {
    /// Found an optimal, or otherwise satisfied our criteria
    Converged,
    Ongoing {
        delta_energy: f32,
        total_iterations: usize,
    },
    Error(E),
}
