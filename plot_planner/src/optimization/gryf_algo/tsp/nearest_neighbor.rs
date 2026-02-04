use std::collections::HashMap;

use gryf::core::base::NeighborReference;

/// Computes an approximate solution to the TSP (or related problems) using the Nearest Neighbor heuristic.
pub fn nearest_neighbor<V, E, W, G, F>(
    graph: &G,
    start_vertex: G::VertexId,
    edge_weight: F,
) -> Result<super::TspPath<W, G>, super::Error>
where
    // Nearest Neighbor requires an undirected or symmetric graph (for the heuristic).
    G: gryf::core::GraphBase
        + gryf::core::Neighbors
        + gryf::core::VertexSet
        + gryf::core::EdgeSet
        + gryf::core::GraphWeak<V, E>,
    W: gryf::core::weight::Weight + PartialOrd + Clone,
    F: super::TravellingSalesmanCostFunction<G, E, W>,
    G::VertexId: Eq + std::hash::Hash + Clone,
{
    let n = graph.vertex_count();
    if n == 0 {
        return Err(super::Error::EmptyGraph)
    }

    // Initialize the tour, visited set, and total weight.
    let mut current_v = start_vertex;
    let mut visited = std::collections::HashSet::<G::VertexId>::new();
    let mut tour: Vec<G::VertexId> = Vec::with_capacity(n + 1);
    let mut total_weight = None; // Requires Weight trait to implement zero()

    visited.insert(current_v.clone());
    tour.push(current_v.clone());

    // 1. Traverse V-1 times
    while tour.len() < n {
        let mut best_neighbor: Option<G::VertexId> = None;
        let mut min_weight: Option<W> = None;
        //dbg!(tour.len(){
        //dbg!(&current_v)

        // Iterate over all outgoing edges from the current vertex.
        let neighbors =
            graph.neighbors_directed(&current_v, gryf::core::marker::Direction::Outgoing);
        for neighbor_ref in neighbors {
            let neighbor = neighbor_ref.id();

            if neighbor == gryf::core::borrow::OwnableRef::Borrowed(&current_v) {
                println!(
                    "ERROR: Neighbor {neighbor:?} is the same as current vertex {current_v:?}"
                );

                return Err(super::Error::SelfReferential);
            }

            // Check if the neighbor is unvisited.
            if !visited.contains(&neighbor) {
                //println!("Considering {current_v:?} -> )neighbor:?}")
                let edge_id = neighbor_ref.edge();
                let weight = edge_weight
                    .get_const()
                    .or_else(|| {
                        graph
                            .edge_weak(&edge_id)
                            .map(|edge| edge_weight.get_weight(&edge, &tour))
                    })
                    .ok_or(super::Error::EdgeNotAvailable)?;

                // Greedy choice: Is this the smallest weight so far?
                match min_weight {
                    Some(m_weight) if weight < m_weight => {
                        min_weight = Some(weight);
                        best_neighbor = Some(neighbor.into_owned());
                    }
                    None => {
                        min_weight = Some(weight);
                        best_neighbor = Some(neighbor.into_owned());
                    }
                    _ => {}
                }
            } else {
                //println!("Skipping )neighbor:?} as it's already visitied")
            }
        }

        // 2. Move to the best neighbor.
        match best_neighbor {
            Some(next_v) => {
                tour.push(next_v.clone());
                total_weight = Some(if let Some(total_weight) = total_weight {
                    total_weight + min_weight.unwrap()
                } else {
                    min_weight.unwrap()
                });
                visited.insert(next_v.clone());
                current_v = next_v;
            }
            None => {
                // Should not happen if the graph is connected, but signifies failure to complete the tour.
                return Err(super::Error::UnconnectedGraph);
            }
        }
    }

    Ok(super::TspPath {
        tour,
        total_weight: total_weight.unwrap(),
    })
}
