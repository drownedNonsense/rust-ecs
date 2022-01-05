//#########################
// D E P E N D E N C I E S
//#########################

    use std::collections::HashMap;
    use std::any::TypeId;
    use std::rc::Rc;
    use std::cell::RefCell;

    use crate::components::{Component, ComponentCell, ComponentColumn, BitMask};
    use crate::entities::{Entity, EntityBuilder, EntityId};
    use crate::queries::QueryBuilder;


//#######################
// D E F I N I T I O N S
//#######################

    pub struct World<B: BitMask> {
        components: Vec<TypeId>,
        component_columns: HashMap<B, Box<dyn ComponentColumn>>,
        static_components: HashMap<u8, Box<dyn ComponentCell>>,
        entities: HashMap<Entity, B>,
        next_entity_id: EntityId,
    } // struct World


    pub struct WorldBuilder<B: BitMask> {
        components: Vec<TypeId>,
        component_count: usize,
        component_columns: HashMap<B, Box<dyn ComponentColumn>>,
        static_components: HashMap<u8, Box<dyn ComponentCell>>,
    } // struct WorldBuilder


//###############################
// I M P L E M E N T A T I O N S
//###############################

    impl<B: BitMask> World<B> {
        fn new(
            components: Vec<TypeId>,
            component_columns: HashMap<B, Box<dyn ComponentColumn>>,
            static_components: HashMap<u8, Box<dyn ComponentCell>>
        ) -> Self {
            World { components, component_columns, static_components, entities: HashMap::new(), next_entity_id: 0 }
        } // fn new'


        pub fn builder() -> WorldBuilder<B> {
            WorldBuilder::<B>::new()
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


        pub(crate) fn add_component_to_entity<C: Component>(&mut self, component: C, entity: Entity, entity_bit_mask: &mut B) {
            let bit_mask = self.component_bit_mask::<C>();
            *entity_bit_mask |= bit_mask;
            self.component_columns.get_mut(&bit_mask).unwrap().as_any_mut().downcast_mut::<HashMap<Entity, Rc<RefCell<C>>>>().unwrap().insert(entity, Rc::new(RefCell::new(component)));
        } // fn add_component()


        pub(crate) fn add_shared_component_to_entity<C: Component>(&mut self, component: &Rc<RefCell<C>>, entity: Entity, entity_bit_mask: &mut B) {
            let bit_mask = self.component_bit_mask::<C>();
            *entity_bit_mask |= bit_mask;
            self.component_columns.get_mut(&bit_mask).unwrap().as_any_mut().downcast_mut::<HashMap<Entity, Rc<RefCell<C>>>>().unwrap().insert(entity, component.clone());
        } // fn add_component()


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


        pub fn get_static_component<C: Component>(&self, id: u8) -> &Rc<RefCell<C>> {
            self.static_components
                .get(&id)
                .expect("Attempted to find a static component that was not registered!")
                .as_any()
                .downcast_ref::<Rc<RefCell<C>>>()
                .expect("Failed to downcast a static component!")
        } // fn get_static_component()


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


        pub fn new_entity(&mut self) -> EntityBuilder<B> {
            self.next_entity_id += 1;
            EntityBuilder::new(self.next_entity_id - 1, self)
        } // fn new_entity()


        pub fn new_query(&self) -> QueryBuilder<B> {
            QueryBuilder::new(self)
        } // fn new_query()
    } // impl World


    impl<B: BitMask> WorldBuilder<B> {
        fn new() -> Self {
            WorldBuilder {
                components: Vec::new(),
                component_count: 0,
                component_columns: HashMap::new(),
                static_components: HashMap::new(),
            } // WorldBuilder
        } // fn new()


        pub fn with_empty_static_component<C: Component + Default>(mut self, id: u8) -> Self {
            match self.static_components.contains_key(&id) {
                true =>  { println!("The static component no.{} has been discarded as it was already registered!", self.static_components.len() ) },
                false => { self.static_components.insert(id, Box::new(Rc::new(RefCell::new(C::default())))); },
            } // match ..

            self
        } // fn with_empty_static_component()


        pub fn with_static_component<C: Component>(mut self, component: C, id: u8) -> Self {
            match self.static_components.contains_key(&id) {
                true =>  { println!("The static component no.{} has been discarded as it was already registered!", self.static_components.len() ) },
                false => { self.static_components.insert(id, Box::new(Rc::new(RefCell::new(component)))); },
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


        pub fn build(self) -> World<B> {
            World::new(self.components, self.component_columns, self.static_components)
        } // fn build()
    } // impl WorldBuilder
