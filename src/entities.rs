//#########################
// D E P E N D E N C I E S
//#########################

    use std::rc::Rc;
    use std::cell::RefCell;
    use std::hash::Hash;

    use crate::worlds::World;
    use crate::components::{Component, BitMask, StaticComponentId};


//#######################
// D E F I N I T I O N S
//#######################

    #[derive(Hash, Clone, Copy, PartialEq, Eq)]
    pub struct Entity(EntityId);


    pub struct EntityBuilder<'world, B: BitMask, S: StaticComponentId> {
        entity: Entity,
        bit_mask: B,
        world: &'world mut World<B, S>,
    } // struct EntityBuilder


    pub(crate) type EntityId = usize;


//###############################
// I M P L E M E N T A T I O N S
//###############################

    impl<'world, B: BitMask, S: StaticComponentId> EntityBuilder<'world, B, S> {
        pub(crate) fn new(id: EntityId, world: &'world mut World<B, S>) -> Self {
            EntityBuilder { entity: Entity(id), bit_mask: B::default(), world }
        } // fn new()


        pub fn with_empty_component<C: Component + Default>(mut self) -> Self {
            self.world.add_component_to_entity(C::default(), self.entity, &mut self.bit_mask);
            self
        } // fn with_empty_component()


        pub fn with_component<C: Component>(mut self, component: C) -> Self {
            self.world.add_component_to_entity(component, self.entity, &mut self.bit_mask);
            self
        } // fn with_component()


        pub fn with_shared_component<C: Component>(mut self, component: &Rc<RefCell<C>>) -> Self {
            self.world.add_shared_component_to_entity(component, self.entity, &mut self.bit_mask);
            self
        } // fn with_component()


        pub fn build(self) -> Entity {
            self.world.add_entity(self.entity, self.bit_mask);
            self.entity
        } // fn build()
    } // impl EntityBuilder ..
