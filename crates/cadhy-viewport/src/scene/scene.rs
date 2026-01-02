use std::collections::HashMap;
use glam::Vec3;

use super::object::{ObjectId, SceneObject};

/// The scene graph containing all objects
pub struct Scene {
    pub objects: HashMap<ObjectId, SceneObject>,
    /// Ordered list for rendering (front to back)
    render_order: Vec<ObjectId>,
    /// Currently selected objects
    pub selection: Vec<ObjectId>,
    /// Scene bounds (for camera framing)
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            render_order: Vec::new(),
            selection: Vec::new(),
            bounds_min: Vec3::ZERO,
            bounds_max: Vec3::ZERO,
        }
    }

    /// Add an object to the scene
    pub fn add(&mut self, object: SceneObject) -> ObjectId {
        let id = object.id;
        self.objects.insert(id, object);
        self.render_order.push(id);
        self.update_bounds();
        id
    }

    /// Remove an object from the scene
    pub fn remove(&mut self, id: ObjectId) -> Option<SceneObject> {
        self.render_order.retain(|&obj_id| obj_id != id);
        self.selection.retain(|&obj_id| obj_id != id);
        let obj = self.objects.remove(&id);
        self.update_bounds();
        obj
    }

    /// Get object by ID
    pub fn get(&self, id: ObjectId) -> Option<&SceneObject> {
        self.objects.get(&id)
    }

    /// Get mutable object by ID
    pub fn get_mut(&mut self, id: ObjectId) -> Option<&mut SceneObject> {
        self.objects.get_mut(&id)
    }

    /// Iterate over visible objects in render order
    pub fn visible_objects(&self) -> impl Iterator<Item = &SceneObject> {
        self.render_order
            .iter()
            .filter_map(|id| self.objects.get(id))
            .filter(|obj| obj.visible)
    }

    /// Select an object
    pub fn select(&mut self, id: ObjectId, add_to_selection: bool) {
        if !add_to_selection {
            // Deselect all
            for obj in self.objects.values_mut() {
                obj.selected = false;
            }
            self.selection.clear();
        }

        if let Some(obj) = self.objects.get_mut(&id) {
            obj.selected = true;
            if !self.selection.contains(&id) {
                self.selection.push(id);
            }
        }
    }

    /// Deselect all objects
    pub fn deselect_all(&mut self) {
        for obj in self.objects.values_mut() {
            obj.selected = false;
        }
        self.selection.clear();
    }

    /// Find object by pick color
    pub fn find_by_pick_color(&self, color: [u8; 3]) -> Option<ObjectId> {
        self.objects
            .values()
            .find(|obj| {
                obj.pick_color[0] == color[0]
                    && obj.pick_color[1] == color[1]
                    && obj.pick_color[2] == color[2]
            })
            .map(|obj| obj.id)
    }

    /// Update scene bounds
    fn update_bounds(&mut self) {
        if self.objects.is_empty() {
            self.bounds_min = Vec3::ZERO;
            self.bounds_max = Vec3::ZERO;
            return;
        }

        self.bounds_min = Vec3::splat(f32::MAX);
        self.bounds_max = Vec3::splat(f32::MIN);

        for obj in self.objects.values() {
            let pos = obj.transform.position;
            self.bounds_min = self.bounds_min.min(pos);
            self.bounds_max = self.bounds_max.max(pos);
        }
    }

    /// Get scene center
    pub fn center(&self) -> Vec3 {
        (self.bounds_min + self.bounds_max) * 0.5
    }

    /// Get scene radius
    pub fn radius(&self) -> f32 {
        (self.bounds_max - self.bounds_min).length() * 0.5
    }

    /// Number of objects
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    /// Is scene empty
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    /// Clear all objects
    pub fn clear(&mut self) {
        self.objects.clear();
        self.render_order.clear();
        self.selection.clear();
        self.bounds_min = Vec3::ZERO;
        self.bounds_max = Vec3::ZERO;
    }

    /// Get list of selected object IDs
    pub fn selected(&self) -> Vec<ObjectId> {
        self.selection.clone()
    }

    /// Iterate over all objects
    pub fn objects(&self) -> impl Iterator<Item = &SceneObject> {
        self.render_order
            .iter()
            .filter_map(|id| self.objects.get(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_add_remove() {
        let mut scene = Scene::new();
        let obj = SceneObject::new("Test");
        let id = scene.add(obj);

        assert_eq!(scene.len(), 1);
        assert!(scene.get(id).is_some());

        scene.remove(id);
        assert!(scene.is_empty());
    }

    #[test]
    fn test_selection() {
        let mut scene = Scene::new();
        let obj1 = SceneObject::new("Obj1");
        let obj2 = SceneObject::new("Obj2");
        let id1 = scene.add(obj1);
        let id2 = scene.add(obj2);

        scene.select(id1, false);
        assert_eq!(scene.selection.len(), 1);
        assert!(scene.get(id1).unwrap().selected);

        scene.select(id2, true);
        assert_eq!(scene.selection.len(), 2);

        scene.deselect_all();
        assert!(scene.selection.is_empty());
    }
}