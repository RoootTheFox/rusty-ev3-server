#[derive(Clone)]
pub(crate) struct Ev3Connection {
    pub name:String,
    pub connected:bool,
    pub last_seen:u64
}