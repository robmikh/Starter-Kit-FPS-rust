use godot::engine::{AnimatedSprite3D, Area3D, Node3D, RayCast3D};
use godot::prelude::*;
use rand::Rng;

#[derive(GodotClass)]
#[class(base=Area3D)]
struct Enemy {
    #[export]
    player: Option<Gd<Node3D>>,

    raycast: OnReady<Gd<RayCast3D>>,
    muzzle_a: OnReady<Gd<AnimatedSprite3D>>,
    muzzle_b: OnReady<Gd<AnimatedSprite3D>>,

    health: f32,
    time: f64,
    target_position: Vector3,
    destroyed: bool,

    base: Base<Area3D>,
}

#[godot_api]
impl INode3D for Enemy {
    fn init(base: Base<Area3D>) -> Self {
        Self {
            player: None,

            raycast: OnReady::manual(),
            muzzle_a: OnReady::manual(),
            muzzle_b: OnReady::manual(),

            health: 100.0,
            time: 0.0,
            target_position: Vector3::new(0.0, 0.0, 0.0),
            destroyed: false,

            base,
        }
    }

    fn ready(&mut self) {
        let position = self.base().get_position();
        self.target_position = position;

        self.raycast.init(self.base().get_node_as("RayCast"));
        self.muzzle_a.init(self.base().get_node_as("MuzzleA"));
        self.muzzle_b.init(self.base().get_node_as("MuzzleB"));
    }

    fn process(&mut self, delta: f64) {
        let position = self.player.as_ref().map(|x| x.get_position());
        if let Some(position) = position {
            //godot_print!("position: {:?}", position);
            self.base_mut()
                .look_at_ex(position + Vector3::new(0.0, 0.5, 0.0))
                .use_model_front(true)
                .done();
        }
        self.target_position.y += ((self.time * 5.0).cos() * delta) as f32;

        self.time += delta;

        let target_position = self.target_position;
        self.base_mut().set_position(target_position);
    }
}

#[godot_api]
impl Enemy {
    #[func]
    fn damage(&mut self, amount: f32) {
        self.play_sound("sounds/enemy_hurt.ogg");
        self.health -= amount;
        if self.health < 0.0 {
            self.destroy();
        }
    }

    #[func]
    fn destroy(&mut self) {
        self.play_sound("sounds/enemy_destroy.ogg");
        self.destroyed = true;
        self.base_mut().queue_free();
    }

    #[func]
    fn _on_timer_timeout(&mut self) {
        self.raycast.force_raycast_update();

        if self.raycast.is_colliding() {
            let collider = self.raycast.get_collider();

            if let Some(mut collider) = collider {
                if collider.has_method("damage".into()) {
                    Self::play_default_animation(&mut self.muzzle_a);
                    Self::play_default_animation(&mut self.muzzle_b);
                    self.play_sound("sounds/enemy_attack.ogg");
                    collider.call("damage".into(), &[Variant::from(5.0)]);
                }
            }
        }
    }

    fn play_default_animation(animation: &mut Gd<AnimatedSprite3D>) {
        animation.set_frame(0);
        animation.play_ex().name("default".into()).done();
        let mut degrees = animation.get_rotation_degrees();
        degrees.z = rand::thread_rng().gen_range(-45.0..45.0);
        animation.set_rotation_degrees(degrees);
    }

    fn play_sound(&self, audio_path: &str) {
        let mut audio = self.base().get_node_as::<Node>("/root/Audio");
        audio.call("play".into(), &[Variant::from(GString::from(audio_path))]);
    }
}
