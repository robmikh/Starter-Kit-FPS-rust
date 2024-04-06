mod enemy;
mod audio;
mod hud;
mod weapon;
mod impact;

use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
