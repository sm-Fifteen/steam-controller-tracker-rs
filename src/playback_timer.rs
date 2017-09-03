use std::thread;
use std::time::Duration;
use routines::{Routine, RoutineResult};
use music::Instrument;
use device_io::DeviceManager;

/// Lock-step timer responsible for playing routines at the right tickrate
/// and controlling device IO
pub struct Timer<'a> {
	device_manager: DeviceManager<'a>,
	beats_per_minute: i32,
	lines_par_beat: f32,
	ticks_per_line: i32,
}

impl<'a> Timer<'a> {
	pub fn new(device_manager: DeviceManager, initial_tempo: i32, initial_speed: i32) -> Timer {
		let mut returned_timer = Timer {
			device_manager,
			beats_per_minute: 0,
			lines_par_beat: 0f32,
			ticks_per_line: 0,
		};

		returned_timer.set_tempo(initial_tempo);
		returned_timer.set_speed(initial_speed);
		returned_timer
	}

	pub fn play_routines(&mut self, routines: &Vec<(Routine, Instrument)>) {
		let row_duration = (60000f32/(self.lines_par_beat * self.beats_per_minute as f32)) as u64;

		// Only play first tick for now, forever
		for (channel_idx, &(routine, instrument)) in routines.iter().enumerate() {
			let tick_result = routine.tick_value(0);
			let channel_idx = channel_idx as u32;
			
			// FIXME : Use a match, but Nothing must return a result
			if let RoutineResult::Play(note) = tick_result {
				self.device_manager.play_note(channel_idx, &note, &instrument, None);
			} else if let RoutineResult::Stop = tick_result {
				self.device_manager.play_raw(channel_idx, 0, 0, 0);
			}
		}

		// Let's be optimistic and assume for now that IO and calculations take 0ms
		thread::sleep(Duration::from_millis(row_duration));
	}

	pub fn set_speed(&mut self, speed: i32) {
		self.lines_par_beat = 24f32/(speed as f32);
		self.ticks_per_line = speed;
	}

	pub fn set_tempo(&mut self, tempo: i32) {
		self.beats_per_minute = tempo;
	}
}