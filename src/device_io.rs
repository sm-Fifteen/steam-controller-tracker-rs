/// Touch feedback is 0, priority 1 means notes don't
/// get interrupted when the user touches the controller
const NOTE_PRIORITY:u8 = 1;

use libusb::{Direction,RequestType,Recipient, DeviceHandle};
use libusb::Context;
use libusb::Error;
use byteorder::{LittleEndian, WriteBytesExt};

pub struct DeviceManager<'a> {
	libusb_context: &'a Context,
	// TODO : Use vector instead
	device: DeviceHandle<'a>,
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

impl<'a> DeviceManager<'a> {
	pub fn new(libusb_context: &'a mut Context) -> DeviceManager<'a> {
		let mut device = libusb_context.open_device_with_vid_pid(0x28de, 0x1102).expect("No matching device");
		device.detach_kernel_driver(2);

		DeviceManager {
			libusb_context,
			device,
		}
	}

	fn get_device_channel(&mut self, channel_num: u32) -> Option<(&mut DeviceHandle<'a>, u8)> {
		Some((&mut self.device, (channel_num % 2) as u8))
	}

	pub fn play_raw(&mut self, channel: u32, hi_period: u16, lo_period: u16, cycle_count: u16) -> Result<usize, Error> {
		if let Some((device, haptic_channel)) = self.get_device_channel(channel) {
			println!("Device detected");
			
			let packet = SCFeedbackPacket {
				haptic_channel,
				hi_period,
				lo_period,
				cycle_count,
				priority: NOTE_PRIORITY,
			};

			let timeout = ::std::time::Duration::from_secs(1);
			let req_type = ::libusb::request_type(Direction::Out, RequestType::Class, Recipient::Interface);
			
			device.write_control(
				req_type,
				::libusb_sys::LIBUSB_REQUEST_SET_CONFIGURATION,
				0x0300, // Still can't remember what this one was for
				2, // Interface number, IIRC
				packet.serialize().as_slice(),
				timeout
			)
		} else {
			Err(Error::NoDevice)
		}
	}
}