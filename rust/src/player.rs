use godot::engine::input::MouseMode;
use godot::engine::tween::EaseType;
use godot::engine::utilities::{deg_to_rad, lerp, lerp_angle};
use godot::engine::{
    AnimatedSprite3D, CharacterBody3D, ICharacterBody3D, InputEvent, InputEventMouseMotion,
    MeshInstance3D, RayCast3D, Texture2D, TextureRect, Timer, Tween,
};
use godot::prelude::*;
use rand::Rng;

use crate::impact::Impact;
use crate::weapon::Weapon;

#[derive(GodotClass)]
#[class(base=CharacterBody3D)]
pub struct Player {
    #[export]
    movement_speed: f32,
    #[export]
    jump_strength: f32,

    #[export]
    weapons: Array<Gd<Weapon>>,

    weapon: Option<Gd<Weapon>>,
    weapon_index: usize,

    mouse_sensitivity: f32,
    gamepad_sensitivity: f32,

    mouse_captured: bool,

    movement_velocity: Vector3,
    rotation_target: Vector3,

    input_mouse: Vector2,

    health: i32,
    gravity: f32,

    previously_floored: bool,

    jump_single: bool,
    jump_double: bool,

    container_offset: Vector3,

    tween: Option<Gd<Tween>>,

    camera: OnReady<Gd<Camera3D>>,
    raycast: OnReady<Gd<RayCast3D>>,
    muzzle: OnReady<Gd<AnimatedSprite3D>>,
    container: OnReady<Gd<Node3D>>,
    sound_footsteps: OnReady<Gd<AudioStreamPlayer>>,
    blaster_cooldown: OnReady<Gd<Timer>>,

    #[export]
    crosshair: Option<Gd<TextureRect>>,

    base: Base<CharacterBody3D>,
}

#[godot_api]
impl ICharacterBody3D for Player {
    fn init(base: Base<CharacterBody3D>) -> Self {
        Self {
            movement_speed: 5.0,
            jump_strength: 8.0,

            weapons: Array::new(),

            weapon: None,
            weapon_index: 0,

            mouse_sensitivity: 700.0,
            gamepad_sensitivity: 0.075,

            mouse_captured: true,

            movement_velocity: Vector3::ZERO,
            rotation_target: Vector3::ZERO,

            input_mouse: Vector2::ZERO,

            health: 100,
            gravity: 0.0,

            previously_floored: false,

            jump_single: true,
            jump_double: true,
            container_offset: Vector3::new(1.2, -1.1, -2.75),

            tween: None,

            camera: OnReady::manual(),
            raycast: OnReady::manual(),
            muzzle: OnReady::manual(),
            container: OnReady::manual(),
            sound_footsteps: OnReady::manual(),
            blaster_cooldown: OnReady::manual(),

            crosshair: None,

            base,
        }
    }

    fn ready(&mut self) {
        self.camera.init(self.base().get_node_as("Head/Camera"));
        self.raycast
            .init(self.base().get_node_as("Head/Camera/RayCast"));
        self.muzzle.init(
            self.base()
                .get_node_as("Head/Camera/SubViewportContainer/SubViewport/CameraItem/Muzzle"),
        );
        self.container.init(
            self.base()
                .get_node_as("Head/Camera/SubViewportContainer/SubViewport/CameraItem/Container"),
        );
        self.sound_footsteps
            .init(self.base().get_node_as("SoundFootsteps"));
        self.blaster_cooldown
            .init(self.base().get_node_as("Cooldown"));

        Input::singleton().set_mouse_mode(MouseMode::CAPTURED);

        self.weapon = Some(self.weapons.get(self.weapon_index));
        self.initiate_change_weapon(self.weapon_index);
    }

    fn physics_process(&mut self, delta: f64) {
        self.handle_controls(delta);
        self.handle_gravity(delta);

        self.movement_velocity = self.base().get_transform().basis * self.movement_velocity;
        let mut applied_velocity = self
            .base()
            .get_velocity()
            .lerp(self.movement_velocity, (delta * 10.0) as f32);
        applied_velocity.y = -self.gravity;

        self.base_mut().set_velocity(applied_velocity);
        self.base_mut().move_and_slide();

        let mut camera_rotation = self.camera.get_rotation();
        camera_rotation.z = lerp_angle(
            camera_rotation.z as f64,
            self.input_mouse.x as f64 * 25.0 * delta,
            delta * 5.0,
        ) as f32;
        camera_rotation.x = lerp_angle(
            camera_rotation.x as f64,
            self.rotation_target.x as f64,
            delta * 25.0,
        ) as f32;
        self.camera.set_rotation(camera_rotation);

        let mut rotation = self.base().get_rotation();
        rotation.y = lerp_angle(
            rotation.y as f64,
            self.rotation_target.y as f64,
            delta * 25.0,
        ) as f32;
        self.base_mut().set_rotation(rotation);

        let position = lerp(
            Variant::from(self.container.get_position()),
            Variant::from(self.container_offset - (applied_velocity / 30.0)),
            Variant::from(delta * 10.0),
        );
        self.container.set_position(position.to::<Vector3>());

        self.sound_footsteps.set_stream_paused(true);

        if self.base().is_on_floor() {
            let velocity = self.base().get_velocity();
            if velocity.x.abs() > 1.0 || velocity.z.abs() > 1.0 {
                self.sound_footsteps.set_stream_paused(false);
            }
        }

        let mut camera_position = self.camera.get_position();
        camera_position.y = lerp(
            Variant::from(camera_position.y),
            Variant::from(0.0f32),
            Variant::from((delta * 5.0) as f32),
        )
        .to::<f32>();

        if self.base().is_on_floor() && self.gravity > 1.0 && !self.previously_floored {
            self.play_sound("sounds/land.ogg");
            camera_position.y = -0.1;
        }
        self.camera.set_position(camera_position);

        self.previously_floored = self.base().is_on_floor();

        let position = self.base().get_position();
        if position.y < -10.0 {
            self.base_mut().get_tree().unwrap().reload_current_scene();
        }
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if let Ok(event) = event.try_cast::<InputEventMouseMotion>() {
            if self.mouse_captured {
                let relative = event.get_relative();
                self.input_mouse = relative / self.mouse_sensitivity;

                self.rotation_target.y -= relative.x / self.mouse_sensitivity;
                self.rotation_target.x -= relative.y / self.mouse_sensitivity;
            }
        }
    }
}

#[godot_api]
impl Player {
    pub fn handle_controls(&mut self, _delta: f64) {
        let mut input = Input::singleton();

        if input.is_action_just_pressed("mouse_capture".into()) {
            input.set_mouse_mode(MouseMode::CAPTURED);
            self.mouse_captured = true;
        }

        if input.is_action_just_pressed("mouse_capture_exit".into()) {
            input.set_mouse_mode(MouseMode::VISIBLE);
            self.mouse_captured = false;

            self.input_mouse = Vector2::ZERO;
        }

        let input_vector = input.get_vector(
            "move_left".into(),
            "move_right".into(),
            "move_forward".into(),
            "move_back".into(),
        );

        self.movement_velocity =
            Vector3::new(input_vector.x, 0.0, input_vector.y).normalized() * self.movement_speed;

        let rotation_input = input.get_vector(
            "camera_right".into(),
            "camera_left".into(),
            "camera_down".into(),
            "camera_up".into(),
        );
        self.rotation_target -= Vector3::new(-rotation_input.y, -rotation_input.x, 0.0)
            .limit_length(Some(1.0))
            * self.gamepad_sensitivity;
        self.rotation_target.x = self
            .rotation_target
            .x
            .clamp(deg_to_rad(-90.0) as f32, deg_to_rad(90.0) as f32);

        self.action_shoot();

        if input.is_action_just_pressed("jump".into()) {
            if self.jump_single || self.jump_double {
                self.play_sound("sounds/jump_a.ogg, sounds/jump_b.ogg, sounds/jump_c.ogg");
            }

            if self.jump_double {
                self.gravity = -self.jump_strength;
                self.jump_double = false;
            }

            if self.jump_single {
                self.action_jump();
            }
        }

        self.action_weapon_toggle();
    }

    fn handle_gravity(&mut self, delta: f64) {
        self.gravity += 20.0 * delta as f32;
        if self.gravity > 0.0 && self.base().is_on_floor() {
            self.jump_single = true;
            self.gravity = 0.0;
        }
    }

    fn action_jump(&mut self) {
        self.gravity = -self.jump_strength;

        self.jump_single = false;
        self.jump_double = true;
    }

    fn action_shoot(&mut self) {
        let input = Input::singleton();
        if input.is_action_pressed("shoot".into()) {
            if !self.blaster_cooldown.is_stopped() {
                return;
            }

            if let Some(sound_shoot) = self
                .weapon
                .as_mut()
                .map(|x| x.get("sound_shoot".into()).to::<GString>())
            {
                let sound_shoot = sound_shoot.to_string();
                self.play_sound(&sound_shoot);
            }

            let mut container_position = self.container.get_position();
            container_position.z += 0.25;
            self.container.set_position(container_position);
            let mut camera_rotation = self.camera.get_rotation();
            camera_rotation.x += 0.025;
            self.camera.set_rotation(camera_rotation);
            self.movement_velocity += Vector3::new(
                0.0,
                0.0,
                self.weapon
                    .as_ref()
                    .unwrap()
                    .get("knockback".into())
                    .to::<f32>(),
            );

            self.muzzle.play_ex().name("default".into()).done();

            let mut rotation_degrees = self.muzzle.get_rotation_degrees();
            rotation_degrees.z = rand::thread_rng().gen_range(-45.0..45.0);
            self.muzzle.set_rotation_degrees(rotation_degrees);
            self.muzzle
                .set_scale(Vector3::ONE * rand::thread_rng().gen_range(0.40..0.75));
            self.muzzle.set_position(
                container_position
                    - self
                        .weapon
                        .as_ref()
                        .unwrap()
                        .get("muzzle_position".into())
                        .to::<Vector3>(),
            );

            self.blaster_cooldown
                .start_ex()
                .time_sec(
                    self.weapon
                        .as_ref()
                        .unwrap()
                        .get("cooldown".into())
                        .to::<f64>(),
                )
                .done();

            for _ in 0..self
                .weapon
                .as_ref()
                .unwrap()
                .get("shot_count".into())
                .to::<i32>()
            {
                let spread = self
                    .weapon
                    .as_ref()
                    .unwrap()
                    .get("spread".into())
                    .to::<f32>();
                let mut target_position = self.raycast.get_target_position();
                target_position.x = rand::thread_rng().gen_range(-spread..spread);
                target_position.y = rand::thread_rng().gen_range(-spread..spread);
                self.raycast.set_target_position(target_position);

                self.raycast.force_raycast_update();

                if !self.raycast.is_colliding() {
                    continue;
                }

                let mut collider = self.raycast.get_collider().unwrap();

                if collider.has_method("damage".into()) {
                    collider.call(
                        "damage".into(),
                        &[Variant::from(
                            self.weapon
                                .as_ref()
                                .unwrap()
                                .get("damage".into())
                                .to::<f32>(),
                        )],
                    );
                }

                let impact = load::<PackedScene>("res://objects/impact.tscn");
                let instance = impact.instantiate().unwrap();
                let mut impact = instance.cast::<Impact>();

                impact.play_ex().name("shot".into()).done();

                self.base()
                    .get_tree()
                    .unwrap()
                    .get_root()
                    .unwrap()
                    .add_child(impact.clone().upcast());

                impact.set_position(
                    self.raycast.get_collision_point()
                        + (self.raycast.get_collision_normal() / 10.0),
                );
                impact
                    .look_at_ex(self.camera.get_global_transform().origin)
                    .use_model_front(true)
                    .done();
            }
        }
    }

    fn action_weapon_toggle(&mut self) {
        let input = Input::singleton();
        if input.is_action_just_pressed("weapon_toggle".into()) {
            self.weapon_index = (self.weapon_index + 1) % self.weapons.len();
            self.initiate_change_weapon(self.weapon_index);

            self.play_sound("sounds/weapon_change.ogg");
        }
    }

    fn initiate_change_weapon(&mut self, weapon_index: usize) {
        self.weapon_index = weapon_index;

        let mut tween = self.base().get_tree().unwrap().create_tween().unwrap();
        tween.set_ease(EaseType::OUT_IN);
        tween.tween_property(
            self.container.clone().upcast(),
            "position".into(),
            Variant::from(self.container_offset - Vector3::new(0.0, 1.0, 0.0)),
            0.1,
        );
        tween.tween_callback(self.base().callable("change_weapon"));
        self.tween = Some(tween);
    }

    #[func]
    fn change_weapon(&mut self) {
        self.weapon = Some(self.weapons.get(self.weapon_index));

        for n in self.container.get_children().iter_shared() {
            self.container.remove_child(n);
        }

        let mut weapon_model = self
            .weapon
            .as_ref()
            .unwrap()
            .get("model".into())
            .to::<Option<Gd<PackedScene>>>()
            .unwrap()
            .instantiate()
            .unwrap()
            .cast::<Node3D>();
        self.container.add_child(weapon_model.clone().upcast());

        weapon_model.set_position(
            self.weapon
                .as_ref()
                .unwrap()
                .get("position".into())
                .to::<Vector3>(),
        );
        weapon_model.set_rotation_degrees(
            self.weapon
                .as_ref()
                .unwrap()
                .get("rotation".into())
                .to::<Vector3>(),
        );

        for child in weapon_model
            .find_children_ex("*".into())
            .type_("MeshInstance3D".into())
            .done()
            .iter_shared()
        {
            let mut child = child.cast::<MeshInstance3D>();
            child.set_layer_mask(2);
        }

        self.raycast.set_target_position(
            Vector3::new(0.0, 0.0, -1.0)
                * self
                    .weapon
                    .as_ref()
                    .unwrap()
                    .get("max_distance".into())
                    .to::<f32>(),
        );
        self.crosshair.as_mut().unwrap().set_texture(
            self.weapon
                .as_ref()
                .unwrap()
                .get("crosshair".into())
                .to::<Option<Gd<Texture2D>>>()
                .unwrap(),
        );
    }

    #[func]
    fn damage(&mut self, amount: f32) {
        self.health -= amount as i32;
        let health = self.health;
        self.base_mut()
            .emit_signal("health_updated".into(), &[Variant::from(health)]);

        if self.health < 0 {
            self.base().get_tree().unwrap().reload_current_scene();
        }
    }

    fn play_sound(&self, audio_path: &str) {
        let mut audio = self.base().get_node_as::<Node>("/root/Audio");
        audio.call("play".into(), &[Variant::from(GString::from(audio_path))]);
    }

    #[signal]
    fn health_updated(health: i32);
}
