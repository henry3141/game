//allow unboxed closures because of the run function
use std::collections::HashMap;
use utils::Channel;
use pixels::{Pixels, SurfaceTexture};
use winit::event_loop::EventLoop;

pub struct Window{
    width: u32,
    height: u32,
    textures: HashMap<i32,Texture>,
    pixels: Pixels,
    window: winit::window::Window,
}

impl Window {
    pub fn run(func:Box<dyn FnOnce(&mut WindowAPI) + Send>) {
        let (channel,channel2) = Channel::new();
        let mut api = WindowAPI::new(channel2);
        std::thread::spawn(move || {
            api.await_start();
            func(&mut api);
        });
        let (width, height) = (800, 600);
        let (window,event_loop) = Window::new(width, height);
        let (ev_channel, ev_channel2) = Channel::new();
        std::thread::spawn(move || {
            let ev:Channel<EvMSG> = ev_channel2;
            let api = channel;
            let mut window = window;
            let mut event_handling = true;
            let mut close = false;
            api.send(Msg::Start);
            loop {
                let mut ev_events = vec![];
                while let Some(ev) = ev.try_recv() {
                    ev_events.push(ev);
                }
                #[allow(unused_assignments)]
                let mut events = vec![];
                if event_handling {
                    events = ev_events.into_iter().map(|ev| match ev {
                        EvMSG::Resize(width,height) => {
                            window.width = width;
                            window.height = height;
                            window.pixels.resize_surface(width, height).unwrap();
                            Event::Resize(width,height)
                        },
                        EvMSG::Redraw => {
                            window.redraw();
                            Event::Redraw
                        },
                        EvMSG::Key(key) => Event::Key(key),
                        EvMSG::MouseMove(x,y) => Event::MouseMove(x,y),
                        EvMSG::MouseButton(button) => Event::MouseButton(button),
                        EvMSG::Exit => {
                            close = true;
                            Event::Exit
                        }
                        _ => Event::None,
                    }).collect();
                } else {
                    events = ev_events.into_iter().map(|ev| match ev {
                        EvMSG::Resize(width,height) => Event::Resize(width,height),
                        EvMSG::Redraw => Event::Redraw,
                        EvMSG::Key(key) => Event::Key(key),
                        EvMSG::MouseMove(x,y) => Event::MouseMove(x,y),
                        EvMSG::MouseButton(button) => Event::MouseButton(button),
                        EvMSG::Exit => {
                            close = true;
                            Event::Exit
                        }
                        _ => Event::None,
                    }).collect();
                }
                if events.len() > 0 {
                    for i in events {
                        api.send(Msg::Event(i));
                    }
                }
                if close {
                    api.send(Msg::Exit);
                    break;
                }
                if let Some(msg) = api.try_recv() {
                    match msg {
                        Msg::Start => {},
                        Msg::Exit => {
                            ev.send(EvMSG::Exit);
                            break;
                        },
                        Msg::Event(_) => {}
                        Msg::SetEventHandling(b) => {
                            event_handling = b;
                        }
                        Msg::SetTexture(id,texture) => {
                            window.textures.insert(id,texture);
                        }
                        Msg::RemoveTexture(id) => {
                            window.textures.remove(&id);
                        }
                        Msg::Resize(width, height) => {
                            window.width = width;
                            window.height = height;
                            window.pixels.resize_surface(width, height).unwrap();
                        }
                        Msg::Redraw => {
                            window.redraw();
                        }
                    }
                }
            }
        });
        event_loop.run(move |event, _, control_flow| {
            if let Some(ev) = ev_channel.try_recv() {
                match ev {
                    EvMSG::Exit => {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                    }
                    _ => {}
                }
            }

            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => {
                        ev_channel.send(EvMSG::Exit);
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                    }
                    winit::event::WindowEvent::Resized(size) => {
                        ev_channel.send(EvMSG::Resize(size.width, size.height));
                    }
                    _ => {}
                },
                winit::event::Event::RedrawRequested(_) => {
                    ev_channel.send(EvMSG::Redraw);
                }
                winit::event::Event::DeviceEvent { event, .. } => match event {
                    winit::event::DeviceEvent::Key(winit::event::KeyboardInput {
                        virtual_keycode: Some(key),
                        ..
                    }) => {
                        ev_channel.send(EvMSG::Key(key));
                    }
                    winit::event::DeviceEvent::MouseMotion { delta } => {
                        ev_channel.send(EvMSG::MouseMove(delta.0 as u32, delta.1 as u32));
                    }
                    winit::event::DeviceEvent::Button { button, state:_state } => {
                        ev_channel.send(EvMSG::MouseButton(button));
                    }
                    _ => {}
                },
                _ => {}
            }
        });

    }

    fn new(width: u32, height: u32) -> (Self,EventLoop<()>) {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title("Pixels")
            .with_inner_size(winit::dpi::LogicalSize::new(width, height))
            .build(&event_loop)
            .unwrap();
        let surface_texture = SurfaceTexture::new(width, height, &window);
        let pixels = Pixels::new(width, height, surface_texture).unwrap();
        (Self {
            width,
            height,
            textures: HashMap::new(),
            pixels,
            window,
        },event_loop)
    }

    fn redraw(&mut self) {
        //set internal window size
        self.window.set_inner_size(winit::dpi::LogicalSize::new(self.width, self.height));
        let mut triangles = vec![];
        for (_,v) in &self.textures {
            for i in v.triangles() {
                triangles.push(i);
            }
        }
        //RENDER triangles
        self.pixels.clear_color(wgpu::Color::BLACK);
        for x in 0..self.width {
            for y in 0..self.height {
                let mut color = (0,0,0);
                for triangle in &triangles {
                    if Self::collides((x as i32,y as i32),triangle.clone()) {
                        color = triangle.color;
                        break;
                    }
                }
                self.set_pixel(x,y,color);
            }
        }
        self.pixels.render().unwrap();
    }

    fn to_f32(one:(i32,i32)) -> (f32,f32) {
        let (x,y) = one;
        (x as f32,y as f32)
    }

    fn collides(point:(i32,i32),triangle:Triangle) -> bool {
        let (x,y) = Self::to_f32(point);
        let (x1,y1) = Self::to_f32(triangle.vertices[0]);
        let (x2,y2) = Self::to_f32(triangle.vertices[1]);
        let (x3,y3) = Self::to_f32(triangle.vertices[2]);
        let d = (x-x1)*(y2-y1) - (y-y1)*(x2-x1);
        let a = (x-x2)*(y3-y2) - (y-y2)*(x3-x2);
        let b = (x-x3)*(y1-y3) - (y-y3)*(x1-x3);
        if d > 0.0 && a > 0.0 && b > 0.0 {
            return true;
        }
        if d < 0.0 && a < 0.0 && b < 0.0 {
            return true;
        }
        return false;
    }

    #[inline]
    fn set_pixel(&mut self, x: u32, y: u32, color: (u8,u8,u8)) {
        let frame = self.pixels.frame_mut();
        let (r,g,b) = color;
        let index = (y * self.width + x) as usize * 4;
        frame[index] = r;
        frame[index + 1] = g;
        frame[index + 2] = b;
    }
}

#[derive(Clone)]
pub enum Msg {
    Start,
    Event(Event),
    Exit,
    SetEventHandling(bool),
    Resize(u32,u32),
    Redraw,
    SetTexture(i32,Texture),
    RemoveTexture(i32),
}

impl PartialEq for Msg {
    fn eq(&self, other: &Self) -> bool {
        match (self,other) {
            (Msg::Start,Msg::Start) => true,
            (Msg::Event(i),Msg::Event(i2)) => i == i2,
            (Msg::Exit,Msg::Exit) => true,
            (Msg::SetEventHandling(b),Msg::SetEventHandling(b2)) => b == b2,
            (Msg::Resize(w,h),Msg::Resize(w2,h2)) => w == w2 && h == h2,
            (Msg::Redraw,Msg::Redraw) => true,
            (Msg::SetTexture(id,texture),Msg::SetTexture(id2,texture2)) => id == id2 && texture == texture2,
            (Msg::RemoveTexture(id),Msg::RemoveTexture(id2)) => id == id2,
            _ => false,
        }
    }
}

pub enum EvMSG {
    Resize(u32,u32),
    Redraw,
    Key(winit::event::VirtualKeyCode),
    MouseMove(u32,u32),
    MouseButton(u32),
    Exit,
    Stop,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum Event {
    Resize(u32,u32),
    Redraw,
    Key(winit::event::VirtualKeyCode),
    MouseMove(u32,u32),
    MouseButton(u32),
    Exit,
    None,
}

pub struct WindowAPI {
    pub events:Vec<(Msg,std::time::Instant)>,
    channel:Channel<Msg>,
    too_old:std::time::Duration,
}

impl WindowAPI {
    fn new(channel:Channel<Msg>) -> Self {
        Self {
            events:Vec::new(),
            channel,
            too_old:std::time::Duration::from_millis(1000),
        }
    }

    pub fn set_decay_time(&mut self, time:std::time::Duration) {
        self.too_old = time;
    }

    fn await_start(&self) {
        while self.channel.recv() != Msg::Start {}
    }

    fn send(&self, msg:Msg) {
        self.channel.send(msg);
    }

    pub fn recv(&mut self) {
        while let Some(ev) = self.channel.try_recv() {
            self.events.push((ev,std::time::Instant::now()));
        }
    }

    pub fn redraw(&self) {
        self.send(Msg::Redraw);
    }

    pub fn set_event_handling(&self, b:bool) {
        self.send(Msg::SetEventHandling(b));
    }

    pub fn set_texture(&self, id:i32, texture:Texture) {
        self.send(Msg::SetTexture(id,texture));
    }

    pub fn remove_texture(&self, id:i32) {
        self.send(Msg::RemoveTexture(id));
    }

    pub fn exit(&self) {
        self.send(Msg::Exit);
    }

    pub fn resize(&self, width:u32, height:u32) {
        self.send(Msg::Resize(width,height));
    }

    fn decay(&mut self) {
        let now = std::time::Instant::now();
        self.events.retain(|(_,t)| now.duration_since(*t) < self.too_old);
    }

    pub fn resized(&mut self) -> Option<(u32,u32)> {
        self.recv();
        self.decay();
        let mut return_ = None;
        self.events = self.events.drain(..).filter(|(ev,_)| {
            match ev {
                Msg::Event(Event::Resize(width, height)) => {
                    return_ = Some((*width,*height));
                    false
                },
                _ => true,
            }
        }).collect();
        return_
    }

    pub fn redrawn(&mut self) -> bool {
        self.recv();
        self.decay();
        let mut return_ = false;
        self.events = self.events.drain(..).filter(|(ev,_)| {
            match ev {
                Msg::Event(Event::Redraw) => {
                    return_ = true;
                    false
                },
                _ => true,
            }
        }).collect();
        return_
    }

    pub fn mouse_moves(&mut self) -> Vec<(u32,u32)> {
        self.recv();
        self.decay();
        let mut return_ = Vec::new();
        self.events = self.events.drain(..).filter(|(ev,_)| {
            match ev {
                Msg::Event(Event::MouseMove(x,y)) => {
                    return_.push((*x,*y));
                    false
                },
                _ => true,
            }
        }).collect();
        return_
    }

    pub fn mouse_presses(&mut self) -> Vec<u32> {
        self.recv();
        self.decay();
        let mut return_ = Vec::new();
        self.events = self.events.drain(..).filter(|(ev,_)| {
            match ev {
                Msg::Event(Event::MouseButton(b)) => {
                    return_.push(*b);
                    false
                },
                _ => true,
            }
        }).collect();
        return_
    }
}

#[derive(Clone,PartialEq,Eq)]
pub struct Triangle {
    pub vertices:[(i32,i32);3],
    pub color:(u8,u8,u8),
}

pub trait TextureTrait:Send + 'static {
    fn triangles(&self) -> Vec<Triangle>;
    fn clone_box(&self) -> Box<dyn TextureTrait>;
}

impl Clone for Texture {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl PartialEq for Texture {
    fn eq(&self, other: &Self) -> bool {
        self.triangles() == other.triangles()
    }
}

pub type Texture = Box<dyn TextureTrait>;

impl Drop for WindowAPI {
    fn drop(&mut self) {
        self.exit();
    }
}