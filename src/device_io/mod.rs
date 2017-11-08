mod device;
mod steam_controller;

use libusb::Context;
use self::device::{USBDeviceWrapper, MusicDevice};
use self::steam_controller::SteamController;
use std::time::Duration;
use music::{Note, Instrument};

// TODO: Replace with MusicDevices (boxed pointers?)
type DeviceType<'a> = SteamController<'a>;

pub struct DeviceManager<'a> {
	libusb_context: &'a Context,
	devices: Vec<DeviceType<'a>>,
}

impl<'a> DeviceManager<'a> {
	pub fn new(libusb_context: &'a mut Context) -> DeviceManager<'a> {
		let mut iter_list = libusb_context.devices().unwrap();
		let mut iter = iter_list.iter();

		let matches = iter.filter_map(|device| {
			// TODO : Try other devices here
			match SteamController::device_matcher(&device) {
				Ok(dev) => dev,
				Err(err) => {
					panic!("USB error while matching devices : \"{}\"", err);
				}
			}
		});

		DeviceManager {
			libusb_context,
			devices: matches.collect(),
		}
	}

	fn get_device_channel(&'a self, channel_num: usize) -> Option<(&DeviceType<'a>, usize)> {
		let mut channel_iter = ChannelIterator::new(&self.devices);
		channel_iter.nth(channel_num)
	}

	pub fn play_note(&self, channel: u32, note: &Note, instr: &Instrument, max_duration: Option<Duration>) -> Result<(), ()> {		
		if let Some((device, haptic_channel)) = self.get_device_channel(channel as usize) {
			let packet = DeviceType::packet_from_note(haptic_channel, note, instr, max_duration).expect("Note conversion should never fail");

			match device.send_packet(packet) {
				Ok(_) => Ok(()),
				//Err(err) => panic!("{}", err), // FIXME : Find a clean way to return the various error types that can be returned
				Err(_) => Err(()), // Device probably disconnected, should be de-listed
			}
		} else {
			Err(()) // No such device
		}
	}

	pub fn play_raw(&self, channel: u32, hi_period: u16, lo_period: u16, cycle_count: u16) -> Result<(), ()> {
		if let Some((device, haptic_channel)) = self.get_device_channel(channel as usize) {
			let packet = DeviceType::packet_from_raw(haptic_channel as u8, hi_period, lo_period, cycle_count);

			match device.send_packet(packet) {
				Ok(_) => Ok(()),
				//Err(err) => panic!("{}", err), // FIXME
				Err(_) => Err(()), // Device probably disconnected, should be de-listed
			}
		} else {
			Err(()) // No such device
		}
	}
}

struct ChannelIterator<'a> {
	devices: &'a Vec<DeviceType<'a>>,
	channel_id: usize,
	device_id: usize,
}

impl<'a> ChannelIterator<'a> {
	fn new(devices: &'a Vec<DeviceType<'a>>) -> Self {
		Self {
			devices,
			channel_id: 0,
			device_id: 0,
		}
	}
}

impl<'a> Iterator for ChannelIterator<'a> {
	type Item = (&'a DeviceType<'a>, usize);

	fn next(&mut self) -> Option<Self::Item>{
		let device = self.devices.get(self.device_id)?;
		if self.channel_id < device.channel_count() {
			let return_val = (device, self.channel_id);
			self.channel_id += 1;
			Some(return_val)
		} else {
			self.device_id += 1;
			self.channel_id = 0;
			self.next()
		}
	}
}