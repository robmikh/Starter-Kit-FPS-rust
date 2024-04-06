use godot::prelude::*;
use godot::engine::{AnimatedSprite3D, IAnimatedSprite3D};

#[derive(GodotClass)]
#[class(base=AnimatedSprite3D)]
pub struct Impact {
    base: Base<AnimatedSprite3D>
}


#[godot_api]
impl IAnimatedSprite3D for Impact {
    fn init(base: Base<AnimatedSprite3D>) -> Self {
        Self {
            base,
        }
    }
}

#[godot_api]
impl Impact {
    #[func]
    fn _on_animation_finished(&mut self) {
        self.base_mut().queue_free();
    }
}