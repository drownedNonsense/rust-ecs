//#########################
// D E P E N D E N C I E S
//#########################

    use std::rc::Rc;
    use std::cell::RefCell;

    use crate::worlds::World;
    use crate::components::{Component, BitMask, StaticComponentId};
    use crate::entities::Entity;


//#######################
// D E F I N I T I O N S
//#######################

    pub struct Query<'world, B: BitMask, S: StaticComponentId> {
        entities: Vec<Entity>,
        world: &'world World<B, S>,
    } // struct Query

    pub struct QueryBuilder<'world, B: BitMask, S: StaticComponentId> {
        bit_mask: B,
        world: &'world World<B, S>,
    } // struct EntityBuilder


//###############################
// I M P L E M E N T A T I O N S
//###############################

    impl<'world, B: BitMask, S: StaticComponentId> Query<'world, B, S> {
        fn new(entities: Vec<Entity>, world: &'world World<B, S>) -> Self {
            Query { entities, world }
        } // fn new()


        pub fn get_components<C: Component>(&self) -> Vec<&Rc<RefCell<C>>> {
            let component_column = self.world.get_component_column::<C>();
            self.entities
                .iter()
                .map(|entity| component_column
                    .get(entity)
                    .expect("Attempted to find a component with an entity ID that was not registered in the column!"))
                .collect()
        } // fn get_components()


        pub fn try_get_components<C: Component>(&self) -> Vec<Option<&Rc<RefCell<C>>>> {
            let component_column = self.world.get_component_column::<C>();
            self.entities
                .iter()
                .map(|entity| component_column
                    .get(entity))
                .collect()
        } // fn try_get_components()


        pub fn get_entities(&self) -> &[Entity] {
            &self.entities
        } // fn get_entities()
    } // impl Query


    impl<'world, B: BitMask, S: StaticComponentId> QueryBuilder<'world, B, S> {
        pub(crate) fn new(world: &'world World<B, S>) -> Self {
            QueryBuilder { bit_mask: B::default(), world }
        } // fn new()


        pub fn with_component<C: Component>(mut self) -> Self {
            self.bit_mask |= self.world.component_bit_mask::<C>();
            self
        } // fn with_component()


        pub fn build(self) -> Query<'world, B, S> {
            let entities = self.world.get_entities(self.bit_mask);
            Query::new(entities, &self.world)
        } // fn build()
    } // impl QueryBuilder
