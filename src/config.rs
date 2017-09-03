use std::fs::File;
use std::error::Error;
use openmpt::module::{Module, Logger};
use clap::ArgMatches;
use super::music::Instrument;


pub struct AppConfig {
	pub module : Module,
	pub instruments: Vec<Instrument>,
	pub num_channels: i32,
	pub channel_filter: Vec<i32>,
}

impl AppConfig {
	pub fn from_opts(matches : ArgMatches) -> Result<AppConfig, String> {
		let mut file = match File::open(matches.value_of("file").unwrap()) {
			Err(e) => return Err(e.description().to_owned()),
			Ok(f) => f
		};

		let mut module = match Module::create(&mut file, Logger::None, &[]) {
			Err(_) => return Err(String::from("Failed to open file as tracker module")),
			Ok(m) => m,
		};
		
		let instruments = vec!(Instrument::PulseWave(1,1); module.get_num_instruments() as usize);

		let num_channels = module.get_num_channels();

		Ok(AppConfig {
			module,
			instruments,
			num_channels,
			channel_filter: (0..num_channels).collect(),
		})
	}
}