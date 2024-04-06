use godot::prelude::*;
use godot::engine::{Resource, Texture2D};

#[derive(GodotClass)]
#[class(base=Resource)]
pub struct Weapon {
    #[export]
    model: Option<Gd<PackedScene>>,
    #[export]
    position: Vector3,
    #[export]
    rotation: Vector3,
    #[export]
    muzzle_position: Vector3,

    #[export(range = (0.1, 1.0))]
    cooldown: f64,
    #[export(range = (1.0, 20.0))]
    max_distance: f32,
    #[export(range = (0.0, 100.0))]
    damage: f32,
    #[export(range = (0.0, 5.0))]
    spread: f32,
    #[export(range = (1.0, 5.0))]
    shot_count: i32,
    #[export(range = (0.0, 20.0))]
    knockback: f32,

    #[export]
    sound_shoot: GString,

    #[export]
    crosshair: Option<Gd<Texture2D>>,

    base: Base<Resource>
}


#[godot_api]
impl IResource for Weapon {
    fn init(base: Base<Resource>) -> Self {
        Self {
            model: None,
            position: Vector3::ZERO,
            rotation: Vector3::ZERO,
            muzzle_position: Vector3::ZERO,

            cooldown: 0.1,
            max_distance: 10.0,
            damage: 25.0,
            spread: 0.0,
            shot_count: 1,
            knockback: 20.0,

            sound_shoot: GString::new(),

            crosshair: None,

            base,
        }
    }
}
