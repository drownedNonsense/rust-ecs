//#########################
// D E P E N D E N C I E S
//#########################

    use std::rc::Rc;
    use std::cell::RefCell;
    use std::hash::Hash;

    use crate::worlds::World;
    use crate::components::Component;

    use rust_utils::BitSequence;


//#######################
// D E F I N I T I O N S
//#######################

    #[derive(Clone, Copy, Hash, PartialEq, Eq, Default, Debug)]
    pub struct Entity(EntityId);


    pub struct EntityBuilder<'world, B: BitSequence, F: BitSequence, P: BitSequence> {
        entity:   Entity,
        bit_mask: B,
        world:    &'world mut World<B, F, P>,
    } // struct EntityBuilder


    pub(crate) type EntityId = usize;


//###############################
// I M P L E M E N T A T I O N S
//###############################

    impl<'world, B: BitSequence, F: BitSequence, P: BitSequence> EntityBuilder<'world, B, F, P> {
        pub(crate) fn new(
            id:    EntityId,
            world: &'world mut World<B, F, P>
        ) -> Self { EntityBuilder { entity: Entity(id), bit_mask: B::default(), world }}


        pub fn with_component<C: Component>(mut self, component: C) -> Self {

            self.world.add_component_to_entity_builder(component, self.entity, &mut self.bit_mask);
            self
            
        } // fn with_component()


        pub fn with_shared_component<C: Component>(mut self, component: &Rc<RefCell<C>>) -> Self {

            self.world.add_shared_component_to_entity_builder(component, self.entity, &mut self.bit_mask);
            self

        } // fn with_shared_component()


        pub fn with_flag(
            mut self,
            flag:    F,
            variant: Option<B>,
        ) -> Self {

            self.world.add_flag_to_entity_builder(flag, variant, &mut self.bit_mask);
            self

        } // fn with_flag()


        pub fn build(self) -> Entity {

            self.world.add_entity(self.entity, self.bit_mask);
            self.entity

        } // fn build()
    } // impl EntityBuilder ..
