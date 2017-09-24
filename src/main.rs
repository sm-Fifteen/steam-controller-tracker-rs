extern crate libusb;
extern crate libusb_sys;
extern crate clap;
extern crate openmpt;
extern crate byteorder;
extern crate crossbeam;

use clap::{App, Arg};
use libusb::Context;
use config::AppConfig;

mod config;
mod device_io;
mod music;
mod module_parser;
mod routines;
mod playback_timer;

fn main() {
	let matches = App::new("Steam Controller Tracker")
			.arg(Arg::with_name("file")
					.help("the module file to be played")
					.index(1)
					.required(true)
			)
			.get_matches();
	
	let config = config::AppConfig::from_opts(matches);
	
	if let Err(msg) = config {
        println!("Problem parsing arguments: {}", msg);
        ::std::process::exit(1);
    };

	run(&mut config.unwrap())
}

fn run(config: &mut AppConfig) {
	let mut libusb_context = Context::new().unwrap();

	// *Almost* static thread lifetime, but must not outlive libusb
	::crossbeam::scope(|scope| {
		let device_manager = device_io::DeviceManager::new(&scope, &mut libusb_context);

		let mut timer = playback_timer::Timer::new(
			device_manager,
			config.module.get_current_tempo(),
			config.module.get_current_speed(),
		);

		module_parser::parse_module(config, &mut timer);
	})
}

#[cfg(test)]
mod tests {
	use libusb;
	use super::{music, device_io};
	use device_io::DeviceManager;

	#[test]
	fn test_beep() {
		let mut libusb_context = ::libusb::Context::new().unwrap();

		::crossbeam::scope(|scope| {
			let mut dm_mutex = DeviceManager::new(&scope, &mut libusb_context);
			let mut dm = dm_mutex.lock().unwrap();

			let mut note = music::Note::new(96);
			let mut instr = music::Instrument::PulseWave(1, 1);

			let ret_value = dm.play_note(1, &note, &instr, Some(::std::time::Duration::from_millis(200)));

			ret_value.expect("Failed to send to device");
		});
	}
}