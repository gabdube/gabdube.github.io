mod delaunator;
use delaunator::{Triangulation, Point, triangulate};

use zerocopy_derive::{FromBytes, Immutable, IntoBytes};
use crate::shared::{PositionF32, pos};
use crate::store::StoreLoad;
use super::GameData;

/// The identifier of a triangle in the navmesh
#[derive(Copy, Clone, PartialEq, Eq, Default, FromBytes, Immutable, IntoBytes)]
pub struct Triangle(u32);

#[derive(Copy, Clone)]
struct StepState {
    target: PositionF32,
    edge: usize,
    done: bool,
}

#[derive(Copy, Clone, FromBytes, Immutable, IntoBytes)]
struct NavNodeNeighbor {
    pub triangle: Triangle,
    pub distance: f32,
}

#[derive(Copy, Clone, Default, FromBytes, Immutable, IntoBytes)]
struct NavNode {
    pub triangle: Triangle,
    pub n0: NavNodeNeighbor,
    pub n1: NavNodeNeighbor,
    pub n2: NavNodeNeighbor,
}

/// 2D pathfinding state
#[derive(Default)]
pub struct NavigationState {
    points: Vec<Point>,
    triangulation: Option<Triangulation>,
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

    pub fn rebuild_navmesh(data: &mut GameData) {
        let nav = &mut data.navigation;

        nav.clear();

        Self::terrain_points(&data.terrain, &mut nav.points);
        Self::sprites_collision_points(&data.world, &mut nav.points);
        nav.triangulation = Some(triangulate(&nav.points));

        nav.generate_nav_graph();
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

    fn sprites_collision_points(world: &super::World, points: &mut Vec<Point>) {
        let mut sprites = world.sprites_with_collisions();
        for (_, (sprite, _)) in sprites.iter() {
            let aabb = sprite.rect();
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

        let edges_count = triangulation.triangles.len();
        let mut i = 0;

        while i < edges_count {
            let mut node = NavNode::default();
            node.triangle = self.triangle_of_edge(i);

            let mut p0 = self.points[triangulation.triangles[i]];
            let mut p1 = self.points[triangulation.triangles[i+1]];
            let mut p2 = self.points[triangulation.triangles[i+2]];
            let center_start = pos((p0.x + p1.x + p2.x) / 3.0, (p0.y + p1.y + p2.y) / 3.0);
            let mut center_stop;

            let mut e3;
            let mut e4;
            let mut e5;

            e3 = triangulation.halfedges[i];
            if e3 != usize::MAX && e3 > i {
                e4 = delaunator::next_halfedge(e3);
                e5 = delaunator::next_halfedge(e4);
                p0 = self.points[triangulation.triangles[e3]];
                p1 = self.points[triangulation.triangles[e4]];
                p2 = self.points[triangulation.triangles[e5]];
                center_stop = pos((p0.x + p1.x + p2.x) / 3.0, (p0.y + p1.y + p2.y) / 3.0);

                node.n0.triangle = self.triangle_of_edge(e3);
                node.n0.distance = center_start.distance(center_stop);
            }

            e3 = triangulation.halfedges[i+1];
            if e3 != usize::MAX && e3 > i {
                e4 = delaunator::next_halfedge(e3);
                e5 = delaunator::next_halfedge(e4);
                p0 = self.points[triangulation.triangles[e3]];
                p1 = self.points[triangulation.triangles[e4]];
                p2 = self.points[triangulation.triangles[e5]];
                center_stop = pos((p0.x + p1.x + p2.x) / 3.0, (p0.y + p1.y + p2.y) / 3.0);
                
                node.n1.triangle = self.triangle_of_edge(e3);
                node.n1.distance = center_start.distance(center_stop);
            }

            e3 = triangulation.halfedges[i+2];
            if e3 != usize::MAX && e3 > i {
                e4 = delaunator::next_halfedge(e3);
                e5 = delaunator::next_halfedge(e4);
                p0 = self.points[triangulation.triangles[e3]];
                p1 = self.points[triangulation.triangles[e4]];
                p2 = self.points[triangulation.triangles[e5]];
                center_stop = pos((p0.x + p1.x + p2.x) / 3.0, (p0.y + p1.y + p2.y) / 3.0);
                
                node.n2.triangle = self.triangle_of_edge(e3);
                node.n2.distance = center_start.distance(center_stop);
            }

            self.graph.push(node);

            i += 3;
        }
    }

    //
    // Pathfinding
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
        let mut i = 0;

        while i < triangle_count {
            let node = self.graph[i];
            let n0 = node.n0.triangle.0;
            let n1 = node.n1.triangle.0;
            let n2 = node.n2.triangle.0;

            let [p0, p1, p2] = self.triangle_points(node.triangle);
            let center_start = pos((p0.x + p1.x + p2.x) / 3.0, (p0.y + p1.y + p2.y) / 3.0);
            debug.draw_point(center_start, 3.0, [255, 255, 0, 255]);

            if n0 != u32::MAX && n0 > (i as u32) {
                let [p0, p1, p2] = self.triangle_points(node.n0.triangle);
                let center_stop = pos((p0.x + p1.x + p2.x) / 3.0, (p0.y + p1.y + p2.y) / 3.0);
                debug.draw_line(center_start, center_stop, [255, 255, 0, 255]);
            }

            if n1 != u32::MAX && n1 > (i as u32) {
                let [p0, p1, p2] = self.triangle_points(node.n1.triangle);
                let center_stop = pos((p0.x + p1.x + p2.x) / 3.0, (p0.y + p1.y + p2.y) / 3.0);
                debug.draw_line(center_start, center_stop, [255, 255, 0, 255]);
            }
          
            if n2 != u32::MAX && n2 > (i as u32) {
                let [p0, p1, p2] = self.triangle_points(node.n2.triangle);
                let center_stop = pos((p0.x + p1.x + p2.x) / 3.0, (p0.y + p1.y + p2.y) / 3.0);
                debug.draw_line(center_start, center_stop, [255, 255, 0, 255]);
            }

            i += 1;
        }
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

impl Default for NavNodeNeighbor {
    fn default() -> Self {
        NavNodeNeighbor { triangle: Triangle(u32::MAX), distance: f32::INFINITY }
    }
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
