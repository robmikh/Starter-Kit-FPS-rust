mod enemy;
mod audio;
mod hud;
mod weapon;

use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
