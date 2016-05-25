
use std::collections::HashSet;
use glutin::{Event, Window, MouseButton, VirtualKeyCode};
use glutin::ElementState::{Pressed, Released};
use ecs;

#[derive(Clone, Debug)]
pub struct Events {
    /// the last known window dimensions
    pub window_size: (u32, u32),
    /// the last known mouse position
    pub mouse_position: (i32, i32),
    /// list of events collected during the frame
    pub events: Vec<Event>,
    /// set of keys that are depressed
    pub key_down: HashSet<VirtualKeyCode>,
    /// set of mouse buttons that are depressed
    pub button_down: HashSet<MouseButton>,
    /// running flag
    pub running: bool
}

impl Events {
    pub fn new(window: &Window) -> Events {
        Events {
            window_size: window.get_inner_size_points().unwrap_or((800, 600)),
            mouse_position: (0, 0),
            events: vec![],
            key_down: HashSet::new(),
            button_down: HashSet::new(),
            running: true
        }
    }

    pub fn next_frame(&mut self, window: &Window) {
        self.events.clear();

        for e in window.poll_events() {
            match e {
                Event::Closed => self.running = false,
                Event::MouseMoved(x, y) => self.mouse_position = (x, y),
                Event::Resized(w, h) => self.window_size = (w, h),
                Event::KeyboardInput(Pressed, _, Some(key)) => {
                    self.key_down.insert(key);
                }
                Event::KeyboardInput(Released, _, Some(ref key)) => {
                    self.key_down.remove(key);
                }
                Event::MouseInput(Pressed, key) => {
                    self.button_down.insert(key);
                }
                Event::MouseInput(Released, ref key) => {
                    self.button_down.remove(key);
                }
                _ => ()
            }
            self.events.push(e);
        }
    }

    pub fn is_key_down(&self, key: VirtualKeyCode) -> bool {
        self.key_down.contains(&key)
    }
}

impl ecs::Component for Events {
    type Storage = Events;
}

/// This is a hack to allow the Events to be shared as part of the ECS's
/// storage pool
impl ecs::UnprotectedStorage<Events> for Events {
    fn new() -> Events {
        Events {
            window_size: (0, 0),
            mouse_position: (0, 0),
            events: vec![],
            key_down: HashSet::new(),
            button_down: HashSet::new(),
            running: true
        }
    }
    unsafe fn get(&self, _: ecs::Index) -> &Events {
        panic!("events it not indented to be used as storage");
    }
    unsafe fn get_mut(&mut self, _: ecs::Index) -> &mut Events {
        panic!("events it not indented to be used as storage");
    }
    unsafe fn insert(&mut self, _: ecs::Index, _: Events) {
        panic!("events it not indented to be used as storage");
    }
    unsafe fn remove(&mut self, _: ecs::Index) -> Events {
        panic!("events it not indented to be used as storage");
    }
    unsafe fn clean<F>(&mut self, _: F)
        where F: Fn(ecs::Index) -> bool
    {
        panic!("events it not indented to be used as storage");
    }
}
