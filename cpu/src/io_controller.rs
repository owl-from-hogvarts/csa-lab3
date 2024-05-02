// data in
// data out

// controls
// IO
// IO write

// device
// send buffer
// receive buffer
// device has logic to determine when to start outputting data

use std::{collections::HashMap, io::Write, mem::size_of};

use isa::{RawAddress, RawPort};

pub struct IOController {
    devices: HashMap<RawPort, Box<dyn Device>>,
}

pub struct SimpleInputOutput {
    output: Vec<u8>,
    cursor: usize,
}

impl SimpleInputOutput {
    pub fn new(string: String) -> Self {
        // place string length beforehand
        let length: RawAddress = string
            .len()
            .try_into()
            .expect("Size of input string should fit into size of pointer which is 16 bits");

        let mut output = Vec::with_capacity(string.len() + size_of::<RawAddress>());
        output[0..size_of::<RawAddress>()].copy_from_slice(&length.to_be_bytes());
        output[size_of::<RawAddress>()..].copy_from_slice(string.as_bytes());

        Self { output, cursor: 0 }
    }
}

impl Device for SimpleInputOutput {
    fn read_from_device(&mut self) -> u8 {
        let data = self.output[self.cursor];
        self.cursor += 1;

        data
    }

    fn write_to_device(&mut self, payload: u8) {
        std::io::stdout().write(&[payload]).unwrap();
    }
}

pub trait Device {
    fn read_from_device(&mut self) -> u8;
    fn write_to_device(&mut self, payload: u8);
}

impl IOController {
    pub fn new() -> Self {
        IOController {
            devices: HashMap::new(),
        }
    }

    pub fn connect(mut self, address: RawPort, device: Box<dyn Device>) -> Self {
        self.devices.insert(address, device);

        self
    }

    pub fn read(&mut self, device_address: RawPort) -> u8 {
        self.devices
            .get_mut(&device_address)
            .map_or(0, |device| device.read_from_device())
    }

    pub fn write(&mut self, device_address: RawPort, payload: u8) {
        self.devices
            .get_mut(&device_address)
            .map(|device| device.write_to_device(payload));
    }
}