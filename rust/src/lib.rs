mod audio;
mod enemy;
mod hud;
mod impact;
mod player;
mod weapon;

use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
