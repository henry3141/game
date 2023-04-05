use window::{TextureTrait,Window,self};

#[derive(Clone,Debug)]
struct Triangle {
    x: f32,
    y: f32,
    color: (u8,u8,u8),
    width: f32,
    height: f32,
}

impl TextureTrait for Triangle {
    fn triangles(&self) -> Vec<window::Triangle> {
        vec![window::Triangle {
            vertices: [
                (self.x as i32, self.y as i32),
                (self.x as i32 + self.width as i32, self.y as i32),
                (self.x as i32 + self.width as i32 / 2.0 as i32, self.y as i32 + self.height as i32),
            ],
            color: self.color,
        }]
    }

    fn clone_box(&self) -> Box<dyn TextureTrait> {
        Box::new(self.clone())
    }
}

impl Triangle {
    fn new(x: f32, y: f32, color: (u8,u8,u8), width: f32, height: f32) -> Triangle {
        Triangle {
            x,
            y,
            color,
            width,
            height,
        }
    }
}

fn main() {
    Window::run(Box::new(|api| {
        api.set_texture(0,Box::new(Triangle::new(100.0, 100.0, (255, 0, 0), 100.0, 100.0)));
        api.redraw();
        loop {
            api.redraw();
            api.recv();
            println!("len: {}", api.events.len());
        }
    }));
}
