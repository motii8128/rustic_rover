#[derive(Debug, Clone)]
pub enum SerialMessage
{
    SetIM920(bool),
    SetSmooth(bool),
    SerialScan,
    SerialStart,
    PortSelected(String),
    SmoothValue(i32),
}

#[derive(Debug, Clone, Copy)]
pub struct Packet
{
    pub id:u16,
    pub x:i32,
    pub y:i32,
    pub ro:i32,
    pub m1:i32,
    pub m2:i32,
}

impl Packet {
    pub fn new(id_:u16, x_:i32, y_:i32, ro_:i32, m1_:i32, m2_:i32)->Packet
    {
        Packet { id: id_, x: x_, y: y_, ro: ro_, m1: m1_, m2: m2_ }
    }
    pub fn get_string(&self)->String
    {
        format!("ID:{}   [x:{:3},y:{:3},ro:{:3},m1:{:3},m2:{:3}]", self.id,self.x, self.y, self.ro, self.m1, self.m2)
    }
}