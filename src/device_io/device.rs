use libusb::{Device, DeviceHandle, Error as USBError};
use std::time::Duration;
use std::error::Error;
use std::fmt;
use crossbeam;
use music::{Note, Instrument};

pub trait MusicDevice {
	type PacketType;
	type IOErrorType: Error;

	fn init(&mut self) -> Result<(), Self::IOErrorType>;
	fn send_packet(&self, transfer: USBControlTransfer) -> Result<(), Self::IOErrorType>;
	fn channel_count(&self) -> usize;
	fn packet_from_note(channel: usize, note: &Note, instr: &Instrument, max_duration: Option<Duration>) -> Result<Self::PacketType, InstrumentError>;
}

pub(super) trait USBDeviceWrapper<'context> : Sized {
	fn device_matcher(libusb_scope: &crossbeam::Scope<'context>, device: Device<'context>) -> Option<Self>;
	fn send_control(handle: &DeviceHandle<'context>, control: USBControlTransfer) -> Result<usize, USBError> {
		handle.write_control(control.request_type, control.request, control.value, control.index, control.buf.as_slice(), control.timeout)
	}
}

pub struct USBControlTransfer {
	request_type: u8,
	request: u8,
	value: u16,
	index: u16,
	buf: Vec<u8>,
	timeout: Duration,
}

impl USBControlTransfer {
	pub fn new(request_type: u8, request: u8, value: u16, index: u16, buf: Vec<u8>, timeout: Duration) -> USBControlTransfer {
		Self {
			request_type,
			request,
			value,
			index,
			buf,
			timeout,
		}
	}
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