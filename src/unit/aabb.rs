// TODO: DOUBLE CHECK IMPL TO MAKE SURE IM NOT MISSING ANY HEIGHT
use std::collections::HashMap;

use crate::rayunit::*;

const NULL_NODE: usize = 0xffffffff;

pub fn min(a: f64, b: f64) -> f64 {
    a.min(b)
}

pub fn max(a: f64, b: f64) -> f64 {
    a.max(b)
}

#[derive(Clone)]
pub struct AABB {
    pub surface_area: f64,
    pub close_corner: Vec3,
    pub far_corner: Vec3,
}

impl AABB {
    pub fn default() -> AABB {
        AABB {
            surface_area: 0.0,
            close_corner: Vec3::new(0.0, 0.0, 0.0),
            far_corner: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn new(close_corner: Vec3, far_corner: Vec3) -> AABB {
        AABB {
            surface_area: AABB::compute_surface_area(close_corner, far_corner),
            close_corner,
            far_corner,
        }
    }

    pub fn compute_surface_area(lower: Vec3, upper: Vec3) -> f64 {
        let d = upper - lower;
        2.0 * ((d.x() * d.y()) + (d.x() * d.z()) + (d.y() * d.z()))
    }

    pub fn union(first: &AABB, second: &AABB) -> AABB {
        let close_corner = Vec3::min(first.close_corner, second.close_corner);
        let far_corner = Vec3::max(first.far_corner, second.far_corner);
        AABB::new(close_corner, far_corner)
    }

    // fast slab method
    // outlined here
    // https://tavianator.com/2015/ray_box_nan.html
    pub fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> bool {
        let mut tmin = t_min;
        let mut tmax = t_max;

        let t1 = (self.close_corner.x() - r.origin().x()) / r.direction().x();
        let t2 = (self.far_corner.x() - r.origin().x()) / r.direction().x();

        let t3 = (self.close_corner.y() - r.origin().y()) / r.direction().y();
        let t4 = (self.far_corner.y() - r.origin().y()) / r.direction().y();

        let t5 = (self.close_corner.z() - r.origin().z()) / r.direction().z();
        let t6 = (self.far_corner.z() - r.origin().z()) / r.direction().z();

        tmin = max(tmin, min(t1, t2));
        tmin = max(tmin, min(t3, t4));
        tmin = max(tmin, min(t5, t6));

        tmax = min(tmax, max(t1, t2));
        tmax = min(tmax, max(t3, t4));
        tmax = min(tmax, max(t5, t6));

        tmax > tmin
    }
}

#[derive(Clone)]
pub struct Node {
    aabb: AABB,
    parent: usize,
    next: usize,
    left: usize,
    right: usize,
    particle: usize,
    height: i32,
}

impl Node {
    pub fn is_leaf(&self) -> bool {
        self.left == NULL_NODE
    }

    pub fn default() -> Node {
        let aabb = AABB::default();
        Node {
            aabb,
            parent: NULL_NODE,
            next: NULL_NODE,
            left: NULL_NODE,
            right: NULL_NODE,
            particle: NULL_NODE,
            height: -1,
        }
    }
}

pub struct Tree {
    root: usize,
    nodes: Vec<Node>,
    node_count: usize,
    capacity: usize,
    free_list: usize,
    sphere_map: HashMap<usize, usize>,
    spheres: Vec<Box<Sphere>>,
}

impl Tree {
    pub fn from(spheres: Vec<Box<Sphere>>) -> Tree {
        let mut tree = Tree::new(spheres.len() * 2);
        for sphere in spheres {
            tree.push(sphere);
        }
        tree
    }

    pub fn new(capacity: usize) -> Tree {
        let mut nodes = vec![Node::default(); capacity];

        for i in 0..capacity - 1 {
            nodes[i].next = i + 1;
            nodes[i].height = -1;
        }
        nodes[capacity - 1].next = NULL_NODE;
        nodes[capacity - 1].height = -1;

        Tree {
            root: NULL_NODE,
            nodes,
            node_count: 0,
            capacity,
            free_list: 0,
            sphere_map: HashMap::new(),
            spheres: Vec::new(),
        }
    }

    pub fn allocate_node(&mut self) -> usize {
        // no more space
        // we need to double our capacity & set up our free list
        if self.free_list == NULL_NODE {
            self.capacity *= 2;
            self.nodes.resize(self.capacity, Node::default());
            for i in self.node_count..self.capacity - 1 {
                self.nodes[i].next = i + 1;
                self.nodes[i].height = -1;
            }
            self.nodes[self.capacity - 1].next = NULL_NODE;
            self.nodes[self.capacity - 1].height = -1;
            self.free_list = self.node_count;
        }

        // set up this next free node, and return it
        let node = self.free_list;
        self.free_list = self.nodes[node].next;
        self.nodes[node].parent = NULL_NODE;
        self.nodes[node].left = NULL_NODE;
        self.nodes[node].right = NULL_NODE;
        self.nodes[node].height = 0;
        self.node_count += 1;
        node
    }

    pub fn free_node(&mut self, node: usize) {
        // we just mark this one as not in use
        self.nodes[node].next = self.free_list;
        self.nodes[node].height = -1;
        self.free_list = node;
        self.node_count -= 1;
    }

    pub fn push(&mut self, val: Box<Sphere>) {
        let idx = self.spheres.len();
        let lower_bound = val.lower_bound();
        let upper_bound = val.upper_bound();
        self.spheres.push(val);
        self.insert_object(idx, lower_bound, upper_bound)
    }

    pub fn insert_object(&mut self, object_idx: usize, lower_bound: Vec3, upper_bound: Vec3) {
        let node = self.allocate_node();
        self.nodes[node].aabb.close_corner = lower_bound;
        self.nodes[node].aabb.far_corner = upper_bound;

        self.nodes[node].aabb.surface_area = AABB::compute_surface_area(lower_bound, upper_bound);

        self.nodes[node].height = 0;

        self.insert_leaf(node);

        self.sphere_map.insert(object_idx, node);

        self.nodes[node].particle = object_idx;
    }

    pub fn remove_object(&mut self, object_idx: usize) {
        let possible = self.sphere_map.get(&object_idx).cloned();
        if possible == None {
            return;
        }
        let node = possible.unwrap();
        self.sphere_map.remove(&object_idx);

        self.remove_leaf(node);
        self.free_node(node);
    }

    pub fn insert_leaf(&mut self, leaf: usize) {
        if self.root == NULL_NODE {
            self.root = leaf;
            self.nodes[self.root].parent = NULL_NODE;
            return;
        }

        let leaf_aabb = self.nodes[leaf].aabb.clone();
        let mut idx = self.root;

        while !self.nodes[idx].is_leaf() {
            let left = self.nodes[idx].left;
            let right = self.nodes[idx].right;

            let surface_area = self.nodes[idx].aabb.surface_area;

            let combined_aabb = AABB::union(&self.nodes[idx].aabb, &leaf_aabb);
            let combined_surface_area = combined_aabb.surface_area;

            let cost = 2.0 * combined_surface_area;
            let inheritance_cost = 2.0 * (combined_surface_area - surface_area);

            let cost_left = {
                if self.nodes[left].is_leaf() {
                    let aabb = AABB::union(&leaf_aabb, &self.nodes[left].aabb);
                    aabb.surface_area + inheritance_cost
                } else {
                    let aabb = AABB::union(&leaf_aabb, &self.nodes[left].aabb);
                    let old_area = self.nodes[left].aabb.surface_area;
                    let new_area = aabb.surface_area;
                    (new_area - old_area) + inheritance_cost
                }
            };

            let cost_right = {
                if self.nodes[right].is_leaf() {
                    let aabb = AABB::union(&leaf_aabb, &self.nodes[right].aabb);
                    aabb.surface_area + inheritance_cost
                } else {
                    let aabb = AABB::union(&leaf_aabb, &self.nodes[right].aabb);
                    let old_area = self.nodes[right].aabb.surface_area;
                    let new_area = aabb.surface_area;
                    (new_area - old_area) + inheritance_cost
                }
            };

            if cost < cost_left && cost < cost_right {
                break;
            }

            if cost_left < cost_right {
                idx = left;
            } else {
                idx = right;
            }
        }

        let sibling = idx;

        let old_parent = self.nodes[sibling].parent;
        let new_parent = self.allocate_node();
        self.nodes[new_parent].parent = old_parent;
        let sibling_aabb = self.nodes[sibling].aabb.clone();
        self.nodes[new_parent].aabb = AABB::union(&leaf_aabb, &sibling_aabb);
        self.nodes[new_parent].height = self.nodes[sibling].height + 1;

        if old_parent != NULL_NODE {
            if self.nodes[old_parent].left == sibling {
                self.nodes[old_parent].left = new_parent;
            } else {
                self.nodes[old_parent].right = new_parent;
            }
        } else {
            self.root = new_parent;
        }
        self.nodes[new_parent].left = sibling;
        self.nodes[new_parent].right = leaf;
        self.nodes[sibling].parent = new_parent;
        self.nodes[leaf].parent = new_parent;

        idx = self.nodes[leaf].parent;
        while idx != NULL_NODE {
            idx = self.balance(idx);
            let left = self.nodes[idx].left;
            let right = self.nodes[idx].right;

            self.nodes[idx].height = 1 + self.nodes[left].height.max(self.nodes[right].height);

            let left_aabb = self.nodes[left].aabb.clone();
            let right_aabb = self.nodes[right].aabb.clone();

            self.nodes[idx].aabb = AABB::union(&left_aabb, &right_aabb);

            idx = self.nodes[idx].parent;
        }
    }

    pub fn remove_leaf(&mut self, leaf: usize) {
        if leaf == self.root {
            self.root = NULL_NODE;
            return;
        }

        let parent = self.nodes[leaf].parent;
        let grandparent = self.nodes[parent].parent;
        let sibling = {
            if self.nodes[parent].left == leaf {
                self.nodes[parent].right
            } else {
                self.nodes[parent].left
            }
        };

        if grandparent != NULL_NODE {
            if self.nodes[grandparent].left == parent {
                self.nodes[grandparent].left = sibling;
            } else {
                self.nodes[grandparent].right = sibling;
            }

            self.nodes[sibling].parent = grandparent;
            self.free_node(parent);

            let mut idx = grandparent;
            while idx != NULL_NODE {
                idx = self.balance(idx);

                // refit the ancestors to contain their children
                let left = self.nodes[idx].left;
                let right = self.nodes[idx].right;

                let left_aabb = self.nodes[left].aabb.clone();
                let right_aabb = self.nodes[right].aabb.clone();

                self.nodes[idx].aabb = AABB::union(&left_aabb, &right_aabb);
                self.nodes[idx].height = 1 + self.nodes[left].height.max(self.nodes[right].height);

                idx = self.nodes[idx].parent;
            }
        } else {
            self.root = sibling;
            self.nodes[sibling].parent = NULL_NODE;
            self.free_node(parent);
        }
    }

    pub fn balance(&mut self, node: usize) -> usize {
        if self.nodes[node].is_leaf() || self.nodes[node].height < 2 {
            return node;
        }

        let left = self.nodes[node].left;
        let right = self.nodes[node].right;

        let current_balance = self.nodes[right].height - self.nodes[left].height;

        // unbalanced to the right
        if current_balance > 1 {
            let right_left = self.nodes[right].left;
            let right_right = self.nodes[right].right;

            //swap node and its right hand node
            self.nodes[right].left = node;
            self.nodes[right].parent = self.nodes[node].parent;
            self.nodes[node].parent = right;

            if self.nodes[right].parent != NULL_NODE {
                let right_parent = self.nodes[right].parent;
                if self.nodes[self.nodes[right].parent].left == node {
                    self.nodes[right_parent].left = right;
                } else {
                    self.nodes[right_parent].right = right;
                }
            } else {
                self.root = right;
            }

            // rotation
            if self.nodes[right_left].height > self.nodes[right_right].height {
                self.nodes[right].right = right_left;
                self.nodes[node].right = right_right;
                self.nodes[right_right].parent = node;

                let left_aabb = self.nodes[left].aabb.clone();
                let right_right_aabb = self.nodes[right_right].aabb.clone();
                self.nodes[node].aabb = AABB::union(&left_aabb, &right_right_aabb);
                self.nodes[node].height =
                    1 + self.nodes[left].height.max(self.nodes[right_right].height);

                let node_aabb = self.nodes[node].aabb.clone();
                let right_left_aabb = self.nodes[right_left].aabb.clone();
                self.nodes[right].aabb = AABB::union(&node_aabb, &right_left_aabb);
                self.nodes[right].height =
                    1 + self.nodes[node].height.max(self.nodes[right_left].height);
            } else {
                self.nodes[right].right = right_right;
                self.nodes[node].right = right_left;
                self.nodes[right_left].parent = node;

                let left_aabb = self.nodes[left].aabb.clone();
                let right_left_aabb = self.nodes[right_left].aabb.clone();
                self.nodes[node].aabb = AABB::union(&left_aabb, &right_left_aabb);
                self.nodes[node].height =
                    1 + self.nodes[left].height.max(self.nodes[right_left].height);

                let node_aabb = self.nodes[node].aabb.clone();
                let right_right_aabb = self.nodes[right_right].aabb.clone();
                self.nodes[right].aabb = AABB::union(&node_aabb, &right_right_aabb);
                self.nodes[right].height =
                    1 + self.nodes[node].height.max(self.nodes[right_right].height);
            }

            return right;
        }

        // unbalanced to the left
        if current_balance < -1 {
            let left_left = self.nodes[left].left;
            let left_right = self.nodes[left].right;

            // swap node and its left hand node
            self.nodes[left].left = node;
            self.nodes[left].parent = self.nodes[node].parent;
            self.nodes[node].parent = left;

            if self.nodes[left].parent != NULL_NODE {
                let left_parent = self.nodes[left].parent;
                if self.nodes[self.nodes[left].parent].left == node {
                    self.nodes[left_parent].left = left;
                } else {
                    self.nodes[left_parent].right = left;
                }
            } else {
                self.root = left;
            }

            // rotation
            if self.nodes[left_left].height > self.nodes[left_right].height {
                if self.nodes[left_left].height > self.nodes[left_right].height {
                    self.nodes[left].right = left_left;
                    self.nodes[node].left = left_right;
                    self.nodes[left_right].parent = node;

                    let right_aabb = self.nodes[right].aabb.clone();
                    let left_right_aabb = self.nodes[left_right].aabb.clone();
                    self.nodes[node].aabb = AABB::union(&right_aabb, &left_right_aabb);
                    self.nodes[node].height =
                        1 + self.nodes[right].height.max(self.nodes[left_right].height);

                    let node_aabb = self.nodes[node].aabb.clone();
                    let left_left_aabb = self.nodes[left_left].aabb.clone();
                    self.nodes[left].aabb = AABB::union(&node_aabb, &left_left_aabb);
                    self.nodes[left].height =
                        1 + self.nodes[node].height.max(self.nodes[left_left].height);
                } else {
                    self.nodes[left].right = left_right;
                    self.nodes[node].left = left_left;
                    self.nodes[left_left].parent = node;

                    let right_aabb = self.nodes[right].aabb.clone();
                    let left_left_aabb = self.nodes[left_left].aabb.clone();
                    self.nodes[node].aabb = AABB::union(&right_aabb, &left_left_aabb);
                    self.nodes[node].height =
                        1 + self.nodes[right].height.max(self.nodes[left_left].height);

                    let node_aabb = self.nodes[node].aabb.clone();
                    let left_right_aabb = self.nodes[left_right].aabb.clone();
                    self.nodes[left].aabb = AABB::union(&node_aabb, &left_right_aabb);
                    self.nodes[left].height =
                        1 + self.nodes[node].height.max(self.nodes[left_right].height);
                }

                return left;
            }
        }

        node
    }

    pub fn step_frame(&mut self, time_delta: f64) -> Tree {
        let old_spheres = self.spheres.clone();
        let new_spheres: Vec<Box<Sphere>> = old_spheres
            .iter()
            .map(|s| Box::new(s.step_frame(time_delta)))
            .collect();
        Tree::from(new_spheres)
    }
}

impl Hit for Tree {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        if self.root == NULL_NODE {
            return None;
        }

        let mut stack = vec![self.root];
        let mut tmp_rec = None;
        let mut current_closest = t_max;

        while stack.len() > 0 {
            let top = stack.pop();

            assert!(top != None);

            let cur_idx = top.unwrap();
            // we hit this container
            let cur_node = &self.nodes[cur_idx];
            if cur_node.aabb.hit(r, t_min, current_closest) {
                // if its a leaf we check the related object
                if cur_node.is_leaf() {
                    let sphere_idx = cur_node.particle;
                    let obj = &self.spheres[sphere_idx];
                    if let Some(rec) = obj.hit(r, t_min, current_closest) {
                        current_closest = rec.t;
                        tmp_rec = Some(rec);
                    }
                }
                // else we add the children
                else {
                    if cur_node.left != NULL_NODE {
                        stack.push(cur_node.left);
                    }
                    if cur_node.right != NULL_NODE {
                        stack.push(cur_node.right);
                    }
                }
            }
        }

        tmp_rec
    }
}
