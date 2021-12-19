#![allow(dead_code)]

use std::cell::{Ref, RefCell, RefMut};

pub trait ComponentVec {
    fn push_none(&mut self);
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub struct World {
    entities_count: usize,
    component_vecs: Vec<Box<dyn ComponentVec>>,
}

impl World {
    pub fn new() -> Self {
        World {
            entities_count: 0,
            component_vecs: Vec::new(),
        }
    }

    pub fn new_entity(&mut self) -> usize {
        // Create id;
        let entity_id = self.entities_count;

        // Initialise components for entity to be none
        for component_vec in self.component_vecs.iter_mut() {
            component_vec.push_none();
        }

        // Increment the number of entities
        self.entities_count += 1;

        // Return created entity id
        entity_id
    }

    pub fn add_component_to_entity<ComponentType: 'static>(
        &mut self,
        entity: usize,
        component: ComponentType,
    ) {
        // Iterate through component vector to find the component vec that matches the component type
        // and set the component for the entity as the supplied component
        for component_vec in self.component_vecs.iter_mut() {
            if let Some(component_vec) = component_vec
                .as_any_mut()
                .downcast_mut::<RefCell<Vec<Option<ComponentType>>>>()
            {
                component_vec.get_mut()[entity] = Some(component);
                return;
            }
        }

        // If the component vector does not already exists for the component type it needs to
        // be created
        let mut new_component_vec: Vec<Option<ComponentType>> =
            Vec::with_capacity(self.entities_count);

        // Set the component for all other entities to zero;
        for _ in 0..self.entities_count {
            new_component_vec.push(None);
        }

        // Set the component for the entity as the supplied component
        new_component_vec[entity] = Some(component);
        self.component_vecs
            .push(Box::new(RefCell::new(new_component_vec)))
    }

    pub fn borrow_component_vec<ComponentType: 'static>(
        &self,
    ) -> Option<Ref<Vec<Option<ComponentType>>>> {
        for component_vec in self.component_vecs.iter() {
            if let Some(component_vec) = component_vec
                .as_any()
                .downcast_ref::<RefCell<Vec<Option<ComponentType>>>>()
            {
                return Some(component_vec.borrow());
            }
        }
        None
    }

    pub fn borrow_component_vec_mut<ComponentType: 'static>(
        &self,
    ) -> Option<RefMut<Vec<Option<ComponentType>>>> {
        for component_vec in self.component_vecs.iter() {
            if let Some(component_vec) = component_vec
                .as_any()
                .downcast_ref::<RefCell<Vec<Option<ComponentType>>>>()
            {
                return Some(component_vec.borrow_mut());
            }
        }
        None
    }

    pub fn update(&mut self, systems: &[Box<System>]) {
        for system in systems {
            system(self)
        }
    }
}

impl<T: 'static> ComponentVec for RefCell<Vec<Option<T>>> {
    fn push_none(&mut self) {
        self.get_mut().push(None);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}

pub type System = dyn Fn(&mut World);

#[cfg(test)]
mod tests {
    use crate::{System, World};

    #[test]
    fn non_update() {
        struct Health(i32);
        struct Name(&'static str);

        let mut world = World::new();
        let entity = world.new_entity();
        world.add_component_to_entity(entity, Name("Somebody"));
        world.add_component_to_entity(entity, Health(10));

        let mut healths = world.borrow_component_vec_mut::<Health>().unwrap();
        let mut names = world.borrow_component_vec_mut::<Name>().unwrap();
        let zip = healths.iter_mut().zip(names.iter_mut());

        for (health, name) in
            zip.filter_map(|(health, name)| Some((health.as_mut()?, name.as_mut()?)))
        {
            health.0 = 100;

            println!("{} has been healed to {}", name.0, health.0);
        }
    }

    #[test]
    fn update() {
        struct Health(i32);
        struct Name(&'static str);

        let mut world = World::new();
        let entity = world.new_entity();
        world.add_component_to_entity(entity, Name("Somebody"));
        world.add_component_to_entity(entity, Health(10));

        let heal_system: Box<System> = Box::new(|world| {
            let mut healths = world.borrow_component_vec_mut::<Health>().unwrap();
            let names = world.borrow_component_vec_mut::<Name>().unwrap();
            let zip = healths.iter_mut().zip(names.iter());

            for (health, name) in
                zip.filter_map(|(health, name)| Some((health.as_mut()?, name.as_ref()?)))
            {
                health.0 = 100;
                println!("{} healed to 100!", name.0)
            }
        });

        let rename_system: Box<System> = Box::new(|world| {
            let mut names = world.borrow_component_vec_mut::<Name>().unwrap();
            for name in names.iter_mut().filter_map(|name| Some(name.as_mut()?)) {
                let tmp = name.0;
                name.0 = "New name";
                println!("\"{}\" is now \"{}\"", tmp, name.0)
            }
        });

        let systems = vec![heal_system, rename_system];

        world.update(&systems);
    }
}
