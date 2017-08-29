extern crate clap;
extern crate openmpt;

use std::fs::File;
use std::error::Error;

use openmpt::module::{Module, Logger};
use clap::{App, Arg, ArgMatches};

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
	println!("{}", config.module.get_num_patterns())
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