mod delaunator;
use core::f32;

use delaunator::{Triangulation, Point, triangulate};

use zerocopy_derive::{FromBytes, Immutable, IntoBytes};
use crate::shared::{PositionF32, AABB, pos};
use crate::store::StoreLoad;
use super::GameWorldData;

/// The identifier of a triangle in the navmesh. 
/// The inner identifier is also the index of the triangle in the navigation graph
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, FromBytes, Immutable, IntoBytes)]
pub struct Triangle(u32);

impl Triangle {
    pub const fn as_graph_node_index(&self) -> usize { self.0 as usize }
    pub const fn is_outside(&self) -> bool { self.0 == u32::MAX }
    pub const fn outside() -> Self { Triangle(u32::MAX) }
}

/// State used when looking up a triangle using `triangle_at`
#[derive(Copy, Clone)]
struct StepState {
    target: PositionF32,
    edge: usize,
    done: bool,
}

/// One of the three possible neighbors of a NavNode
#[derive(Debug, Copy, Clone, FromBytes, Immutable, IntoBytes)]
struct NavNodeNeighbor {
    pub center: PositionF32,
    pub segment: [PositionF32; 2],
    pub triangle: Triangle,
    pub distance: f32,
}

impl NavNodeNeighbor {
    pub const fn as_graph_node_index(&self) -> usize { self.triangle.as_graph_node_index() }
}

/// A node in the pathfinding graph
#[derive(Debug, Copy, Clone, Default, FromBytes, Immutable, IntoBytes)]
struct NavNode {
    pub triangle: Triangle,
    pub center: PositionF32,
    pub n0: NavNodeNeighbor,
    pub n1: NavNodeNeighbor,
    pub n2: NavNodeNeighbor,
}

impl NavNode {
    pub const fn is_disconnected(&self) -> bool { 
        self.n0.triangle.is_outside() && self.n1.triangle.is_outside() && self.n2.triangle.is_outside()
    }

    pub fn gate(&self, from: usize) -> [PositionF32; 2] {
        let triangle = Triangle(from as u32);
        if self.n0.triangle == triangle {
            self.n0.segment
        } else if self.n1.triangle == triangle {
            self.n1.segment
        } else if self.n2.triangle == triangle {
            self.n2.segment
        } else {
            Default::default()
        }
    }
}

/// A temporary node used when computing the optimal path between two points
#[derive(Copy, Clone, Default)]
struct PathComputeNode {
    /// Index of the `NavNode` this compute node references
    pub node_index: usize,
    /// Index of the `NavNode` we used to reach this node
    pub came_from: usize,
    pub cost_to_start: f32,
    pub estimated_cost_to_end: f32,
}

/// Node used to construct a rough path from one point to another
#[derive(Copy, Clone, Default)]
struct PathComputeNodeParent {
    pub index: usize,
    pub parent_index: usize,
    pub gate: [PositionF32; 2],
    pub cost_to_start: f32,
}

enum PathfindingOutput<'a> {
    Gates(&'a mut Vec<[PositionF32; 2]>),
    Nodes(&'a mut Vec<PathComputeNodeParent>)
}


/// 2D pathfinding state
#[derive(Default)]
pub struct NavigationState {
    points: Vec<Point>,
    triangulation: Option<Triangulation>,
    blocked_areas: Vec<AABB>,
    graph: Vec<NavNode>
}

impl NavigationState {
    
    pub fn clear(&mut self) {
        self.points.clear();
        self.graph.clear();
        self.triangulation = None;
    }

    //
    // Navmesh generation
    //

    pub fn rebuild_navmesh(world_data: &mut GameWorldData) {
        let nav = &mut world_data.data.navigation;
        let terrain = &world_data.data.terrain;

        nav.clear();

        Self::terrain_points(terrain, &mut nav.points);
        Self::sprites_collision_points(&world_data.world, &mut nav.points, &mut nav.blocked_areas);
        nav.triangulation = Some(triangulate(&nav.points));

        nav.generate_nav_graph();
        nav.remove_blocked_nodes_from_graph();

        // Blocked areas are not needed past this point
        // We store the vec in the state to reuse the memory between rebuild
        nav.blocked_areas.clear(); 
    }

    fn terrain_points(terrain: &super::Terrain, points: &mut Vec<Point>) {
        let cell_size = super::terrain::TERRAIN_SPRITE_SIZE;
        let w = (terrain.width() as f32) * cell_size;
        let h = (terrain.height() as f32) * cell_size;
        points.push(pos(0.0, 0.0));
        points.push(pos(w, 0.0));
        points.push(pos(0.0, h));
        points.push(pos(w, h));
    }

    fn sprites_collision_points(world: &super::World, points: &mut Vec<Point>, blocked_area: &mut Vec<AABB>) {
        let mut sprites = world.sprites_with_collisions();
        for (_, (sprite, _)) in sprites.iter() {
            let aabb = sprite.rect();
            blocked_area.push(aabb);
            points.push(pos(aabb.left, aabb.top));
            points.push(pos(aabb.left, aabb.bottom));
            points.push(pos(aabb.right, aabb.top));
            points.push(pos(aabb.right, aabb.bottom));
        }
    }

    fn generate_nav_graph(&mut self) {
        let triangulation = match self.triangulation.as_ref() {
            Some(t) => t,
            None => { return; }
        };

        let points = &self.points;
        let triangle_count = triangulation.triangles.len() / 3;
        
        for i in 0..triangle_count {
            let mut node = NavNode::default();
            node.triangle = Triangle(i as u32);

            // Edges of the current triangle
            let e0 = i * 3;
            let e1 = e0 + 1;
            let e2 = e0 + 2;

            let mut p0 = points[triangulation.triangles[e0]];
            let mut p1 = points[triangulation.triangles[e1]];
            let mut p2 = points[triangulation.triangles[e2]];
            let center_start = pos((p0.x + p1.x + p2.x) / 3.0, (p0.y + p1.y + p2.y) / 3.0);
            let mut center_stop;

            node.center = center_start;

            // Edges of the neighbors triangle
            let mut e3;
            let mut e4;
            let mut e5;

            e3 = triangulation.halfedges[e0];
            if e3 != usize::MAX {
                e4 = delaunator::next_halfedge(e3);
                e5 = delaunator::next_halfedge(e4);
                p0 = points[triangulation.triangles[e3]];
                p1 = points[triangulation.triangles[e4]];
                p2 = points[triangulation.triangles[e5]];
                center_stop = pos((p0.x + p1.x + p2.x) / 3.0, (p0.y + p1.y + p2.y) / 3.0);

                node.n0.triangle = self.triangle_of_edge(e3);
                node.n0.distance = center_start.distance(center_stop);
                node.n0.center = center_stop;
                node.n0.segment = [p0, p1];
            }

            e3 = triangulation.halfedges[e1];
            if e3 != usize::MAX {
                e4 = delaunator::next_halfedge(e3);
                e5 = delaunator::next_halfedge(e4);
                p0 = points[triangulation.triangles[e3]];
                p1 = points[triangulation.triangles[e4]];
                p2 = points[triangulation.triangles[e5]];
                center_stop = pos((p0.x + p1.x + p2.x) / 3.0, (p0.y + p1.y + p2.y) / 3.0);
                
                node.n1.triangle = self.triangle_of_edge(e3);
                node.n1.distance = center_start.distance(center_stop);
                node.n1.center = center_stop;
                node.n1.segment = [p0, p1];
            }

            e3 = triangulation.halfedges[e2];
            if e3 != usize::MAX {
                e4 = delaunator::next_halfedge(e3);
                e5 = delaunator::next_halfedge(e4);
                p0 = points[triangulation.triangles[e3]];
                p1 = points[triangulation.triangles[e4]];
                p2 = points[triangulation.triangles[e5]];
                center_stop = pos((p0.x + p1.x + p2.x) / 3.0, (p0.y + p1.y + p2.y) / 3.0);
                
                node.n2.triangle = self.triangle_of_edge(e3);
                node.n2.distance = center_start.distance(center_stop);
                node.n2.center = center_stop;
                node.n2.segment = [p0, p1];
            }

            self.graph.push(node);
        }
    }

    fn remove_blocked_nodes_from_graph(&mut self) {
        let node_count = self.graph.len();
        let blocked_area_count = self.blocked_areas.len();

        fn disconnect_node(node: &mut NavNode) {
            node.n0.triangle = Triangle::outside();
            node.n0.distance = f32::INFINITY;
            node.n1.triangle = Triangle::outside();
            node.n1.distance = f32::INFINITY;
            node.n2.triangle = Triangle::outside();
            node.n2.distance = f32::INFINITY;
        }

        fn disconnect_node_neighbors(nodes: &mut Vec<NavNode>, from: Triangle, n0: Triangle, n1: Triangle, n2: Triangle) {
            let nodes_indices = [n0.0 as usize, n1.0 as usize, n2.0 as usize];
            for i in nodes_indices {
                if i == usize::MAX {
                    continue;
                }

                let node = &mut nodes[i];
                if node.n0.triangle == from {
                    node.n0.triangle = Triangle::outside();
                    node.n0.distance = f32::INFINITY;
                } else if node.n1.triangle == from {
                    node.n1.triangle = Triangle::outside();
                    node.n1.distance = f32::INFINITY;
                } else if node.n2.triangle == from {
                    node.n2.triangle = Triangle::outside();
                    node.n2.distance = f32::INFINITY;
                }
            }
        }

        // Brute force algorithm. There are better way to handle this.
        // For example by querying the triangle inside the blocked area using `triangle_at`
        for blocked_index in 0..blocked_area_count {
            let blocked = self.blocked_areas[blocked_index];
            for node_index in 0..node_count {
                let node = self.graph[node_index];
                let [p0, p1, p2] = self.triangle_points(node.triangle);
                let center = pos((p0.x + p1.x + p2.x) / 3.0, (p0.y + p1.y + p2.y) / 3.0);
                if blocked.point_inside(center) {
                    disconnect_node(&mut self.graph[node_index]);
                    disconnect_node_neighbors(&mut self.graph, node.triangle, node.n0.triangle, node.n1.triangle, node.n2.triangle);
                }
            }
        }
    }

    //
    // Triangle lookup by position
    //

    pub fn triangle_at(&self, position: PositionF32, start_edge: u32) -> Option<Triangle> {
        if self.triangulation.is_none() {
            return None;
        }

        let mut state =  StepState {
            target: position,
            edge: start_edge as usize,
            done: false,
        };

        loop {
            self.step(&mut state);
            if state.done || state.edge == usize::MAX {
                break;
            }
        }

        if state.edge == usize::MAX {
            None
        } else {
            Some(self.triangle_of_edge(state.edge))
        }
    }

    fn step(&self, state: &mut StepState) {
        let sibling = delaunator::next_halfedge(state.edge);
        let last_sibling = delaunator::next_halfedge(sibling);

        let tri = match self.triangulation.as_ref() {
            Some(t) => t,
            None => unsafe { ::std::hint::unreachable_unchecked() }
        };

        let point1 = self.points[tri.triangles[state.edge]];
        let point2 = self.points[tri.triangles[sibling]];
        let point3 = self.points[tri.triangles[last_sibling]];

        // If `target` is not counterclockwise (cc), this means the point is in another triangle.
        // If so we test the opposite half edge in the next iteration
        let mut next_edge = None;
        if orient_point(point1, point2, state.target) > 0.0 {
            next_edge = Some(tri.halfedges[state.edge]);
        }
        else if orient_point(point2, point3, state.target) > 0.0 {
            next_edge = Some(tri.halfedges[sibling]);
        }
        else if orient_point(point3, point1, state.target) > 0.0 {
            next_edge = Some(tri.halfedges[last_sibling]);
        }

        if let Some(next_edge) = next_edge {
            state.edge = next_edge;
            return;
        }

        state.done = true;
    }

    //
    // A* pathfinding
    //

    fn get_pathfinding_nodes(&self, start: PositionF32, end: PositionF32) -> Option<[usize; 2]> {
        let start = self.triangle_at(start, 0).unwrap_or(Triangle::outside());
        let end = self.triangle_at(end, 0).unwrap_or(Triangle::outside());
        if start.is_outside() || end.is_outside() {
            return None;
        }

        let start = start.as_graph_node_index();
        let end = end.as_graph_node_index();
        if self.graph[start].is_disconnected() || self.graph[end].is_disconnected() {
            return None;
        }

        Some([start, end])
    }

    /**
        Compute a rough (without smoothing) path from start to end using the A* algorithm. Returns `true` if a path was found or returns `false` otherwise.
        Panics if `start` and `end` are in the same triangle (the case should be handled by the caller).

        Either writes writes the nodes to walk through or the gates (edges the actor will have to go through) in output
        `PathfindingOutput::Gates` is used by the final algorithm and `PathfindingOutput::Nodes` is used for debugging purpose.
    */
    fn compute_rough_path(
        &self,
        start_node_index: usize,
        end_node_index: usize,
        output: PathfindingOutput
    ) -> bool {
        use std::collections::{BinaryHeap, HashMap, hash_map::Entry};

        fn heuristic(p1: PositionF32, p2: PositionF32) -> f32 {
            p1.distance(p2)
        }

        fn build_final_path(
            end_node_index: usize,
            processed: &mut HashMap<usize, PathComputeNodeParent>,
            mut output: PathfindingOutput
        ) {
            let mut next = end_node_index;
            loop {
                let current = processed.get(&next).copied()
                    .unwrap_or(PathComputeNodeParent { index: next, parent_index: next, gate: Default::default(), cost_to_start: 0.0 }); // Unwrap should never be reached
                
                if current.index == current.parent_index {
                    break; // First node loop with itself
                }

                match &mut output {
                    PathfindingOutput::Gates(gates) => { gates.push(current.gate); },
                    PathfindingOutput::Nodes(nodes) => { nodes.push(current); }
                }

                next = current.parent_index;
            }

            // We need to reverse the path because it was built from the last node to the first
            match &mut output {
                PathfindingOutput::Gates(gates) => gates.reverse(),
                PathfindingOutput::Nodes(nodes) => nodes.reverse(),
            }
        }

        // A list of processed nodes that stores the cost to reach that node and the index of the parent node
        // Nodes can be walked back to build the final path (see `build_final_path`)
        let mut processed: HashMap<usize, PathComputeNodeParent> = HashMap::new();

        // A Priority queue to process the nodes from the smallest cost to the largest
        let mut to_see: BinaryHeap<PathComputeNode> = BinaryHeap::new();

        to_see.push(PathComputeNode { node_index: start_node_index, cost_to_start: 0.0, estimated_cost_to_end: 0.0, came_from: 3 });
        processed.insert(start_node_index, PathComputeNodeParent { index: start_node_index, parent_index: start_node_index, gate: Default::default(), cost_to_start: 0.0 });

        while let Some(cell) = to_see.pop() {
            if cell.node_index == end_node_index {
                // Last node
                let gate = self.graph[cell.came_from].gate(cell.node_index);
                processed.insert(cell.node_index, PathComputeNodeParent { 
                    index: cell.node_index,
                    parent_index: cell.came_from,
                    gate,
                    cost_to_start: 0.0 
                });

                build_final_path(cell.node_index, &mut processed, output);

                return true;
            }

            let node = self.graph[cell.node_index];
            let neighbors = [node.n0, node.n1, node.n2];

            for next in neighbors {
                if next.triangle.is_outside() || next.triangle.as_graph_node_index() == cell.came_from {
                    continue;
                }

                let next_node_index = next.as_graph_node_index();
                let new_cost = cell.cost_to_start + next.distance;
                let parent_value = PathComputeNodeParent { index: next_node_index, parent_index: cell.node_index, gate: next.segment, cost_to_start: new_cost };
                match processed.entry(next_node_index) {
                    Entry::Vacant(e) => {
                        e.insert(parent_value);
                    }
                    Entry::Occupied(mut e) => {
                        // If the new node is more efficient that the old one, replace it. Otherwise skip it
                        if new_cost < e.get().cost_to_start {
                            e.insert(parent_value);
                        } else {
                            continue;
                        }
                    }
                }

                to_see.push(PathComputeNode {
                    node_index: next_node_index,
                    came_from: cell.node_index,
                    cost_to_start: new_cost,
                    estimated_cost_to_end: new_cost + heuristic(node.center, next.center),
                });
            }
        }

        return false;
    }

    // Funnel algorithm
    fn compute_funnel(
        &self,
        start: PositionF32,
        end: PositionF32,
        gates: &[[PositionF32; 2]],
        output: &mut Vec<PositionF32>,
    ) {
        /// By computing the cross product of the apex, the opposite point, and the new point we can find
        /// if moving one side would tighten the funnel (if the cross product returns a smaller value)
        fn cross_product(apex: PositionF32, a: PositionF32, b: PositionF32) -> f32 {
            let ax = a.x - apex.x;
            let ay = a.y - apex.y;
            let bx = b.x - apex.x;
            let by = b.y - apex.y;

            // dbg!("cross {:?} {:?} {:?} = {}", apex, a, b,  bx*ay - ax*by);

            bx*ay - ax*by
        }

        assert!(gates.len() >= 1, "Funnel algorithm requires that rough_output contains at least one node");

        // Path always begins with the start node
        output.push(start);

        let mut apex = start;
        let mut right_index = 0;
        let mut left_index = 0;
        let [mut left, mut right] = gates.first().map(|[left, right]| [*left, *right] ).unwrap();

        let gates_count = gates.len();
        let mut gate_index = 1;
        let mut iter_count = 0;
        while gate_index < gates_count {
            let new_right = gates[gate_index][1];
            let new_left = gates[gate_index][0];

            if iter_count == 10 {
                break;
            }
            iter_count += 1;

            // Right vertex
            // cross_product will return >= 0 if moving the `right` point to `new_right` would tighten the funnel
            if cross_product(apex, right, new_right) >= 0.0 {
                // cross_product of `apex`, `left`, and `new_right` checks if the funnel degenerates to a line segment
                if right == apex || cross_product(apex, left, new_right) < 0.0 { 
                    // If it doesn't we move the funnel point
                    right_index = gate_index;
                    right = new_right;
                } else {
                    // If the funnel degenerates to a line segment. We need to move the apex to the current opposite point (left in this case)
                    // The new apex will a point in the path.
                    
                    // Insert pivot point
                    output.push(left);

                    // Resets loop at gate index
                    right_index = left_index;
                    gate_index = left_index+1;

                    // Update apex & reset gates
                    apex = left;
                    left = apex;
                    right = apex;

                    continue;
                }
            }

            // Left vertex
            // cross_product will return <= 0 if moving the `left` point to `new_left` would tighten the funnel
            if cross_product(apex, left, new_left) <= 0.0 {
                if left == apex || cross_product(apex, right, new_left) > 0.0 {
                    left_index = gate_index;
                    left = new_left;
                } else {
                    output.push(right);
                    
                    left_index = right_index;
                    gate_index = right_index+1;

                    apex = right;
                    right = apex;
                    left = apex;

                    continue;
                }
            }

            gate_index += 1;
        }

        // Process the path to the `end` node.
        loop {
            if right_index >= gates_count && left_index >= gates_count {
                break;
            } else {
                if left_index < gates_count {
                    left = gates[left_index][0];
                }

                if right_index < gates_count {
                    right = gates[right_index][1];
                }
            }
            
            if apex != right && cross_product(apex, left, end) > 0.0 { 
                output.push(left);
                apex = left;

                left_index +=1;
                right_index = left_index;
    
                continue;
            } else {
                right_index += 1;
            }
        
            if apex != left && cross_product(apex, right, end) < 0.0 {
                output.push(right);
                apex = right;

                right_index +=1;
                left_index = right_index;

                continue;
            } else {
                left_index += 1;
            }
        }

        // Path always ends with the end node
        output.push(end);
    }

    /**
        Compute and optimize the path between `start` and `end`. Returns a vector of points the actor at `start` will move through to reach `end`
    */
    fn compute_path(&self, start: PositionF32, end: PositionF32, output: &mut Vec<PositionF32>) -> bool {
        let [start_node_index, end_node_index] = match self.get_pathfinding_nodes(start, end) {
            Some([start, end]) => [start, end],
            None => { return false; }
        };

        // If we're in the same node, just return right away
        if start_node_index == end_node_index {
            output.push(start);
            output.push(end);
            return true;
        }

        let mut gates = Vec::with_capacity(16);
        if !self.compute_rough_path(start_node_index, end_node_index, PathfindingOutput::Gates(&mut gates)) {
            return false;
        }

        self.compute_funnel(start, end, &mut gates, output);

        true
    }

    //
    // Debug
    //

    pub fn debug_navmesh(&self, debug: &mut super::DebugState, show_cell_centers: bool) {
        let triangulation = match self.triangulation.as_ref() {
            Some(t) => t,
            None => { return; }
        };

        let edges_count = triangulation.triangles.len();
        let color = [255, 0, 0, 255];

        let mut i = 0;
        while i < edges_count {
            let triangle = self.triangle_of_edge(i);
            let [p1, p2, p3] = self.triangle_points(triangle); 
            debug.draw_triangle(p1, p2, p3, color);

            if show_cell_centers {
                let x = (p1.x + p2.x + p3.x) / 3.0;
                let y = (p1.y + p2.y + p3.y) / 3.0;
                debug.draw_point(pos(x, y), 3.0, [255, 0, 0, 255]);
            }

            i += 3;
        }
    }

    pub fn debug_triangle_at_position(&self, debug: &mut super::DebugState, position: PositionF32) {
        let triangle = self.triangle_at(position, 49);
        if let Some(triangle) = triangle {
            let [v0, v1, v2] = self.triangle_points(triangle);
            debug.fill_triangle(v0, v1, v2, [255, 255, 255, 100]);
        }
    }

    pub fn debug_triangle_lookup_path(&self, debug: &mut super::DebugState, position: PositionF32) {
        if self.triangulation.is_none() {
            return;
        }

        let mut state = StepState {
            target: position,
            edge: 0,
            done: false,
        };

        let triangle =  self.triangle_of_edge(state.edge);
        let [v0, v1, v2] = self.triangle_points(triangle);
        debug.fill_triangle(v0, v1, v2, [0, 0, 255, 100]);

        loop {
            self.step(&mut state);
            if state.done || state.edge == usize::MAX {
                break;
            }

            let triangle =  self.triangle_of_edge(state.edge);
            let [v0, v1, v2] = self.triangle_points(triangle);
            debug.fill_triangle(v0, v1, v2, [255, 255, 255, 100]);
        }

        if state.edge != usize::MAX {
            let triangle =  self.triangle_of_edge(state.edge);
            let [v0, v1, v2] = self.triangle_points(triangle);
            debug.fill_triangle(v0, v1, v2, [0, 255, 0, 100]);
        }
    }

    pub fn debug_pathfinding_graph(&self, debug: &mut super::DebugState) {
        let triangulation = match self.triangulation.as_ref() {
            Some(t) => t,
            None => { return; }
        };

        let triangle_count = triangulation.triangles.len() / 3;
        let center_color = [255, 255, 0, 255];
        let mut i = 0;

        while i < triangle_count {
            let node = self.graph[i];

            let n0 = node.n0.triangle.0;
            let n1 = node.n1.triangle.0;
            let n2 = node.n2.triangle.0;

            if n0 != u32::MAX || n1 != u32::MAX || n2 != u32::MAX {
                debug.draw_point(node.center, 3.0, center_color);
            }

            if n0 != u32::MAX && n0 > (i as u32) {
                debug.draw_line(node.center, node.n0.center, center_color);
            }

            if n1 != u32::MAX && n1 > (i as u32) {
                debug.draw_line(node.center, node.n1.center, center_color);
            }

            if n2 != u32::MAX && n2 > (i as u32) {
                debug.draw_line(node.center, node.n2.center, center_color);
            }

            i += 1;
        }
    }

    /// Show path without the simple stupid funnel algorithm processing
    pub fn debug_rough_path(&self, debug: &mut super::DebugState, start: PositionF32, end: PositionF32) {
        let [start_node_index, end_node_index] = match self.get_pathfinding_nodes(start, end) {
            Some([start, end]) => [start, end],
            None => { return; }
        };

        let mut output = Vec::with_capacity(16);
        if start_node_index == end_node_index {
            debug.draw_line(start, end, [255, 0, 255, 255]);
            return;
        }

        if !self.compute_rough_path(start_node_index, end_node_index, PathfindingOutput::Nodes(&mut output)) {
            return;
        }

        debug.set_current_layer(1);

        let mut last = start;
        for node in output {
            let node = self.graph[node.index];
            let current = node.center;
            debug.draw_line(last, current, [255, 0, 255, 255]);

            last = current;
        }

        debug.draw_line(last, end, [255, 0, 255, 255]);

        debug.set_current_layer(0);
    }

    /// Show the funnel of a rough path.
    pub fn debug_funnel(&self, debug: &mut super::DebugState, start: PositionF32, end: PositionF32) {
        let [start_node_index, end_node_index] = match self.get_pathfinding_nodes(start, end) {
            Some([start, end]) => [start, end],
            None => { return; }
        };

        let mut rough_output = Vec::with_capacity(16);

        // If we're in the same node, just return right away
        if start_node_index == end_node_index { return; }
        if !self.compute_rough_path(start_node_index, end_node_index, PathfindingOutput::Nodes(&mut rough_output)) { return; }

        debug.set_current_layer(1);

        // First & End triangle
        let [v0, v1, v2] = self.triangle_points(Triangle(start_node_index as u32));
        debug.draw_triangle(v0, v1, v2, [255, 255, 0, 255]);

        let [v0, v1, v2] = self.triangle_points(Triangle(end_node_index as u32));
        debug.draw_triangle(v0, v1, v2, [255, 255, 0, 255]);

        // Gates
        let [mut p1, mut p2] = rough_output.first().map(|path_node| path_node.gate ).unwrap();

        let mut last_point = rough_output.first().map(|path_node| self.graph[path_node.index].center ).unwrap();
        debug.draw_line(start, last_point, [0, 255, 255, 255]);

        for i in 0..rough_output.len() {
            let node = rough_output[i];
            let [p3, p4] = node.gate;
            debug.draw_line(p3, p4, [255, 0, 255, 255]);

            if p1 == p3 || p2 == p3 {
                debug.draw_line(p2, p4, [0, 255, 0, 255]);
            } else if p1 == p4 || p2 == p4 {
                debug.draw_line(p1, p3, [0, 0, 255, 255]);
            }

            let point = self.graph[node.index].center;
            debug.draw_line(last_point, point, [0, 255, 255, 255]);
            last_point = point;

            p1 = p3;
            p2 = p4;
        }

        debug.draw_line(end, last_point, [0, 255, 255, 255]);

        debug.set_current_layer(0);
    }

    pub fn debug_path(&self, debug: &mut super::DebugState, start: PositionF32, end: PositionF32) {
        let mut points = Vec::with_capacity(16);
        if !self.compute_path(start, end, &mut points) {
            return;
        }

        debug.set_current_layer(1);

        for i in 0..points.len() {
            let p1 = points[i];
            if let Some(p2) = points.get(i+1) {
                debug.draw_line(p1, *p2, [255, 0, 255, 255]);
            }
        }

        debug.set_current_layer(0);
    }

    //
    // Helpers
    //

    fn triangle_of_edge(&self, edge: usize) -> Triangle {
        Triangle((edge / 3) as u32)
    }

    fn triangle_points(&self, triangle: Triangle) -> [PositionF32; 3] {
        let triangle_index = triangle.0 as usize;
        if triangle_index == (u32::MAX as usize) {
            return [Default::default(); 3];
        }

        let edge0 = 3*triangle_index;

        match self.triangulation.as_ref() {
            None => [pos(0.0, 0.0); 3],
            Some(t) => {
                [
                    self.points[t.triangles[edge0+0]],
                    self.points[t.triangles[edge0+1]],
                    self.points[t.triangles[edge0+2]],
                ]
            },
        }
    }

}

/// Returns a **negative** value if `p1`, `p2` and `p3` occur in counterclockwise order
/// Returns a **positive** value if they occur in clockwise order
/// Returns zero is they are collinear
fn orient_point(p1: PositionF32, p2: PositionF32, p3: PositionF32) -> f32 {
    // robust-rs orients Y-axis upwards, our convention is Y downwards. This means that the interpretation of the result must be flipped
    robust::orient2d(p1.into(), p2.into(), p3.into()) as f32
}


impl StoreLoad for NavigationState {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        writer.write_array(&self.points);
        writer.write_array(&self.graph);
        self.triangulation.store(writer);
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut nav = NavigationState::default();
        nav.points = reader.read_array().to_vec();
        nav.graph = reader.read_array().to_vec();
        nav.triangulation = StoreLoad::load(reader)?;
        Ok(nav)
    }
}

impl StoreLoad for Option<Triangulation> {
    fn store(&mut self, writer: &mut crate::store::StoreWriter) {
        writer.write_bool(self.is_some());

        if let Some(tri) = self {
            writer.write_array(&tri.triangles);
            writer.write_array(&tri.halfedges);
            writer.write_array(&tri.hull);
        }
    }

    fn load(reader: &mut crate::store::StoreReader) -> Result<Self, crate::error::Error> {
        let mut triangulation = None;
        let is_some = reader.try_read_bool()?;
        if is_some {
            triangulation = Some(Triangulation {
                triangles: reader.read_array().to_vec(),
                halfedges: reader.read_array().to_vec(),
                hull: reader.read_array().to_vec(),
            });
        }

        Ok(triangulation)
    }
}

//
// Other Impl
//

impl Default for NavNodeNeighbor {
    fn default() -> Self {
        NavNodeNeighbor { 
            center: PositionF32::default(),
            segment: [PositionF32::default(); 2],
            triangle: Triangle(u32::MAX),
            distance: f32::INFINITY
        }
    }
}

impl Ord for PathComputeNode {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match other.estimated_cost_to_end.total_cmp(&self.estimated_cost_to_end) {
            std::cmp::Ordering::Equal => self.cost_to_start.total_cmp(&other.cost_to_start),
            s => s,
        }
    }
}

impl PartialOrd for PathComputeNode {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PathComputeNode {
    fn eq(&self, other: &Self) -> bool {
        self.cost_to_start == other.cost_to_start && self.estimated_cost_to_end == other.estimated_cost_to_end
    }
}

impl Eq for PathComputeNode {}
