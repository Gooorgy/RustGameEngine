use crate::component::component_storage::ComponentInsertion;
use crate::entity::Entity;
use crate::world::{EntityAllocator, World};

pub struct Commands<'a> {
    pub(crate) queue: Vec<Command>,
    pub(crate) entity_allocator: &'a mut EntityAllocator,
}

pub enum Command {
    SpawnEntity(Entity, Box<dyn FnOnce(&mut World)>),
    DespawnEntity(Entity),
}

impl<'a> Commands<'a> {
    #[allow(private_bounds)]
    pub fn spawn_entity(&mut self, components: impl ComponentInsertion + 'static) -> Entity {
        let reserved_entity_id = self.entity_allocator.reserve();

        self.queue.push(Command::SpawnEntity(
            reserved_entity_id,
            Box::new(move |w: &mut World| {
                w.create_reserved_entity(reserved_entity_id, components);
            }),
        ));

        reserved_entity_id
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        self.queue.push(Command::DespawnEntity(entity));
    }

    pub fn into_queue(self) -> Vec<Command> {
        self.queue
    }
}
