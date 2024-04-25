// data in
// data out

// controls
// IO
// IO write

// device
// send buffer
// receive buffer
// device has logic to determine when to start outputting data

use std::collections::HashMap;

use isa::RawPort;

pub struct IOController {
    devices: HashMap<RawPort, Box<dyn Device>>,
}

pub trait Device {
    fn read_from_device(&self) -> u8;
    fn write_to_device(&self, payload: u8);
}

impl IOController {
    pub fn new() -> Self {
        IOController {
            devices: HashMap::new(),
        }
    }

    pub fn connect(&mut self, address: RawPort, device: impl Device) {
        self.devices.insert(address, Box::new(device));
    }

    pub fn read(&self, device_address: RawPort) -> u8 {
        self.devices[&device_address].read_from_device()
    }

    pub fn write(&self, device_address: RawPort, payload: u8) {
        self.devices[&device_address].write_to_device(payload);
    }
}
