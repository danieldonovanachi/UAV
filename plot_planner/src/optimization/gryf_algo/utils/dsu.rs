use std::collections::HashMap;
use std::hash::Hash;

/// A minimal Disjoint Set Union structure for Kruskal's algorithm.
struct Dsu<VI> {
    // Map vertex ID to its parent's ID.
    parent: HashMap<VI, VI>,
}

impl<VI: Eq + Hash + Clone> Dsu<VI> {
    /// Initializes the DSU where every vertex is its own parent.
    fn new<I>(vertices: I) -> Self
    where
        I: Iterator<Item = VI>,
    {
        let parent = vertices.map(|v| (v.clone(), v)).collect();
        Dsu { parent }
    }

    /// Finds the representative (root) of the set containing `i`.
    /// Uses **path compression** for efficiency.
    fn find(&mut self, i: &VI) -> VI {
        let p = self.parent.get(i).expect("Vertex must be in DSU").clone();
        if p == *i {
            return i.clone();
        }
        // Path compression: set parent directly to the root
        let root = self.find(&p);
        self.parent.insert(i.clone(), root.clone());
        root
    }

    /// Unites the sets containing `i` and `j`. Returns `true` if a union occurred
    /// (i.e., they were in different sets), and `false` if they were already connected (cycle).
    fn union(&mut self, i: &VI, j: &VI) -> bool {
        let root_i = self.find(i);
        let root_j = self.find(j);

        if root_i != root_j {
            // Union by rank/size could be added, but simple union is sufficient here.
            self.parent.insert(root_i, root_j);
            true
        } else {
            false // Cycle detected
        }
    }
}
