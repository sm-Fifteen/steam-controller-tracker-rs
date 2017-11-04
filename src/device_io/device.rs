use libusb::Device;
use std::sync::mpsc;
use std::time::Duration;
use std::error::Error;
use std::fmt;
use music::{Note, Instrument};

pub trait MusicDevice<PacketType> {
	fn get_io_queue(&self) -> mpsc::SyncSender<PacketType>;
	fn channel_count(&self) -> usize;
	fn packet_from_note(channel: usize, note: &Note, instr: &Instrument, max_duration: Option<Duration>) -> Result<PacketType, InstrumentError>;
}

pub trait USBDeviceWrapper<'context> : Sized {
	fn device_matcher(device: Device<'context>) -> Option<Self>;
}

pub struct USBControlTransfer {
	request_type: u8,
	request: u8,
	value: u16,
	index: u16,
	buf: Vec<u8>,
	timeout: Duration,
}

#[derive(Debug)]
pub struct InstrumentError {
	channel: usize,
	note: Note,
	instr: Instrument,
	max_duration: Option<Duration>,
	reason_msg: String,
}

impl Error for InstrumentError {
	fn description(&self) -> &str {
        &self.reason_msg
    }
}

impl fmt::Display for InstrumentError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to convert note into device packet.")
    }
}