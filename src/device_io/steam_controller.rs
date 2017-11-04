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

pub struct SteamController<'context> {
	device: libusb::Device<'context>,
	//tx: mpsc::SyncSender<USBControlTransfer>,
	//rx_thread: crossbeam::ScopedJoinHandle<libusb::Error>,
}

impl<'context> USBDeviceWrapper<'context> for SteamController<'context> {
	fn device_matcher(device: libusb::Device<'context>) -> Option<Self> {
		let desc = device.device_descriptor().ok()?;

		if desc.vendor_id() == 0x28de && desc.product_id() == 0x1102 {
			Some(SteamController{ device })
		} else {
			None
		}
	}
}

impl<'context> MusicDevice<USBControlTransfer> for SteamController<'context> {
	fn get_io_queue(&self) -> mpsc::SyncSender<USBControlTransfer> {
		unimplemented!()
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

	fn init_io_queue(&self) {
		
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