
#[macro_export]
macro_rules! key_event {
    ($k:ident, $a:ident) => (KeyEvent {
        state: winit::event::ElementState::$a,
        physical_key: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::$k),
        repeat: false,
        ..
    })
}

#[macro_export]
macro_rules! key_press {
    ($k:ident) => (key_event!($k, Pressed))
}

#[macro_export]
macro_rules! key_release {
    ($k:ident) => (key_event!($k, Released))
}

