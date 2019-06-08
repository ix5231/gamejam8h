use crate::types::*;

use specs::*;
use specs_derive::*;

// ///////////////////////////////////////////////////////////////////////
// Components
// ///////////////////////////////////////////////////////////////////////

/// A position in the game world.
#[derive(Clone, Debug, Component)]
#[storage(VecStorage)]
pub struct Position(pub Point2);

/// Motion in the game world.
#[derive(Clone, Debug, Component)]
#[storage(VecStorage)]
pub struct Motion {
    pub velocity: Vector2,
    pub acceleration: Vector2,
}

#[derive(Clone, Debug, Component)]
#[storage(HashMapStorage)]
pub struct Matubokkuri {
    pub is_open: bool,
    is_active: bool,
    pub is_target: bool,
    pub is_bringing: bool,
}

impl Matubokkuri {
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
}

impl Default for Matubokkuri {
    fn default() -> Matubokkuri {
        Matubokkuri { is_open: false, is_active: true, is_target: false, is_bringing: false }
    }
}

#[derive(Clone, Debug, Component)]
#[storage(VecStorage)]
pub struct Bullet {
    is_active: bool,
}

impl Bullet {
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
}

impl Default for Bullet {
    fn default() -> Bullet {
        Bullet { is_active: true }
    }
}

#[derive(Clone, Debug, Copy)]
pub enum BirdState {
    Entry,
    Seeking,
    Reaching(Entity),
    Grabbing(Entity),
    Bringing(Entity),
}

#[derive(Clone, Debug, Component)]
#[storage(HashMapStorage)]
pub struct Bird {
    pub state: BirdState,
    pub grab_timer: i32,
    is_active: bool,
}

impl Bird {
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
}

impl Default for Bird {
    fn default() -> Bird {
        Bird { state: BirdState::Entry, grab_timer: 100, is_active: true }
    }
}

#[derive(Clone, Debug, Default, Component)]
#[storage(HashMapStorage)]
pub struct Score {
    pub score: i32,
}

pub fn register_components(specs_world: &mut World) {
    specs_world.register::<Position>();
    specs_world.register::<Motion>();
    specs_world.register::<Matubokkuri>();
    specs_world.register::<Bullet>();
    specs_world.register::<Bird>();
    specs_world.register::<Score>();
}
