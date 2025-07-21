

use nannou::prelude::*;
use bytemuck::{Pod, Zeroable};


fn main() {
    nannou::app(model).run();
}


struct Model {
    window_id: WindowId,
}

fn model(app: &App) -> Model {
    let window_id = app.new_window().build().unwrap();
    let window = app.window(window_id).unwrap();

    Model {
        window_id,
    }
}