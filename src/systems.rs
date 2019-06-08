//! specs systems.
use crate::components::*;
use crate::util;
use specs::{self, Join};
use ggez::conf::WindowMode;

pub struct MovementSystem;

impl<'a> specs::System<'a> for MovementSystem {
    type SystemData = (
        specs::WriteStorage<'a, Position>,
        specs::WriteStorage<'a, Motion>,
    );

    fn run(&mut self, (mut pos, mut motion): Self::SystemData) {
        // The `.join()` combines multiple components,
        // so we only access those entities which have
        // both of them.
        for (pos, motion) in (&mut pos, &mut motion).join() {
            pos.0 += motion.velocity;
            motion.velocity += motion.acceleration;
        }
    }
}

pub struct MatubokkuriSystem;

impl<'a> specs::System<'a> for MatubokkuriSystem {
    type SystemData = (
        specs::Entities<'a>,
        specs::WriteStorage<'a, Position>,
        specs::WriteStorage<'a, Matubokkuri>,
        specs::WriteStorage<'a, Motion>,
        specs::WriteStorage<'a, GameState>,
        specs::Read<'a, WindowMode>,
    );

    fn run(&mut self, (entities, mut pos, mut mat, mut motion, mut game_state, window_mode): Self::SystemData) {
        let mut mat_exhausted = false;
        for (mot, p, m) in (&mut motion, &pos, &mut mat).join() {
            if m.is_active() {
                mot.velocity.x = if m.is_open {
                    1.5
                } else {
                    0.
                };
                if p.0.y > window_mode.height - 60. {
                    mot.velocity.x = 0.;
                    mot.velocity.y = 0.;
                    mot.acceleration.y = 0.;
                    mat_exhausted = true;
                    m.is_open = false;
                    m.deactivate()
                }
            }
        }
        if mat_exhausted {
            entities.build_entity()
                .with(Position(util::point2(240.0, 110.0)), &mut pos)
                .with(Motion {
                    velocity: util::vec2(0.0, 1.0),
                    acceleration: util::vec2(0.0, 0.01),
                }, &mut motion)
                .with(Matubokkuri::default(), &mut mat)
                .build();
            for g in (&mut game_state).join() {
                g.matubokkuri_fall += 1;
            }
        }
    }
}

pub struct BulletManagementSystem;

impl<'a> specs::System<'a> for BulletManagementSystem {
    type SystemData = (
        specs::Entities<'a>,
        specs::ReadStorage<'a, Position>,
        specs::ReadStorage<'a, Bullet>,
        specs::Read<'a, WindowMode>,
    );

    fn run(&mut self, (entities, pos, bullet, window_mode): Self::SystemData) {
        for (e, pos, _b) in (&entities, &pos, &bullet).join() {
            if (pos.0.x < 0. || pos.0.x > window_mode.width) &&
               (pos.0.y < 0. || pos.0.y > window_mode.height) {
                entities.delete(e).unwrap();
            }
        }
    }
}

pub struct CollisionSystem;

impl<'a> specs::System<'a> for CollisionSystem {
    type SystemData = (
        specs::Entities<'a>,
        specs::WriteStorage<'a, Position>,
        specs::WriteStorage<'a, Bird>,
        specs::WriteStorage<'a, Bullet>,
        specs::WriteStorage<'a, Matubokkuri>,
        specs::Read<'a, WindowMode>,
    );

    fn run(&mut self, (entities, mut pos, mut bird, mut bullet, mut mat, window_mode): Self::SystemData) {
        let mut v = vec![];
        for (e_bu, p_bu, bi) in (&entities, &pos, &mut bullet).join() {
            for (e_bi, p_bi, bu) in (&entities, &pos, &mut bird).join() {
                let bird_width = 60.;
                let bird_height = 34.;

                if (p_bi.0.x <= p_bu.0.x && p_bu.0.x <= p_bi.0.x + bird_width) &&
                   (p_bi.0.y <= p_bu.0.y && p_bu.0.y <= p_bi.0.y + bird_height) {
                    if let BirdState::Bringing(e) = bu.state {
                        if let Some(ma) = mat.get_mut(e) {
                            ma.is_bringing = false;
                            ma.is_target = false;
                        }
                        v.push(e);
                    }
                    bi.deactivate();
                    bu.deactivate();
                    entities.delete(e_bi).unwrap();
                    entities.delete(e_bu).unwrap();
                }
            }
        }
        for e in v {
            if let Some(ma) = pos.get_mut(e) {
                ma.0.y =  window_mode.height - 60.;
            }
        }
    }
}

pub struct BirdSystem;

impl<'a> specs::System<'a> for BirdSystem {
    type SystemData = (
        specs::Entities<'a>,
        specs::WriteStorage<'a, Position>,
        specs::WriteStorage<'a, Matubokkuri>,
        specs::WriteStorage<'a, Motion>,
        specs::WriteStorage<'a, Bird>,
        specs::Read<'a, WindowMode>,
    );

    fn run(&mut self, (entities, mut pos, mut mat, mut mot, mut bird, window_mode): Self::SystemData) {
        let mut v = vec![];
        for (p, m, bi) in (&pos, &mut mot, &mut bird).join() {
            if bi.is_active() {
                match bi.state {
                    BirdState::Entry => if p.0.x < window_mode.width - 100. { 
                        bi.state = BirdState::Seeking;
                    },
                    BirdState::Seeking => {
                        if (p.0.x < 150. && m.velocity.x < 0.) ||
                           (p.0.x > window_mode.width - 100. && m.velocity.x > 0.) {
                            m.velocity.x *= -1.;
                        }
                        for (e, m) in (&entities, &mut mat).join() {
                            if !m.is_active() && !m.is_target {
                                bi.state = BirdState::Reaching(e);
                                m.is_target = true;
                            }
                        }
                    }
                    BirdState::Reaching(e) => {
                        if let Some(mpos) = pos.get(e) {
                            let v = util::vec2(mpos.0.x - p.0.x, mpos.0.y - p.0.y).normalize() * 2.;
                            m.velocity = v;

                            if (p.0.x - mpos.0.x).abs() < 1. && (p.0.y - mpos.0.y).abs() < 1. {
                                bi.state = BirdState::Grabbing(e);
                                m.velocity.x = 0.;
                                m.velocity.y = 0.;
                            }
                        }
                    }
                    BirdState::Grabbing(e) => {
                        if bi.grab_timer != 0 {
                            bi.grab_timer -= 1;
                        } else {
                            bi.state = BirdState::Bringing(e);
                        }
                    },
                    BirdState::Bringing(e) => {
                        if let Some(ma) = mat.get_mut(e) {
                            m.velocity.x = 1.5;
                            m.velocity.y = -2.0;
                            v.push((e, p.clone()));
                            ma.is_bringing = true;
                        }
                    }
                }
            }
        }
        for (e, p) in v {
            if let Some(mpos) = pos.get_mut(e) {
                *mpos = p;
            }
        }
    }
}
