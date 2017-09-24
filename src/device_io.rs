/// Touch feedback is 0, priority 1 means notes don't
/// get interrupted when the user touches the controller
const NOTE_PRIORITY:u8 = 1;
const REPEAT_FOREVER:u16 = 0x7FFF;

use libusb::{Direction,RequestType,Recipient, Device, DeviceHandle};
use libusb::Context;
use libusb::Error;
use byteorder::{LittleEndian, WriteBytesExt};
use std::time::Duration;
use std::sync::Mutex;
use music::{Note, Instrument};

pub struct DeviceManager<'a> {
	libusb_context: &'a Context,
	devices: Vec<Device<'a>>,
	handles: Vec<DeviceHandle<'a>>,
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

pub struct USBControlTransfer {
	request_type: u8,
	request: u8,
	value: u16,
	index: u16,
	buf: Vec<u8>,
	timeout: Duration,
}

impl<'a> DeviceManager<'a> {
	pub fn new(libusb_context: &'a mut Context) -> Mutex<DeviceManager<'a>> {
		let mut iter_list = libusb_context.devices().unwrap();
		let mut iter = iter_list.iter();
		let mut devices = Vec::<Device<'a>>::new();
		let mut handles = Vec::<DeviceHandle<'a>>::new();

		let matches = iter.filter(|device| {
			match device.device_descriptor() {
				Err(_) => false,
				Ok(desc) => {
					desc.vendor_id() == 0x28de &&
					desc.product_id() == 0x1102
				}
			}
		});

		for device in matches {
			if let Ok(mut handle) = device.open() {
				handle.detach_kernel_driver(2);
				devices.push(device);
				handles.push(handle);
			}
			
		}

		Mutex::new(DeviceManager {
			libusb_context,
			devices,
			handles,
		})
	}

	fn get_device_channel(&mut self, channel_num: u32) -> Option<(&mut DeviceHandle<'a>, u8)> {
		if let Some(mut handle) = self.handles.get_mut(channel_num as usize >> 1) {
			Some((handle, (channel_num % 2) as u8))
		} else {
			None
		}
	}

	pub fn play_note(&mut self, channel: u32, note: &Note, instr: &Instrument, max_duration: Option<Duration>) -> Result<usize, Error> {		
		if let Some((device, haptic_channel)) = self.get_device_channel(channel) {
			Self::send_control(device, USBControlTransfer::from_note(haptic_channel, note, instr, max_duration))
		} else {
			Err(Error::NoDevice)
		}		
	}

	pub fn play_raw(&mut self, channel: u32, hi_period: u16, lo_period: u16, cycle_count: u16) -> Result<usize, Error> {
		if let Some((device, haptic_channel)) = self.get_device_channel(channel) {
			Self::send_control(device, USBControlTransfer::from_raw(haptic_channel, hi_period, lo_period, cycle_count))
		} else {
			Err(Error::NoDevice)
		}	
	}
	
	fn send_control(device: &mut DeviceHandle, control: USBControlTransfer) -> Result<usize, ::libusb::Error> {
		device.write_control(control.request_type, control.request, control.value, control.index, control.buf.as_slice(), control.timeout)
	}
}

impl USBControlTransfer {
	pub fn from_note(haptic_channel: u8, note: &Note, instr: &Instrument, max_duration: Option<Duration>) -> USBControlTransfer {
		if let Some(duration) = max_duration {
			let (hi_period, lo_period, cycle_count) = instr.get_periods_for_note_with_duration(note, duration);
			USBControlTransfer::from_raw(haptic_channel, hi_period, lo_period, cycle_count)
		} else {
			let (hi_period, lo_period) = instr.get_periods_for_note(note);
			USBControlTransfer::from_raw(haptic_channel, hi_period, lo_period, REPEAT_FOREVER)
		}
	}

	pub fn from_raw(haptic_channel: u8, hi_period: u16, lo_period: u16, cycle_count: u16) -> USBControlTransfer {
		let packet = SCFeedbackPacket {
			haptic_channel,
			hi_period,
			lo_period,
			cycle_count,
			priority: NOTE_PRIORITY,
		};

		let timeout = Duration::from_secs(1);
		let request_type = ::libusb::request_type(Direction::Out, RequestType::Class, Recipient::Interface);

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