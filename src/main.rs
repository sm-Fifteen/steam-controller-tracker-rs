extern crate libusb;
extern crate libusb_sys;
extern crate clap;
extern crate openmpt;
extern crate byteorder;

use std::fs::File;
use std::error::Error;

use openmpt::module::{Module, Logger};
use clap::{App, Arg, ArgMatches};

mod device_io;
mod music;

fn main() {
	let matches = App::new("Steam Controller Tracker")
			.arg(Arg::with_name("file")
					.help("the module file to be played")
					.index(1)
					.required(true)
			)
			.get_matches();
	
	let config = AppConfig::from_opts(matches);
	
	if let Err(msg) = config {
        println!("Problem parsing arguments: {}", msg);
        ::std::process::exit(1);
    };

	run(&mut config.unwrap())
}

fn run(config: &mut AppConfig) {
	unimplemented!();
}

struct AppConfig {
	pub module : Module,
}

impl AppConfig {
	fn from_opts(matches : ArgMatches) -> Result<AppConfig, String> {
		let mut file = match File::open(matches.value_of("file").unwrap()) {
			Err(e) => return Err(e.description().to_owned()),
			Ok(f) => f
		};

		let module = match Module::create(&mut file, Logger::None, &[]) {
			Err(_) => return Err(String::from("Failed to open file as tracker module")),
			Ok(m) => m,
		};
		
		Ok(AppConfig {
			module : module,
		})
	}
}

#[cfg(test)]
mod tests {
	use libusb;
	use super::{music, device_io};
	use device_io::DeviceManager;

	#[test]
	fn test_beep() {
		let mut libusb_context = libusb::Context::new().unwrap();
		let mut dm = DeviceManager::new(&mut libusb_context);

		let mut note = music::Note::new(59);
		let mut instr = music::PulseWave::new(1, 1);

		let ret_value = dm.play_note(1, &note, &instr, ::std::time::Duration::from_millis(200));

		ret_value.expect("Failed to send to device");
	}
}