use std::time::Duration;

const PERIOD_1HZ:u32 = 1_000_000; // 1 million microseconds

// Could be a pulse, could be a frequency with a duty-cycle
pub trait Instrument {
	fn get_periods_for_note(&self, note: &Note) -> (u16, u16);
	fn get_cycles_for_duration(&self, note: &Note, duration: Duration) -> u16;
}

pub struct Note {
	semitone_idx: u8,
	// sub-semitone stuff
}

pub struct PulseWave {
	duty_on : u32,
	duty_off : u32,
}

impl Note {
	pub fn new(semitone_idx: u8) -> Note {
		Note { semitone_idx }
	}

	pub fn get_frequency(&self) -> f32 {
		1960f32
	}
}

impl PulseWave {
	pub fn new(duty_on: u32, duty_off: u32) -> PulseWave {
		PulseWave{ duty_on, duty_off }
	}
}

impl Instrument for PulseWave {
	fn get_periods_for_note(&self, note: &Note) -> (u16, u16) {
		let on_time = (self.duty_on * PERIOD_1HZ) as f32;
		let off_time = (self.duty_off * PERIOD_1HZ) as f32;
		let scaled_freq = note.get_frequency() * (self.duty_on + self.duty_off) as f32;

		let hi_period = on_time/scaled_freq;
		let lo_period = off_time/scaled_freq;

		(hi_period as u16, lo_period as u16)
	}

	fn get_cycles_for_duration(&self, note: &Note, duration: Duration) -> u16 {
		let duration_secs = duration.as_secs() as f32 + duration.subsec_nanos() as f32 * 1e-9;
		(duration_secs*note.get_frequency()) as u16
	}
}