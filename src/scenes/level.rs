use ggez;
use ggez::graphics;
use ggez_goodies::scene;
use log::*;
use specs::{self, Join};
use warmy;
use specs::world::Builder;

use crate::components as c;
use crate::input;
use crate::resources;
use crate::scenes;
use crate::systems::*;
use crate::world::World;
use crate::components;
use crate::util;

pub struct LevelScene {
    done: bool,
    tree: warmy::Res<resources::Image>,
    matubo: warmy::Res<resources::Image>,
    bird: warmy::Res<resources::Image>,
    bird_mirror: warmy::Res<resources::Image>,
    bird_timer: i32,
    dispatcher: specs::Dispatcher<'static, 'static>,
}

impl LevelScene {
    pub fn new(ctx: &mut ggez::Context, world: &mut World) -> Self {
        let done = false;
        let tree = world
            .resources
            .get::<resources::Image>(&resources::Key::from_path("/images/tree.png"), ctx)
            .unwrap();
        let matubo = world
            .resources
            .get::<resources::Image>(&resources::Key::from_path("/images/matubo.png"), ctx)
            .unwrap();
        let bird = world
            .resources
            .get::<resources::Image>(&resources::Key::from_path("/images/bird.png"), ctx)
            .unwrap();
        let bird_mirror = world
            .resources
            .get::<resources::Image>(&resources::Key::from_path("/images/bird_mirror.png"), ctx)
            .unwrap();
        world
            .specs_world
            .create_entity()
            .with(components::Position(util::point2(240.0, 110.0)))
            .with(components::Motion {
                velocity: util::vec2(0.0, 1.0),
                acceleration: util::vec2(0.0, 0.01),
            })
            .with(components::Matubokkuri::default())
            .build();
        world
            .specs_world
            .create_entity()
            .with(components::Score::default())
            .build();
        world
            .specs_world
            .create_entity()
            .with(components::GameState::default())
            .build();

        let dispatcher = Self::register_systems();
        LevelScene {
            done,
            tree,
            matubo,
            bird,
            bird_mirror,
            bird_timer: 0,
            dispatcher,
        }
    }

    fn register_systems() -> specs::Dispatcher<'static, 'static> {
        specs::DispatcherBuilder::new()
            .with(MovementSystem, "sys_movement", &[])
            .with(MatubokkuriSystem, "matubokkuri", &[])
            .with(BulletManagementSystem, "bullet_mgr", &[])
            .with(CollisionSystem, "collision", &[])
            .with(BirdSystem, "sys_bird", &[])
            .build()
    }

    fn shot_bullet(&self, gameworld: &mut World, ctx: &mut ggez::Context) {
        let mut vs = vec![];
        {
            let mat = gameworld.specs_world.read_storage::<c::Matubokkuri>();
            let pos = gameworld.specs_world.read_storage::<c::Position>();
            let mouse = ggez::input::mouse::position(ctx);
            for (m, p) in (&mat, &pos).join() {
                if m.is_active() {
                    let v = util::vec2(mouse.x - p.0.x - 25., mouse.y - p.0.y - 25.).normalize();
                    vs.push((v, p.clone()));
                }
            }
        }
        for (v, p) in vs {
            gameworld
                .specs_world
                .create_entity()
                .with(components::Position(util::point2(p.0.x + 25., p.0.y + 25.)))
                .with(components::Motion {
                    velocity: v * 3.,
                    acceleration: util::vec2(0.0, 0.0),
                })
                .with(components::Bullet::default())
                .build();
        }
    }
}

impl scene::Scene<World, input::Event> for LevelScene {
    fn update(&mut self, gameworld: &mut World, ctx: &mut ggez::Context) -> scenes::Switch {
        gameworld.specs_world.maintain();
        self.dispatcher.dispatch(&mut gameworld.specs_world.res);

        if ggez::input::mouse::button_pressed(ctx, ggez::input::mouse::MouseButton::Left) {
            self.shot_bullet(gameworld, ctx);
            let mut mat = gameworld.specs_world.write_storage::<c::Matubokkuri>();
            for m in (&mut mat).join() {
                m.is_open = false;
            }
        } else if ggez::input::mouse::button_pressed(ctx, ggez::input::mouse::MouseButton::Right) {
            let mut mat = gameworld.specs_world.write_storage::<c::Matubokkuri>();
            for m in (&mut mat).join() {
                m.is_open = true;
            }
        } else {
            let mut mat = gameworld.specs_world.write_storage::<c::Matubokkuri>();
            for m in (&mut mat).join() {
                m.is_open = false;
            }
        }

        let bird_spawn_time = 60 * 3;
        if self.bird_timer == bird_spawn_time {
            gameworld
                .specs_world
                .create_entity()
                .with(components::Position(util::point2(800.0, 200.0)))
                .with(components::Motion {
                    velocity: util::vec2(-2.0, 0.),
                    acceleration: util::vec2(0., 0.)
                })
                .with(components::Bird::default())
                .build();
            self.bird_timer = 0;
        } else {
            self.bird_timer += 1;
        }

        let mut score = gameworld.specs_world.write_storage::<c::Score>();
        for s in (&mut score).join() {
            let mat = gameworld.specs_world.read_storage::<c::Matubokkuri>();
            let pos = gameworld.specs_world.read_storage::<c::Position>();
            let mut sc = 0;
            for (m, p) in (&mat, &pos).join() {
                if !m.is_active() && !m.is_bringing {
                    sc += p.0.x as i32;
                }
            }
            s.score = sc;
        }

        let game_state = gameworld.specs_world.read_storage::<c::GameState>();
        for g in game_state.join() {
            if g.matubokkuri_fall == 10 {
                self.done = true;
            }
        }

        if self.done {
            ctx.continuing = false;
            for s in (&mut score).join() {
                println!("Good Game!");
                println!("Score: {}", s.score);
            }
            scene::SceneSwitch::None
        } else {
            scene::SceneSwitch::None
        }
    }

    fn draw(&mut self, gameworld: &mut World, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let mat = gameworld.specs_world.read_storage::<c::Matubokkuri>();
        let pos = gameworld.specs_world.read_storage::<c::Position>();
        let mot = gameworld.specs_world.read_storage::<c::Motion>();
        let bul = gameworld.specs_world.read_storage::<c::Bullet>();
        let bir = gameworld.specs_world.read_storage::<c::Bird>();
        let score = gameworld.specs_world.read_storage::<c::Score>();

        let tree_place = [-230.,0.];
        for (_m, p) in (&mat, &pos).join() {
            graphics::draw(
                ctx,
                &(self.matubo.borrow().0),
                graphics::DrawParam::default().dest(p.0),
            )?;
        }
        graphics::draw(
            ctx,
            &(self.tree.borrow().0),
            graphics::DrawParam::default().dest(tree_place),
        )?;

        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            [0., 0.],
            4.,
            1.,
            graphics::Color::from_rgb(116, 80, 48)
        ).unwrap();
        for (b, p) in (&bul, &pos).join() {
            if b.is_active() {
                graphics::draw(
                    ctx,
                    &circle,
                    graphics::DrawParam::default().dest(p.0),
                )?;
            }
        }

        for (b, p, m) in (&bir, &pos, &mot).join() {
            if b.is_active() {
                // DEBUG: show state
                let circle = graphics::Mesh::new_circle(
                    ctx,
                    graphics::DrawMode::fill(),
                    [0., 0.],
                    4.,
                    1.,
                    match b.state {
                        c::BirdState::Entry => graphics::Color::from_rgb(0, 0, 0),
                        c::BirdState::Seeking => graphics::Color::from_rgb(255, 0, 0),
                        c::BirdState::Reaching(_) => graphics::Color::from_rgb(0, 0, 255),
                        c::BirdState::Grabbing(_) => graphics::Color::from_rgb(255, 255, 0),
                        c::BirdState::Bringing(_) => graphics::Color::from_rgb(0, 255, 0),
                    }
                ).unwrap();
                graphics::draw(
                    ctx,
                    &circle,
                    graphics::DrawParam::default().dest(p.0),
                )?;

                graphics::draw(
                    ctx,
                    &((if m.velocity.x < 0. { &self.bird } else { &self.bird_mirror }).borrow().0),
                    graphics::DrawParam::default().dest(p.0),
                )?;
            }
        }

        for s in score.join() {
            graphics::draw(
                ctx,
                &graphics::Text::new(format!("Score: {}", s.score)),
                graphics::DrawParam::default().dest([ctx.conf.window_mode.width - 100., 0.]),
            )?;
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "LevelScene"
    }

    fn input(&mut self, gameworld: &mut World, ev: input::Event, _started: bool) {
        debug!("Input: {:?}", ev);
        if gameworld.input.get_button_pressed(input::Button::Menu) {
            self.done = true;
        }
    }
}
