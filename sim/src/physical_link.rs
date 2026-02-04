pub const PRESS_DURATION: std::time::Duration = std::time::Duration::from_millis(150);
pub const SERIAL_PORT: &'static str = "/dev/ttyACM0";

pub struct Link {
    port: Box<dyn serialport::SerialPort>,
}

impl Link {
    pub fn new() -> Result<Self, ()> {
        let port = serialport::new(SERIAL_PORT, 115_200)
            .timeout(std::time::Duration::from_millis(10))
            .dtr_on_open(true)
            .open()
            .expect("Failed to open port");

        Ok(Self { port })
    }

    pub fn signal(&mut self, state: bool) {
        let message = if state { b'o' } else { b'-' };
        self.port
            .write(&[message, b'\r', b'\n'])
            .expect("wiring failed");
        self.port.flush().expect("flushing failed");
    }
}
