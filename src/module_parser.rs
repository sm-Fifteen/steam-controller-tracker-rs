use super::AppConfig;
use openmpt::module::iteration::{Pattern, Row, Cell};
use openmpt::mod_command::*;

use music::{Instrument, ChannelInstruction, NO_INSTRUMENT};
use routines::Routine;
use playback_timer::Timer;

struct RowParsingConfig<'a> {
	num_channels: i32,
	channel_filter: &'a [i32],
	instruments: &'a [Instrument],
}

pub fn parse_module(config: &mut AppConfig, timer: &mut Timer) {
	let mut next_pattern_order = 0;
	let mut next_row_num = 0;

	let row_config = RowParsingConfig {
		instruments: &config.instruments,
		num_channels: config.num_channels,
		channel_filter: &config.channel_filter,
	};

	let mut routines:Vec<(Routine, Instrument)> = vec![(Routine::StopNote, NO_INSTRUMENT); config.num_channels as usize];
	let mut chan_state:Vec<ChannelInstruction> = vec![ChannelInstruction::Stop; config.num_channels as usize];

	while let Some(mut pattern) = config.module.get_pattern_by_order(next_pattern_order) {
		while let Some(mut row) = pattern.get_row_by_number(next_row_num) {
			parse_row(&mut row, &row_config, &mut routines, &mut chan_state);
			timer.play_routines(&routines, &mut chan_state);

			next_row_num += 1;
		}
		next_pattern_order += 1;
		next_row_num = 0;
	}
}

fn parse_row(row: &mut Row, config: &RowParsingConfig, routines: &mut Vec<(Routine, Instrument)>, chan_state: &mut Vec<ChannelInstruction>) {
	// TODO : Check global effects (set speed/tempo, break pattern, goto order)
	// FIXME : Not ideal to play same channel multiple times
	for (idx, channel_num) in config.channel_filter.iter().enumerate() {
		let mut cell = row.get_cell_by_channel(*channel_num).expect(&format!("No cell at channel {}", *channel_num));
		let mut state = &mut chan_state[idx..idx+1];
		let routine:&mut [(Routine, Instrument)] = &mut routines[idx..idx+1];

		if let Ok(cell_data) = cell.get_data() {
			cell_to_routine(&cell_data, config.instruments, routine, state);
		}
	}
}

fn cell_to_routine<'a>(cell_data: &ModCommand, instruments: &[Instrument], out_routines: &mut [(Routine, Instrument)], state: &mut [ChannelInstruction]) {
	let instr = match cell_data.instr {
		0 => NO_INSTRUMENT,
		// Instrument indexing starts at 1
		_ => instruments[cell_data.instr as usize -1],
	};
	
	// Check for note stops early, no need to worry about them later
	let mut new_routine = match cell_data.note {
		Note::Special(SpecialNote::KeyOff)  |
		Note::Special(SpecialNote::NoteCut) |
		Note::Special(SpecialNote::Fade)   => Some(Routine::StopNote),
		_ => None,
	};

	if new_routine.is_none() {
		let new_note = match cell_data.note {
			Note::Note(semitone_idx) => Some(::music::Note::new(semitone_idx as i16)),
			_ => None,
		};

		new_routine = match cell_data.command {
			// TODO: Fill with effects
			EffectCommand::Arpeggio(x, y) => {
				if let Some(note) = new_note {
					state[0] = ChannelInstruction::Long(note);
				}
				
				Some(Routine::Arpeggio{ x, y })
			},
			_ => {
				if let Some(note) = new_note {
					state[0] = ChannelInstruction::Long(note);
					Some(Routine::FlatNote)
				} else {
					None
				}
			},
		};
	}

	if let Some(new_routine) = new_routine {
		out_routines[0] = (new_routine, instr);
	}
}