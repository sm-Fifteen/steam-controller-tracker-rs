use libusb::{Device, DeviceHandle};
use std::sync::mpsc;
use std::time::Duration;
use std::error::Error;
use std::fmt;
use crossbeam;
use music::{Note, Instrument};

pub trait MusicDevice {
	type PacketType;

	fn get_io_queue(&self) -> mpsc::SyncSender<Self::PacketType>;
	fn channel_count(&self) -> usize;
	fn packet_from_note(channel: usize, note: &Note, instr: &Instrument, max_duration: Option<Duration>) -> Result<Self::PacketType, InstrumentError>;
}

pub trait USBDeviceWrapper<'context> : Sized {
	fn device_matcher(libusb_scope: &crossbeam::Scope<'context>, device: Device<'context>) -> Option<Self>;

	fn send_control(device: &DeviceHandle, control: USBControlTransfer) -> Result<usize, ::libusb::Error> {
		device.write_control(control.request_type, control.request, control.value, control.index, control.buf.as_slice(), control.timeout)
	}

	fn start_rx_thread(libusb_scope: &crossbeam::Scope<'context>, rx: mpsc::Receiver<USBControlTransfer>, handle: DeviceHandle<'context>) -> crossbeam::ScopedJoinHandle<::libusb::Error> {
		libusb_scope.spawn(move || {
			let final_val = loop {
				if let Ok(control) = rx.recv() {
					let status = Self::send_control(&handle, control).err();

					if let Some(err) = status {
						break err;
					}
				} else {
					// tx has been freed
					break ::libusb::Error::Success;
				}
			};

			final_val
		})
	}
}

pub struct USBControlTransfer {
	pub request_type: u8,
	pub request: u8,
	pub value: u16,
	pub index: u16,
	pub buf: Vec<u8>,
	pub timeout: Duration,
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