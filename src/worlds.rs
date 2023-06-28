//#########################
// D E P E N D E N C I E S
//#########################

    use std::collections::HashMap;
    use std::any::TypeId;
    use std::rc::Rc;
    use std::cell::RefCell;
    use std::ops::Range;
    use std::hash::Hash;
    use std::fmt::Debug;

    use crate::components::{Component, ComponentCell, ComponentColumn};
    use crate::entities::{Entity, EntityBuilder, EntityId};
    use crate::queries::QueryBuilder;

    use rusty_toolkit::BitField;


//#######################
// D E F I N I T I O N S
//#######################

    pub struct World<B: BitField, F: BitField, P: Hash + Eq + Debug> {
        components:         Vec<TypeId>,
        flags:              HashMap<F, Range<u8>>,
        component_columns:  HashMap<B, Box<dyn ComponentColumn>>,
        component_pointers: HashMap<P, Box<dyn ComponentCell>>,
        entities:           HashMap<Entity, B>,
        next_entity_id:     EntityId,
    } // struct World


    pub struct WorldBuilder<B: BitField, F: BitField, P: Hash + Eq + Debug> {
        components:         Vec<TypeId>,
        flags:              HashMap<F, Range<u8>>,
        component_count:    usize,
        component_columns:  HashMap<B, Box<dyn ComponentColumn>>,
        component_pointers: HashMap<P, Box<dyn ComponentCell>>,
    } // struct WorldBuilder
    

//###############################
// I M P L E M E N T A T I O N S
//###############################

    impl<B: BitField, F: BitField, P: Hash + Eq + Debug> World<B, F, P> {
        pub fn builder() -> WorldBuilder<B, F, P> {
            WorldBuilder {
                components:         Vec::default(),
                flags:              HashMap::default(),
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
                        true  => Some(B::bit(index as u8)),
                        false => None,
                    } // return ..
                }).expect("Attempted to get a component bit mask that was not registered!")

        } // fn component_bit_mask()


        pub(crate) fn flag_bit_mask(&self, flag: F, variant: Option<B>) -> B {
            self.flags
                .iter()
                .find_map(|(id, range)| {
                    return match id == &flag {
                        true => Some(match variant {
                            Some(variant) => ((variant << range.start) & B::bit_mask(range.clone())) << self.components.len() as u8,
                            None          => B::bit_mask(range.clone()),
                        }), // => ..
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
            variant:         Option<B>,
            entity_bit_mask: &mut B,
        ) {

            let bit_mask     = self.flag_bit_mask(flag, variant);
            *entity_bit_mask |= bit_mask;

        } // fn add_flag_to_entity_builder()


        pub fn entity_has_component<C: Component>(&self, entity: Entity) -> bool {

            let bit_mask = self.component_bit_mask::<C>();
            *self.entities
                .get(&entity)
                .expect("Attempted to find an entity that was not registered!") & bit_mask == bit_mask

        } // fn entity_has_component()


        pub fn entity_group_has_component<C: Component>(&self, entity_group: &[Entity]) -> Vec<bool> {

            let bit_mask = self.component_bit_mask::<C>();

            entity_group.iter()
                .map(|entity| *self.entities
                    .get(&entity)
                    .expect("Attempted to find an entity that was not registered!") & bit_mask == bit_mask
                ).collect()

        } // fn entity_group_has_component()


        pub fn entity_has_flag(
            &self,
            entity:  Entity,
            flag:    F,
            variant: Option<B>,
        ) -> bool {
            self.entities
                .get(&entity)
                .expect("Attempted to find an entity that was not registered!")
                .has_bits(self.flag_bit_mask(flag, variant))
        } // fn entity_has_flag()


        pub fn entity_group_has_flag(
            &self,
            entity_group: &[Entity],
            flag:         F,
            variant:      Option<B>,
        ) -> Vec<bool> {

            let bit_mask = self.flag_bit_mask(flag, variant);

            entity_group.iter()
                .map(|entity| self.entities.get(&entity)
                    .expect("Attempted to find an entity that was not registered!")
                    .has_bits(bit_mask)
                ).collect()

        } // fn entity_group_has_flag()


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
            entity:  Entity,
            flag:    F,
            variant: Option<B>,
        ) {

            let bit_mask = self.flag_bit_mask(flag, variant);
            *self.entities
                .get_mut(&entity)
                .expect("Attempted to find an entity that was not registered!") |= bit_mask;

        } // fn set_entity_flag()


        pub fn set_entity_group_flag(
            &mut self,
            entity_group: &[Entity],
            flag:         F,
            variant:      Option<B>,
        ) {

            let bit_mask = self.flag_bit_mask(flag, variant);

            entity_group
                .iter()
                .for_each(|entity| *self.entities
                        .get_mut(entity)
                        .expect("Attempted to find an entity that was not registered!") |= bit_mask);
        } // fn set_entity_group_flag()


        pub fn remove_entity_flag(
            &mut self,
            entity:  Entity,
            flag:    F,
            variant: Option<B>,
        ) {

            let bit_mask = self.flag_bit_mask(flag, variant);
            *self.entities
                .get_mut(&entity)
                .expect("Attempted to find an entity that was not registered!") &= !bit_mask;

        } // fn remove_entity_flag()


        pub fn remove_entity_group_flag(
            &mut self,
            entity_group: &[Entity],
            flag:         F,
            variant:      Option<B>,
        ) {

            let bit_mask = self.flag_bit_mask(flag, variant);

            entity_group
                .iter()
                .for_each(|entity| *self.entities
                        .get_mut(entity)
                        .expect("Attempted to find an entity that was not registered!") &= !bit_mask);
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


        pub fn delete_entity_group(&mut self, entity_group: &[Entity]) {
            entity_group.iter()
                .for_each(|entity| self.delete_entity(*entity));
        } // fn delete_entity_group()


        pub fn new_entity(&mut self) -> EntityBuilder<B, F, P> {

            self.next_entity_id += 1;
            EntityBuilder::new(self.next_entity_id - 1, self)

        } // fn new_entity()


        pub const fn new_query(&self) -> QueryBuilder<B, F, P> { QueryBuilder { bit_mask: B::MIN, world: self }}

    } // impl World


    impl<B: BitField, F: BitField, P: Hash + Eq + Debug> WorldBuilder<B, F, P> {
        pub fn with_component_pointer<C: Component, T: Into<P>>(mut self, id: T, component: C) -> Self {

            let id = id.into();
            match self.component_pointers.contains_key(&id) {
                true =>  { println!("The component pointer {:?} has been discarded as it was already registered!", id) },
                false => { self.component_pointers.insert(id, Box::new(Rc::new(RefCell::new(component)))); },
            } // match ..

            self

        } // fn with_component_pointer()


        pub fn with_shared_component_pointer<C: Component, T: Into<P>>(mut self, id: T, component: &Rc<RefCell<C>>) -> Self {

            let id = id.into();
            match self.component_pointers.contains_key(&id) {
                true =>  { println!("The component pointer {:?} has been discarded as it was already registered!", id) },
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
                        B::bit(self.component_count as u8),
                        Box::new(HashMap::<Entity, Rc<RefCell<C>>>::new()
                    )); // insert()
                    self.component_count += 1;
                }, // false
            } // match ..

            self

        } // fn with_component()


        pub fn with_flag<T: Into<F>>(mut self, flag: T, range: Range<u8>) -> Self {

            self.flags.insert(flag.into(), range);
            self

        } // fn with_flag()


        pub fn build(self) -> World<B, F, P> {

            assert!(
                self.component_count + usize::from(
                    match self.flags.iter().max_by(|(_, a), (_, b)| a.start.cmp(&b.end)) {
                        Some(max) => max.1.end,
                        None      => 0u8,
                    } // match ..
                ) < usize::from(B::BITS),
                "WARNING: entity bitmask is overflowing!\n consider using a larger bit count!"
            );

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
