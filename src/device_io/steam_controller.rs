use super::device::{USBDeviceWrapper, MusicDevice, USBControlTransfer, InstrumentError};
use libusb;
use crossbeam;
use std::sync::mpsc;
use std::time::Duration;
use byteorder::{LittleEndian, WriteBytesExt};
use music::{Note, Instrument};

/// Touch feedback is 0, priority 1 means notes don't
/// get interrupted when the user touches the controller
const NOTE_PRIORITY:u8 = 1;
const REPEAT_FOREVER:u16 = 0x7FFF;

type IO_Queue = (mpsc::SyncSender<USBControlTransfer>, crossbeam::ScopedJoinHandle<libusb::Error>);

pub struct SteamController<'context> {
	device: libusb::Device<'context>,
	io_queue: Option<IO_Queue>,
}

// I promise I won't have rx_thread used in a non-thread safe way.
// TODO: Figure out a way to shut down the thread in case of error
unsafe impl<'a> Sync for SteamController<'a>{}

impl<'context> USBDeviceWrapper<'context> for SteamController<'context> {
	fn device_matcher(libusb_scope: &crossbeam::Scope<'context>, device: libusb::Device<'context>) -> Option<Self> {
		// The returned device should be opened and ready to use
		let desc = device.device_descriptor().ok()?;

		if desc.vendor_id() == 0x28de && desc.product_id() == 0x1102 {
			if let Ok(mut handle) = device.open() {
				let device = SteamController{
					device,
					io_queue: None,
				};

				Some(device.init_io_queue(libusb_scope, handle))
			} else {
				None
			}
		} else {
			None
		}
	}
}

impl<'context> MusicDevice for SteamController<'context> {
	type PacketType = USBControlTransfer;
	
	fn get_io_queue(&self) -> mpsc::SyncSender<USBControlTransfer> {
		if let Some((ref tx, _)) = self.io_queue {
			tx.clone()
		} else {
			panic!("IO queue has not been opened")
		}
		
	}

	fn channel_count(&self) -> usize {
		2
	}

	fn packet_from_note(channel: usize, note: &Note, instr: &Instrument, max_duration: Option<Duration>) -> Result<USBControlTransfer, InstrumentError> {
		// TODO : move get_periods_for_note to a more sensible place, use a match here.
		if let Some(duration) = max_duration {
			let (hi_period, lo_period, cycle_count) = instr.get_periods_for_note_with_duration(note, duration);
			Ok(Self::packet_from_raw(channel as u8, hi_period, lo_period, cycle_count))
		} else {
			let (hi_period, lo_period) = instr.get_periods_for_note(note);
			Ok(Self::packet_from_raw(channel as u8, hi_period, lo_period, REPEAT_FOREVER))
		}
	}
}

impl<'context> SteamController<'context> {
	fn init_io_queue(mut self, libusb_scope: &crossbeam::Scope<'context>, mut handle: libusb::DeviceHandle<'context>) -> Self {
		handle.detach_kernel_driver(2).expect("Failed to detach kernel driver");
		let (tx, rx): (mpsc::SyncSender<USBControlTransfer>, mpsc::Receiver<USBControlTransfer>) = mpsc::sync_channel(0);

		let rx_thread = Self::start_rx_thread(libusb_scope, rx, handle);
		self.io_queue = Some((tx, rx_thread));
		self
	}

	pub fn packet_from_raw(haptic_channel: u8, hi_period: u16, lo_period: u16, cycle_count: u16) -> USBControlTransfer {
		let packet = SCFeedbackPacket {
			haptic_channel,
			hi_period,
			lo_period,
			cycle_count,
			priority: NOTE_PRIORITY,
		};

		let timeout = Duration::from_secs(1);
		let request_type = ::libusb::request_type(libusb::Direction::Out, libusb::RequestType::Class, libusb::Recipient::Interface);

		USBControlTransfer{
			request_type,
			request: ::libusb_sys::LIBUSB_REQUEST_SET_CONFIGURATION,
			value: 0x0300, // Still can't remember what this one was for
			index: 2, // Interface number, IIRC
			buf: packet.serialize(),
			timeout
		}
	}
}

struct SCFeedbackPacket {
	haptic_channel: u8,
	hi_period: u16,
	lo_period: u16,
	cycle_count: u16,
	priority: u8,
}

impl SCFeedbackPacket {
	pub fn serialize(&self) -> Vec<u8> {
		let mut buf = vec![];
		buf.write_u8(0x8f); // Feedback data
		buf.write_u8(0x08); // Length : 8 bytes
		buf.write_u8(self.haptic_channel);
		buf.write_u16::<LittleEndian>(self.hi_period);
		buf.write_u16::<LittleEndian>(self.lo_period);
		buf.write_u16::<LittleEndian>(self.cycle_count);
		buf.write_u8(self.priority);

		buf.resize(64, 0);
		buf
	} 
}