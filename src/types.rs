pub type Efm8Cmd = Vec<u8>;
pub type Efm8Cmds = Vec<Efm8Cmd>;

pub struct Config {
    pub path: String,
    pub vid: u16,
    pub pid: u16,
}
