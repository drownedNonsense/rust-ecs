//#########################
// D E P E N D E N C I E S
//#########################

    use std::collections::HashMap;
    use std::any::TypeId;
    use std::rc::Rc;
    use std::cell::RefCell;
    use std::hash::Hash;

    use crate::components::{Component, ComponentCell, ComponentColumn};
    use crate::entities::{Entity, EntityBuilder, EntityId};
    use crate::queries::QueryBuilder;

    use rust_utils::BitSequence;


//#######################
// D E F I N I T I O N S
//#######################

    pub struct World<B: BitSequence, F: Flags, P: ComponentPointers> {
        components:         Vec<TypeId>,
        flags:              Vec<F>,
        component_columns:  HashMap<B, Box<dyn ComponentColumn>>,
        component_pointers: HashMap<P, Box<dyn ComponentCell>>,
        entities:           HashMap<Entity, B>,
        next_entity_id:     EntityId,
    } // struct World


    pub struct WorldBuilder<B: BitSequence, F: Flags, P: ComponentPointers> {
        components:         Vec<TypeId>,
        flags:              Vec<F>,
        component_count:    usize,
        component_columns:  HashMap<B, Box<dyn ComponentColumn>>,
        component_pointers: HashMap<P, Box<dyn ComponentCell>>,
    } // struct WorldBuilder


    pub trait Flags:
        'static
        + Sized
        + Eq
        + Hash
        + Clone + Copy {}

        
    pub trait ComponentPointers:
        'static
        + Sized
        + Eq
        + Hash
        + Clone + Copy {}


//###############################
// I M P L E M E N T A T I O N S
//###############################

    impl<B: BitSequence, F: Flags, P: ComponentPointers> World<B, F, P> {
        pub fn builder() -> WorldBuilder<B, F, P> {
            WorldBuilder {
                components:         Vec::default(),
                flags:              Vec::default(),
                component_count:    0usize,
                component_columns:  HashMap::default(),
                component_pointers: HashMap::default(),
            } // WorldBuilder
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


        pub(crate) fn flag_bit_mask(&self, flag: F) -> B {
            self.flags
                .iter()
                .enumerate()
                .find_map(|(index, id)| {
                    return match id == &flag {
                        true  => Some(B::bit_mask(index + self.components.len())),
                        false => None,
                    } // return ..
                }).expect("Attempted to get a flag bit mask that was not registered!")
        } // fn component_bit_mask()


        pub(crate) fn add_component_to_entity_builder<C: Component>(
            &mut self,
            component:       C,
            entity:          Entity,
            entity_bit_mask: &mut B,
        ) {

            let bit_mask     = self.component_bit_mask::<C>();
            *entity_bit_mask |= bit_mask;

            self.component_columns
                .get_mut(&bit_mask)
                .expect("Attempted to find a component column that was not registered!")
                .as_any_mut()
                .downcast_mut::<HashMap<Entity, Rc<RefCell<C>>>>()
                .expect("Failed to downcast a component column!")
                .insert(entity, Rc::new(RefCell::new(component)));

        } // fn add_component_to_entity_builder()


        pub(crate) fn add_shared_component_to_entity_builder<C: Component>(
            &mut self,
            component:       &Rc<RefCell<C>>,
            entity:          Entity,
            entity_bit_mask: &mut B,
        ) {

            let bit_mask     = self.component_bit_mask::<C>();
            *entity_bit_mask |= bit_mask;

            self.component_columns
                .get_mut(&bit_mask)
                .expect("Attempted to find a component column that was not registered!")
                .as_any_mut()
                .downcast_mut::<HashMap<Entity, Rc<RefCell<C>>>>()
                .expect("Failed to downcast a component column!")
                .insert(entity, component.clone());
        
        } // fn add_shared_component_to_entity_builder()


        pub(crate) fn add_flag_to_entity_builder(
            &mut self,
            flag:            F,
            entity_bit_mask: &mut B,
        ) {

            let bit_mask     = self.flag_bit_mask(flag);
            *entity_bit_mask |= bit_mask;

        } // fn add_flag_to_entity_builder()


        pub fn entity_has_component<C: Component>(&self, entity: Entity) -> bool {

            let bit_mask = self.component_bit_mask::<C>();
            *self.entities
                .get(&entity)
                .expect("Attempted to find an entity that was not registered!") & bit_mask == bit_mask

        } // fn entity_has_component()


        pub fn add_component_to_entity<C: Component>(
            &mut self,
            component: C,
            entity:    Entity,
        ) {

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


        pub fn add_component_to_entity_group<C: Component>(
            &mut self,
            component:    C,
            entity_group: &[Entity],
        ) {

            let bit_mask         = self.component_bit_mask::<C>();
            let component_column = self.component_columns
                .get_mut(&bit_mask)
                .expect("Attempted to find a component column that was not registered!")
                .as_any_mut()
                .downcast_mut::<HashMap<Entity, Rc<RefCell<C>>>>()
                .expect("Failed to downcast a component column!");

            entity_group.iter()
                .for_each(|entity| {

                    component_column.insert(*entity, Rc::new(RefCell::new(component.clone())));
                    *self.entities
                        .get_mut(entity)
                        .expect("Attempted to find an entity that was not registered!") |= bit_mask;

                }); // for_each()
        } // fn add_component_to_entity_group()


        pub fn add_shared_component_to_entity<C: Component>(
            &mut self,
            component: &Rc<RefCell<C>>,
            entity:    Entity,
        ) {

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


        pub fn add_shared_component_to_entity_group<C: Component>(
            &mut self,
            component:    &Rc<RefCell<C>>,
            entity_group: &[Entity],
        ) {

            let bit_mask         = self.component_bit_mask::<C>();
            let component_column = self.component_columns
                .get_mut(&bit_mask)
                .expect("Attempted to find a component column that was not registered!")
                .as_any_mut()
                .downcast_mut::<HashMap<Entity, Rc<RefCell<C>>>>()
                .expect("Failed to downcast a component column!");

            entity_group.iter()
                .for_each(|entity| {

                    component_column.insert(*entity, component.clone());
                    *self.entities
                        .get_mut(entity)
                        .expect("Attempted to find an entity that was not registered!") |= bit_mask;

                }); // for_each()
        } // fn add_component_to_entity_group()


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
                .remove_entity(entity);

        } // fn delete_entity_component()


        pub fn delete_entity_group_component<C: Component>(&mut self, entity_group: &[Entity]) {

            let bit_mask         = self.component_bit_mask::<C>();
            let component_column = self.component_columns
                .get_mut(&bit_mask)
                .expect("Attempted to find a component column that was not registered!")
                .as_any_mut()
                .downcast_mut::<HashMap<Entity, Rc<RefCell<C>>>>()
                .expect("Failed to downcast a component column!");

            entity_group
                .iter()
                .for_each(|entity| {

                    component_column.remove_entity(*entity);
                    *self.entities
                        .get_mut(entity)
                        .expect("Attempted to find an entity that was not registered!") &= !bit_mask;

                }); // for_each()
        } // fn delete_entity_group_component()


        pub fn set_entity_flag(
            &mut self,
            entity: Entity,
            flag:   F,
        ) {

            let bit_mask = self.flag_bit_mask(flag);
            *self.entities
                .get_mut(&entity)
                .expect("Attempted to find an entity that was not registered!") |= bit_mask;

        } // fn set_entity_flag()


        pub fn set_entity_group_flag(
            &mut self,
            entity_group: &[Entity],
            flag:         F,
        ) {

            let bit_mask = self.flag_bit_mask(flag);

            entity_group
                .iter()
                .for_each(|entity| *self.entities
                        .get_mut(entity)
                        .expect("Attempted to find an entity that was not registered!") |= bit_mask); // for_each()
        } // fn set_entity_group_flag()


        pub fn remove_entity_flag(
            &mut self,
            entity: Entity,
            flag:   F,
        ) {

            let bit_mask = self.flag_bit_mask(flag);
            *self.entities
                .get_mut(&entity)
                .expect("Attempted to find an entity that was not registered!") &= !bit_mask;

        } // fn remove_entity_flag()


        pub fn remove_entity_group_flag(
            &mut self,
            entity_group: &[Entity],
            flag:         F,
        ) {

            let bit_mask = self.flag_bit_mask(flag);

            entity_group
                .iter()
                .for_each(|entity| *self.entities
                        .get_mut(entity)
                        .expect("Attempted to find an entity that was not registered!") &= !bit_mask); // for_each()
        } // fn set_entity_group_flag()


        pub(crate) fn add_entity(
            &mut self,
            entity:          Entity,
            entity_bit_mask: B,
        ) { self.entities.insert(entity, entity_bit_mask); }


        pub(crate) fn get_component_column<C: Component>(&self) -> &HashMap<Entity, Rc<RefCell<C>>> {
            self.component_columns
                .get(&self.component_bit_mask::<C>())
                .expect("Attempted to find a component column that was not registered!")
                .as_any()
                .downcast_ref::<HashMap<Entity, Rc<RefCell<C>>>>()
                .expect("Failed to downcast a component column!")
        } // fn get_component_column()


        pub fn get_pointer_component<C: Component>(&self, id: P) -> &Rc<RefCell<C>> {
            self.component_pointers
                .get(&id)
                .expect("Attempted to find a component pointer that was not registered!")
                .as_any()
                .downcast_ref::<Rc<RefCell<C>>>()
                .expect("Failed to downcast a component!")
        } // fn get_pointer_component()


        pub(crate) fn get_entities(&self, bit_mask_filter: B) -> Vec<Entity> {
            self.entities
                .iter()
                .filter(|(_, bit_mask)| **bit_mask & bit_mask_filter == bit_mask_filter)
                .map(|(entity, _)| *entity)
                .collect()
        } // fn get_entities()


        pub fn delete_entity(&mut self, entity: Entity) {

            let entity_mask = self.entities
                .get(&entity)
                .expect("Attempted to delete an entity that was not registered!");

            self.component_columns
                .iter_mut()
                .filter(|(bit_mask, _)| *entity_mask & **bit_mask == **bit_mask)
                .for_each(|(_, component_column)| {
                    component_column.remove_entity(entity);
                }); // iter_mut()
            
            self.entities.remove(&entity);
            
        } // fn delete_entity()


        pub fn new_entity(&mut self) -> EntityBuilder<B, F, P> {

            self.next_entity_id += 1;
            EntityBuilder::new(self.next_entity_id - 1, self)

        } // fn new_entity()


        pub const fn new_query(&self) -> QueryBuilder<B, F, P> { QueryBuilder { bit_mask: B::MIN, world: self }}

    } // impl World


    impl<B: BitSequence, F: Flags, P: ComponentPointers> WorldBuilder<B, F, P> {
        pub fn with_default_component_pointer<C: Component + Default>(mut self, id: P) -> Self {
            
            match self.component_pointers.contains_key(&id) {
                true =>  { println!("The component pointer no.{} has been discarded as it was already registered!", self.component_pointers.len()) },
                false => { self.component_pointers.insert(id, Box::new(Rc::new(RefCell::new(C::default())))); },
            } // match ..

            self

        } // fn with_default_component_pointer()


        pub fn with_component_pointer<C: Component>(mut self, id: P, component: C) -> Self {

            match self.component_pointers.contains_key(&id) {
                true =>  { println!("The component pointer no.{} has been discarded as it was already registered!", self.component_pointers.len()) },
                false => { self.component_pointers.insert(id, Box::new(Rc::new(RefCell::new(component)))); },
            } // match ..

            self

        } // fn with_component_pointer()


        pub fn with_shared_component_pointer<C: Component>(mut self, id: P, component: &Rc<RefCell<C>>) -> Self {

            match self.component_pointers.contains_key(&id) {
                true =>  { println!("The component pointer no.{} has been discarded as it was already registered!", self.component_pointers.len()) },
                false => { self.component_pointers.insert(id, Box::new(component.clone())); },
            } // match ..

            self
            
        } // fn with_shared_component_pointer()


        pub fn with_component<C: Component>(mut self) -> Self {

            match self.components.contains(&TypeId::of::<C>()) {
                true =>  { println!("The component no.{} has been discarded as it was already registered!", self.component_count ) },
                false => {
                    self.components.push(TypeId::of::<C>());
                    self.component_columns.insert(
                        B::bit_mask(self.component_count),
                        Box::new(HashMap::<Entity, Rc<RefCell<C>>>::new()
                    )); // insert()
                    self.component_count += 1;
                }, // false
            } // match ..

            self

        } // fn with_component()


        pub fn build(self) -> World<B, F, P> {
            World {
                components:         self.components,
                flags:              self.flags,
                component_columns:  self.component_columns,
                component_pointers: self.component_pointers,
                entities:           HashMap::default(),
                next_entity_id:     0usize,
            } // World
        } // fn build()
    } // impl WorldBuilder
