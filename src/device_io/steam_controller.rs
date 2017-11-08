use super::device::{USBDeviceWrapper, MusicDevice, USBControlTransfer, InstrumentError};
use libusb;
use std::time::Duration;
use byteorder::{LittleEndian, WriteBytesExt};
use music::{Note, Instrument};

/// Touch feedback is 0, priority 1 means notes don't
/// get interrupted when the user touches the controller
const NOTE_PRIORITY:u8 = 1;
const REPEAT_FOREVER:u16 = 0x7FFF;

pub struct SteamController<'context> {
	// Handle is the "active" device, the Device reference isn't needed after that
	handle: libusb::DeviceHandle<'context>,
}

impl<'context> USBDeviceWrapper<'context> for SteamController<'context> {
	fn match_rules(device: &libusb::Device<'context>) -> Result<bool, libusb::Error> {
		let desc = device.device_descriptor()?;

		return Ok(desc.vendor_id() == 0x28de && desc.product_id() == 0x1102);
	}

	fn device_matcher(device: &libusb::Device<'context>) -> Result<Option<Self>, libusb::Error> {
		match Self::match_rules(device) {
			Ok(true) => {
				let mut handle = device.open()?;
				if handle.kernel_driver_active(2)? { handle.detach_kernel_driver(2)?; }
				
				Ok(Some(SteamController{
					handle,
				}))
			},
			Ok(false) => Ok(None),
			Err(err) => Err(err),
		}
	}
}

impl<'context> MusicDevice for SteamController<'context> {
	type PacketType = USBControlTransfer;
	type IOErrorType = libusb::Error;
	
	fn send_packet(&self, transfer: USBControlTransfer) -> Result<(), libusb::Error> {
		<Self as USBDeviceWrapper<'context>>::send_control(&self.handle, transfer).and(Ok(()))
	}

	fn init(&mut self) -> Result<(), libusb::Error> {
		Ok(()) // No init procedure here
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

		USBControlTransfer::new(
			request_type,
			::libusb_sys::LIBUSB_REQUEST_SET_CONFIGURATION, // request
			0x0300, // value: Still can't remember what this one was for
			2, // index: Interface number, IIRC
			packet.serialize(),
			timeout
		)
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
		buf.write_u8(0x8f).unwrap(); // Feedback data
		buf.write_u8(0x08).unwrap(); // Length : 8 bytes
		buf.write_u8(self.haptic_channel).unwrap();
		buf.write_u16::<LittleEndian>(self.hi_period).unwrap();
		buf.write_u16::<LittleEndian>(self.lo_period).unwrap();
		buf.write_u16::<LittleEndian>(self.cycle_count).unwrap();
		buf.write_u8(self.priority).unwrap();

		buf.resize(64, 0);
		buf
	} 
}