use std::collections::VecDeque;

use godot::engine::{Node, ResourceLoader};
use godot::prelude::*;
use rand::Rng;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct AudioBase {
    num_players: i32,
    bus: &'static str,
    available: VecDeque<Gd<AudioStreamPlayer>>,
    queue: VecDeque<String>,

    base: Base<Node>,
}

#[godot_api]
impl INode for AudioBase {
    fn init(base: Base<Node>) -> Self {
        Self {
            num_players: 12,
            bus: "master",
            available: VecDeque::new(),
            queue: VecDeque::new(),

            base,
        }
    }

    fn ready(&mut self) {
        for _ in 0..self.num_players {
            let mut p = AudioStreamPlayer::new_alloc();
            self.base_mut().add_child(p.clone().upcast());

            p.set_volume_db(-10.0);
            let bind_p = p.clone();
            p.connect(
                "finished".into(),
                self.base()
                    .callable("_on_stream_finished")
                    .bindv((&[Variant::from(bind_p)]).into()),
            );
            p.set_bus(self.bus.into());

            self.available.push_back(p);
        }
    }

    fn process(&mut self, _delta: f64) {
        if !self.queue.is_empty() && !self.available.is_empty() {
            let queued_front = self.queue.pop_front().unwrap();
            let mut available_front = self.available.pop_front().unwrap();
            let resource = ResourceLoader::singleton()
                .load(queued_front.into())
                .unwrap();
            available_front.set_stream(resource.cast());
            available_front.play();
            available_front.set_pitch_scale(rand::thread_rng().gen_range(0.9..1.1));
        }
    }
}

#[godot_api]
impl AudioBase {
    #[func]
    pub fn play(&mut self, sound_path: GString) {
        let sound_path = sound_path.to_string();
        let sounds = sound_path.split(",").collect::<Vec<_>>();
        self.queue.push_back(format!(
            "res:///{}",
            sounds[rand::thread_rng().gen_range(0..sounds.len())].trim()
        ));
    }

    #[func]
    fn _on_stream_finished(&mut self, stream: Gd<AudioStreamPlayer>) {
        self.available.push_back(stream);
    }
}
