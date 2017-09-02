use openmpt::module::Module;
use openmpt::module::iteration::{Pattern, Row, Cell};
use openmpt::mod_command::*;

use routines::{Routine, StopNote};

struct RowParsingConfig {
	num_channels: i32,
	channel_filter: Vec<i32>,
}

pub fn parse_module(module: &mut Module) {
	let mut next_pattern_order = 0;
	let mut next_row_num = 0;

	let num_channels = module.get_num_channels();

	// Both num_channels and the filter are needed
	// because global effects must still be parsed
	let row_config = RowParsingConfig {
		num_channels,
		channel_filter: (0..num_channels).collect(),
	};

	let mut routines:Vec<Box<Routine>> = Vec::new();
	for _ in &row_config.channel_filter {
		routines.push(Box::new(StopNote::new()));
	}

	while let Some(mut pattern) = module.get_pattern_by_order(next_pattern_order) {
		while let Some(mut row) = pattern.get_row_by_number(next_row_num) {
			parse_row(&mut row, &row_config, &mut routines);

			next_row_num += 1;
		}
		next_pattern_order += 1;
		next_row_num = 0;
	}
}

fn parse_row(row: &mut Row, config: &RowParsingConfig, routines: &mut Vec<Box<Routine>>) {
	// TODO : Check global effects (set speed/tempo, break pattern, goto order)
	// FIXME : Not ideal to play same channel multiple times
	for (idx, channel_num) in config.channel_filter.iter().enumerate() {
		let mut cell = row.get_cell_by_channel(*channel_num).expect(&format!("Not cell at channel {}", *channel_num));
		let routine:&mut [Box<Routine>] = &mut routines[idx..idx+1];

		if let Ok(cell_data) = cell.get_data() {
			cell_to_routine(&cell_data, routine);
		}
	}
}

fn cell_to_routine(cell_data: &ModCommand, out_routines: &mut [Box<Routine>]) {
	let new_routine:Option<Box<Routine>> = match cell_data.note {
		Note::Note(semitone_idx) => {
			let note = ::music::Note::new(semitone_idx as i16);
			Some(Box::new(::routines::FlatNote::new(note)))
		},
		Note::Special(SpecialNote::KeyOff) | Note::Special(SpecialNote::NoteCut) | Note::Special(SpecialNote::Fade) => Some(Box::new(StopNote::new())),
		_ => None,
	};

	if let Some(routine_box) = new_routine {
		out_routines[0] = routine_box;
	}
}