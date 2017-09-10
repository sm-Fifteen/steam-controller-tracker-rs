use std::thread;
use std::time::Duration;
use std::sync::Mutex;
use routines::Routine;
use music::{Instrument, ChannelInstruction};
use device_io::DeviceManager;

/// Lock-step timer responsible for playing routines at the right tickrate
/// and controlling device IO
pub struct Timer<'a> {
	device_manager: Mutex<DeviceManager<'a>>,
	beats_per_minute: i32,
	lines_par_beat: f32,
	ticks_per_line: i32,
}

impl<'a> Timer<'a> {
	pub fn new(device_manager: Mutex<DeviceManager>, initial_tempo: i32, initial_speed: i32) -> Timer {
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

	pub fn play_routines(&mut self, routines: &Vec<(Routine, Instrument)>, chan_state: &mut Vec<ChannelInstruction>) {
		let row_duration = Duration::from_millis((60000f32/(self.lines_par_beat * self.beats_per_minute as f32)) as u64);
		let device_manager = self.device_manager.get_mut().unwrap();

		let timer_thread = thread::spawn(move || {
			thread::sleep(row_duration);
		});

		// Only play first tick until I figure out threaded I/O
		for (channel_idx, &(routine, instrument)) in routines.iter().enumerate() {
			let mut state = &mut chan_state[channel_idx..channel_idx+1];
			let tick_result = routine.tick_value(0, &mut state[0]);
			let channel_idx = channel_idx as u32;
			
			let usb_error = if let Some(instruction) = tick_result {
				match instruction {
					ChannelInstruction::Stop => device_manager.play_raw(channel_idx, 0, 0, 0).err(),
					ChannelInstruction::Long(note) => device_manager.play_note(channel_idx, &note, &instrument, None).err(),
					ChannelInstruction::Short(note) => device_manager.play_note(channel_idx, &note, &instrument, Some(row_duration)).err(),
				}
			} else if let &ChannelInstruction::Short(note) = &state[0] {
				// If a short note is not renewed, it's replaced by a long note of the channel state
				state[0] = ChannelInstruction::Long(note);
				device_manager.play_note(channel_idx, &note, &instrument, None).err()
			} else {
				None
			};

			if let Some(error) = usb_error {
				println!("Error from libusb : {}", error.strerror());
			}
		}

		timer_thread.join();
	}

	pub fn set_speed(&mut self, speed: i32) {
		self.lines_par_beat = 24f32/(speed as f32);
		self.ticks_per_line = speed;
	}

	pub fn set_tempo(&mut self, tempo: i32) {
		self.beats_per_minute = tempo;
	}
}