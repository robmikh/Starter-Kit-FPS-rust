use godot::prelude::*;
use godot::engine::{CanvasLayer, ICanvasLayer, Label};

#[derive(GodotClass)]
#[class(base=CanvasLayer)]
pub struct HUD {
    base: Base<CanvasLayer>
}


#[godot_api]
impl ICanvasLayer for HUD {
    fn init(base: Base<CanvasLayer>) -> Self {
        Self {
            base,
        }
    }
}

#[godot_api]
impl HUD {
    #[func]
    fn _on_health_updated(&mut self, health: i32) {
        let mut health_text_node = self.base().get_node_as::<Label>("Health");
        health_text_node.set_text(format!("{}%", health).into());
    }
}