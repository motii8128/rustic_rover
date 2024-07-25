use crate::dualshock::ControllerConnectionType;

#[derive(Debug, Clone, Copy)]
pub struct DualShock4
{
    pub mode:ControllerConnectionType,
    pub state:bool,
    pub sticks:JoyStick,
    pub btns:Buttons,
    pub dpad:Dpad
}

impl DualShock4 {
    pub fn new()->DualShock4
    {
        DualShock4 { mode:ControllerConnectionType::BLE,state:true, sticks: JoyStick::new(), btns: Buttons::new(), dpad: Dpad::new() }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct JoyStick
{
    pub left_x:f32,
    pub left_y:f32,
    pub right_x:f32,
    pub right_y:f32,
}
impl JoyStick {
    pub fn new()->JoyStick
    {
        JoyStick { left_x: 0.0, left_y: 0.0, right_x: 0.0, right_y: 0.0 }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Dpad
{
    pub up_key:bool,
    pub down_key:bool,
    pub left_key:bool,
    pub right_key:bool,   
}
impl Dpad {
    pub fn new()->Dpad
    {
        Dpad { up_key: false, down_key: false, left_key: false, right_key: false }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Buttons
{
    pub circle:bool,
    pub cross:bool,
    pub triangle:bool,
    pub cube:bool,
    pub r1:bool,
    pub r2:bool,
    pub l1:bool,
    pub l2:bool,
    pub left_push:bool,
    pub right_push:bool
}
impl Buttons {
    pub fn new()->Buttons
    {
        Buttons { circle: false, cross: false, triangle: false, cube: false, r1: false, r2: false, l1: false, l2: false, left_push: false, right_push: false }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Packet
{
    pub x:i32,
    pub y:i32,
    pub ro:i32,
    pub m1:i32,
    pub m2:i32,
}

impl Packet {
    pub fn new()->Packet
    {
        Packet { x: 0, y: 0, ro: 0, m1: 0, m2: 0 }
    }

    pub fn from_value(x_:i32, y_:i32, ro_:i32, m1_:i32, m2_:i32)->Packet
    {
        Packet { x: x_, y: y_, ro: ro_, m1: m1_, m2: m2_ }
    }
}