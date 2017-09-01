extern crate libusb;
extern crate libusb_sys;
extern crate clap;
extern crate openmpt;
extern crate byteorder;

use std::fs::File;
use std::error::Error;

use openmpt::module::{Module, Logger};
use clap::{App, Arg, ArgMatches};
use device_io::DeviceManager;

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
	let mut libusb_context = libusb::Context::new().unwrap();
	let mut dm = DeviceManager::new(&mut libusb_context);

	let mut note = music::Note::new(59);
	let mut instr = music::PulseWave::new(1, 1);

	println!("Returned : {:?}", dm.play_note(1, &note, &instr, ::std::time::Duration::from_millis(200)));
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