use std::thread;
use std::time::Duration;
use routines::Routine;
use music::{Instrument, ChannelInstruction};
use device_io::DeviceManager;
use crossbeam::scope;

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
			device_manager: device_manager,
			beats_per_minute: 0,
			lines_par_beat: 0f32,
			ticks_per_line: 0,
		};

		returned_timer.set_tempo(initial_tempo);
		returned_timer.set_speed(initial_speed);
		returned_timer
	}

	pub fn play_routines(&mut self, routines: &Vec<(Routine, Instrument)>, chan_state: &mut Vec<ChannelInstruction>) {
		let row_duration = Duration::from_millis((60000f32/(self.lines_par_beat * self.beats_per_minute as f32)) as u64);
		let mut state_chunks = chan_state.chunks_mut(1);

		let row_timer_thread = thread::spawn(move || {
			thread::sleep(row_duration);
		});

		scope(|scope| {
			for (channel_idx, &(routine, instrument)) in routines.iter().enumerate() {
				let device_manager = &self.device_manager;
				let mut state = state_chunks.next().expect("State and device lists do not match");
				
				let speed = match routine.get_speed() {
					Some(speed) => speed,
					None => self.ticks_per_line as u32,
				};

				scope.spawn(move || {
					let tick_duration = row_duration/speed;
					
					for tick in 0..speed {
						let tick_timer_thread = thread::spawn(move || {
							thread::sleep(tick_duration);
						});

						let tick_result = routine.tick_value(tick as i32, &mut state[0]);
						let channel_idx = channel_idx as u32;
						
						// TODO: Make usb tx acquisition happen somewhere around here, so we can react properly to errors.
						// Also, remove device_manager.play_*, since those won't be needed anymore

						if let Some(instruction) = tick_result {
							match instruction {
								ChannelInstruction::Stop => device_manager.play_raw(channel_idx, 0, 0, 0),
								ChannelInstruction::Long(note) => device_manager.play_note(channel_idx, &note, &instrument, None),
								ChannelInstruction::Short(note) => device_manager.play_note(channel_idx, &note, &instrument, Some(tick_duration)),
							};
						} else if let &ChannelInstruction::Short(note) = &state[0] {
							// If a short note is not renewed, it's replaced by a long note of the channel state
							state[0] = ChannelInstruction::Long(note);
							device_manager.play_note(channel_idx, &note, &instrument, None);
						}

						tick_timer_thread.join();
					}
				});
			}
		});

		row_timer_thread.join();
	}

	pub fn set_speed(&mut self, speed: i32) {
		self.lines_par_beat = 24f32/(speed as f32);
		self.ticks_per_line = speed;
	}

	pub fn set_tempo(&mut self, tempo: i32) {
		self.beats_per_minute = tempo;
	}
}