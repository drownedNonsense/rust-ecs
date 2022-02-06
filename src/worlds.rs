//#########################
// D E P E N D E N C I E S
//#########################

    use std::collections::HashMap;
    use std::any::TypeId;
    use std::rc::Rc;
    use std::cell::RefCell;

    use crate::components::{Component, ComponentCell, ComponentColumn, BitMask, StaticComponentId};
    use crate::entities::{Entity, EntityBuilder, EntityId};
    use crate::queries::QueryBuilder;


//#######################
// D E F I N I T I O N S
//#######################

    pub struct World<B: BitMask, S: StaticComponentId> {
        components: Vec<TypeId>,
        component_columns: HashMap<B, Box<dyn ComponentColumn>>,
        static_components: HashMap<S, Box<dyn ComponentCell>>,
        entities: HashMap<Entity, B>,
        next_entity_id: EntityId,
    } // struct World


    pub struct WorldBuilder<B: BitMask, S: StaticComponentId> {
        components: Vec<TypeId>,
        component_count: usize,
        component_columns: HashMap<B, Box<dyn ComponentColumn>>,
        static_components: HashMap<S, Box<dyn ComponentCell>>,
    } // struct WorldBuilder


//###############################
// I M P L E M E N T A T I O N S
//###############################

    impl<B: BitMask, S: StaticComponentId> World<B, S> {
        fn new(
            components: Vec<TypeId>,
            component_columns: HashMap<B, Box<dyn ComponentColumn>>,
            static_components: HashMap<S, Box<dyn ComponentCell>>
        ) -> Self {
            World { components, component_columns, static_components, entities: HashMap::new(), next_entity_id: 0 }
        } // fn new'


        pub fn builder() -> WorldBuilder<B, S> {
            WorldBuilder::<B, S>::new()
        } // fn builder()


        pub(crate) fn component_bit_mask<C: Component>(&self) -> B {
            self.components
                .iter()
                .enumerate()
                .find_map(|(index, id)| {
                    return match id == &TypeId::of::<C>() {
                        true  => Some(B::bit_mask(index)),
                        false => None,
                    } // return ..
                }).expect("Attempted to get a component bit mask that was not registered!")
        } // fn component_bit_mask()


        pub(crate) fn add_component_to_entity_builder<C: Component>(&mut self, component: C, entity: Entity, entity_bit_mask: &mut B) {
            let bit_mask = self.component_bit_mask::<C>();
            *entity_bit_mask |= bit_mask;
            self.component_columns
                .get_mut(&bit_mask)
                .expect("Attempted to find a component column that was not registered!")
                .as_any_mut()
                .downcast_mut::<HashMap<Entity, Rc<RefCell<C>>>>()
                .expect("Failed to downcast a component column!")
                .insert(entity, Rc::new(RefCell::new(component)));
        } // fn add_component_to_entity_builder()


        pub(crate) fn add_shared_component_to_entity_builder<C: Component>(&mut self, component: &Rc<RefCell<C>>, entity: Entity, entity_bit_mask: &mut B) {
            let bit_mask = self.component_bit_mask::<C>();
            *entity_bit_mask |= bit_mask;
            self.component_columns
                .get_mut(&bit_mask)
                .expect("Attempted to find a component column that was not registered!")
                .as_any_mut()
                .downcast_mut::<HashMap<Entity, Rc<RefCell<C>>>>()
                .expect("Failed to downcast a component column!")
                .insert(entity, component.clone());
        } // fn add_shared_component_to_entity_builder()


        pub fn add_component_to_entity<C: Component>(&mut self, component: C, entity: Entity) {
            let bit_mask = self.component_bit_mask::<C>();
            *self.entities
                .get_mut(&entity)
                .expect("Attempted to find an entity that was not registered!") |= bit_mask;
            self.component_columns
                .get_mut(&bit_mask)
                .expect("Attempted to find a component column that was not registered!")
                .as_any_mut()
                .downcast_mut::<HashMap<Entity, Rc<RefCell<C>>>>()
                .expect("Failed to downcast a component column!")
                .insert(entity, Rc::new(RefCell::new(component)));
        } // fn add_component_to_entity()


        pub fn add_empty_component_to_entity<C: Component + Default>(&mut self, entity: Entity) {
            let bit_mask = self.component_bit_mask::<C>();
            *self.entities
                .get_mut(&entity)
                .expect("Attempted to find an entity that was not registered!") |= bit_mask;
            self.component_columns
                .get_mut(&bit_mask)
                .expect("Attempted to find a component column that was not registered!")
                .as_any_mut()
                .downcast_mut::<HashMap<Entity, Rc<RefCell<C>>>>()
                .expect("Failed to downcast a component column!")
                .insert(entity, Rc::new(RefCell::new(C::default())));
        } // fn add_empty_component_to_entity()


        pub fn add_empty_component_to_entity_group<C: Component + Default>(&mut self, entity_group: &[Entity]) {
            let bit_mask = self.component_bit_mask::<C>();
            let component_column = self.component_columns
                .get_mut(&bit_mask)
                .expect("Attempted to find a component column that was not registered!")
                .as_any_mut()
                .downcast_mut::<HashMap<Entity, Rc<RefCell<C>>>>()
                .expect("Failed to downcast a component column!");
            entity_group
                .iter()
                .for_each(|entity| {

                    component_column.insert(*entity, Rc::new(RefCell::new(C::default())));
                    *self.entities
                        .get_mut(entity)
                        .expect("Attempted to find an entity that was not registered!") |= bit_mask;

                }); // for_each()
        } // fn add_empty_component_to_entity_group()


        pub fn add_shared_component_to_entity<C: Component>(&mut self, component: &Rc<RefCell<C>>, entity: Entity) {
            let bit_mask = self.component_bit_mask::<C>();
            *self.entities
                .get_mut(&entity)
                .expect("Attempted to find an entity that was not registered!") |= bit_mask;
            self.component_columns
                .get_mut(&bit_mask)
                .expect("Attempted to find a component column that was not registered!")
                .as_any_mut()
                .downcast_mut::<HashMap<Entity, Rc<RefCell<C>>>>()
                .expect("Failed to downcast a component column!")
                .insert(entity, component.clone());
        } // fn add_shared_component_to_entity()


        pub fn add_shared_component_to_entity_group<C: Component>(&mut self, component: &Rc<RefCell<C>>, entity_group: &[Entity]) {
            let bit_mask = self.component_bit_mask::<C>();
            let component_column = self.component_columns
                .get_mut(&bit_mask)
                .expect("Attempted to find a component column that was not registered!")
                .as_any_mut()
                .downcast_mut::<HashMap<Entity, Rc<RefCell<C>>>>()
                .expect("Failed to downcast a component column!");
            entity_group
                .iter()
                .for_each(|entity| {

                    component_column.insert(*entity, component.clone());
                    *self.entities
                        .get_mut(entity)
                        .expect("Attempted to find an entity that was not registered!") |= bit_mask;

                }); // for_each()
        } // fn add_shared_component_to_entity_group()


        pub fn get_entity_component<C: Component>(&self, entity: Entity) -> Option<&Rc<RefCell<C>>> {
            self.component_columns
                .get(&self.component_bit_mask::<C>())
                .expect("Attempted to find a component column that was not registered!")
                .as_any()
                .downcast_ref::<HashMap<Entity, Rc<RefCell<C>>>>()
                .expect("Failed to downcast a component column!")
                .get(&entity)
        } // fn get_entity_component()


        pub fn get_entity_group_component<C: Component>(&self, entity_group: &[Entity]) -> Vec<Option<&Rc<RefCell<C>>>> {
            let component_column = self.component_columns
                .get(&self.component_bit_mask::<C>())
                .expect("Attempted to find a component column that was not registered!")
                .as_any()
                .downcast_ref::<HashMap<Entity, Rc<RefCell<C>>>>()
                .expect("Failed to downcast a component column!");
            entity_group
                .iter()
                .map(|entity| component_column.get(entity))
                .collect()
        } // fn get_entity_group_component()


        pub fn delete_entity_component<C: Component>(&mut self, entity: Entity) {
            let bit_mask = self.component_bit_mask::<C>();
            *self.entities
                .get_mut(&entity)
                .expect("Attempted to find an entity that was not registered!") &= !bit_mask;  
            self.component_columns
                .get_mut(&bit_mask)
                .unwrap()
                .remove_entity(&entity);
        } // fn delete_entity_component()


        pub fn delete_entity_group_component<C: Component>(&mut self, entity_group: &[Entity]) {
            let bit_mask = self.component_bit_mask::<C>();
            let component_column = self.component_columns
                .get_mut(&bit_mask)
                .expect("Attempted to find a component column that was not registered!")
                .as_any_mut()
                .downcast_mut::<HashMap<Entity, Rc<RefCell<C>>>>()
                .expect("Failed to downcast a component column!");
            entity_group
                .iter()
                .for_each(|entity| {

                    component_column.remove_entity(entity);
                    *self.entities
                        .get_mut(entity)
                        .expect("Attempted to find an entity that was not registered!") &= !bit_mask;

                }); // for_each()
        } // fn delete_entity_group_component()


        pub(crate) fn add_entity(&mut self, entity: Entity, entity_bit_mask: B) {
            self.entities.insert(entity, entity_bit_mask);
        } // fn add_entity()


        pub(crate) fn get_component_column<C: Component>(&self) -> &HashMap<Entity, Rc<RefCell<C>>> {
            self.component_columns
                .get(&self.component_bit_mask::<C>())
                .expect("Attempted to find a component column that was not registered!")
                .as_any()
                .downcast_ref::<HashMap<Entity, Rc<RefCell<C>>>>()
                .expect("Failed to downcast a component column!")
        } // fn get_component_column()


        pub fn get_static_component<C: Component>(&self, id: S) -> &Rc<RefCell<C>> {
            self.static_components
                .get(&id)
                .expect("Attempted to find a static component that was not registered!")
                .as_any()
                .downcast_ref::<Rc<RefCell<C>>>()
                .expect("Failed to downcast a static component!")
        } // fn get_static_component()


        pub fn set_static_component<C: Component>(&mut self, id: S, component: C) {
            self.static_components.remove(&id);
            self.static_components.insert(id, Box::new(Rc::new(RefCell::new(component))));
        } // fn set_static_component()


        pub fn set_shared_static_component<C: Component>(&mut self, id: S, component: Rc<RefCell<C>>) {
            self.static_components.remove(&id);
            self.static_components.insert(id, Box::new(component));
        } // fn set_shared_static_component()


        pub(crate) fn get_entities(&self, bit_mask_filter: B) -> Vec<Entity> {
            self.entities
                .iter()
                .filter(|(_, bit_mask)| **bit_mask & bit_mask_filter == bit_mask_filter)
                .map(|(entity, _)| *entity)
                .collect()
        } // fn get_entities()


        pub fn delete_entity(&mut self, entity: &Entity) {

            let entity_mask = self.entities.get(entity).expect("Attempted to delete an entity that was not registered!");
            self.component_columns
                .iter_mut()
                .filter(|(bit_mask, _)| *entity_mask & **bit_mask == **bit_mask)
                .for_each(|(_, component_column)| {
                    component_column.remove_entity(entity);
                }); // iter_mut()
            
            self.entities.remove(entity);
            
        } // fn delete_entity()


        pub fn new_entity(&mut self) -> EntityBuilder<B, S> {
            self.next_entity_id += 1;
            EntityBuilder::new(self.next_entity_id - 1, self)
        } // fn new_entity()


        pub fn new_query(&self) -> QueryBuilder<B, S> {
            QueryBuilder::new(self)
        } // fn new_query()
    } // impl World


    impl<B: BitMask, S: StaticComponentId> WorldBuilder<B, S> {
        fn new() -> Self {
            WorldBuilder {
                components: Vec::new(),
                component_count: 0,
                component_columns: HashMap::new(),
                static_components: HashMap::new(),
            } // WorldBuilder
        } // fn new()


        pub fn with_empty_static_component<C: Component + Default>(mut self, id: S) -> Self {
            match self.static_components.contains_key(&id) {
                true =>  { println!("The static component no.{} has been discarded as it was already registered!", self.static_components.len() ) },
                false => { self.static_components.insert(id, Box::new(Rc::new(RefCell::new(C::default())))); },
            } // match ..

            self
        } // fn with_empty_static_component()


        pub fn with_static_component<C: Component>(mut self, id: S, component: C) -> Self {
            match self.static_components.contains_key(&id) {
                true =>  { println!("The static component no.{} has been discarded as it was already registered!", self.static_components.len() ) },
                false => { self.static_components.insert(id, Box::new(Rc::new(RefCell::new(component)))); },
            } // match ..

            self
        } // fn with_static_component()


        pub fn with_shared_static_component<C: Component>(mut self, id: S, component: &Rc<RefCell<C>>) -> Self {
            match self.static_components.contains_key(&id) {
                true =>  { println!("The static component no.{} has been discarded as it was already registered!", self.static_components.len() ) },
                false => { self.static_components.insert(id, Box::new(component.clone())); },
            } // match ..

            self
        } // fn with_static_component()


        pub fn with_component<C: Component>(mut self) -> Self {
            match self.components.contains(&TypeId::of::<C>()) {
                true =>  { println!("The component no.{} has been discarded as it was already registered!", self.component_count ) },
                false => {
                    self.components.push(TypeId::of::<C>());
                    self.component_columns.insert(B::bit_mask(self.component_count), Box::new(HashMap::<Entity, Rc<RefCell<C>>>::new()));
                    self.component_count += 1;
                }, // false
            } // match ..

            self
        } // fn with_component()


        pub fn build(self) -> World<B, S> {
            World::new(self.components, self.component_columns, self.static_components)
        } // fn build()
    } // impl WorldBuilder
