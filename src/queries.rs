//#########################
// D E P E N D E N C I E S
//#########################

    use std::rc::Rc;
    use std::cell::RefCell;

    use crate::worlds::World;
    use crate::components::Component;
    use crate::entities::Entity;

    use rust_utils::BitSequence;


//#######################
// D E F I N I T I O N S
//#######################

    pub struct Query<'world, B: BitSequence, F: BitSequence, P: BitSequence> {
        entities: Vec<Entity>,
        world:    &'world World<B, F, P>,
    } // struct Query


    pub struct QueryBuilder<'world, B: BitSequence, F: BitSequence, P: BitSequence> {
        pub(crate) bit_mask: B,
        pub(crate) world:    &'world World<B, F, P>,
    } // struct QueryBuilder


//###############################
// I M P L E M E N T A T I O N S
//###############################

    impl<'world, B: BitSequence, F: BitSequence, P: BitSequence> Query<'world, B, F, P> {
        pub fn get_components<C: Component>(&self) -> Vec<&Rc<RefCell<C>>> {

            let component_column = self.world.get_component_column::<C>();
            self.entities
                .iter()
                .map(|entity| component_column
                    .get(entity)
                    .expect("Attempted to find a component with an entity ID that was not registered in the column!"))
                .collect()

        } // fn get_components()


        pub fn get_entities(&self) -> Vec<Entity> { self.entities.clone() }

    } // impl Query


    impl<'world, B: BitSequence, F: BitSequence, P: BitSequence> QueryBuilder<'world, B, F, P> {
        pub fn with_component<C: Component>(mut self) -> Self {

            self.bit_mask |= self.world.component_bit_mask::<C>();
            self

        } // fn with_component()


        pub fn with_flag<T: Into<F>>(mut self, flag: T, variant: Option<B>) -> Self {

            self.bit_mask |= self.world.flag_bit_mask(flag.into(), variant);
            self
            
        } // fn with_flag()


        pub fn build(self) -> Query<'world, B, F, P> {

            let entities = self.world.get_entities(self.bit_mask);

            Query {
                entities,
                world: self.world,
            } // Query
        } // fn build()
    } // impl QueryBuilder
