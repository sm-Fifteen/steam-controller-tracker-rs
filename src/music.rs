use std::time::Duration;

const FLOAT_2:f32 = 2f32;
// static MIDDLE_C_FREQ:f32 = 440f32 * FLOAT_2.powf(-9f32/12f32);
static MIDDLE_C_FREQ:f32 = 261.625565;
// static MIDDLE_C_IDX:i16 = ::openmpt::mod_command::ModCommand::middle_c() as i16;
static MIDDLE_C_IDX:i16 = 61;

const PERIOD_1HZ:u32 = 1_000_000; // 1 million microseconds

pub const NO_INSTRUMENT:Instrument = Instrument::NoInstrument;

#[derive(Copy,Clone)]
pub enum ChannelInstruction {
	/// Plays for an indefinite amount of time,
	/// until replaced by another
	Long(Note),
	/// Plays for the duration of one tick only.
	/// Counts as Long on the last tick of a row.
	Short(Note),
	/// Immediately stop playing
	Stop,
	// Once? Drums could need that
}

#[derive(Copy,Clone)]
pub enum Instrument {
	PulseWave (u32, u32),
	NoInstrument,
}

#[derive(Copy,Clone)]
pub struct Note {
	semitone_idx: i16,
	// sub-semitone stuff
}

// TODO : Add with tuple for sub-semitone variations
impl ::std::ops::Add<u8> for Note {
    type Output = Note;

    fn add(self, semitones: u8) -> Note {
        Note { semitone_idx: self.semitone_idx + semitones as i16 }
    }
}

impl Note {
	pub fn new(semitone_idx: i16) -> Note {
		Note { semitone_idx }
	}

	pub fn get_frequency(&self) -> f32 {
		let rel_idx = (self.semitone_idx - MIDDLE_C_IDX) as f32;
		MIDDLE_C_FREQ * FLOAT_2.powf(rel_idx/12f32)
	}
}

impl Instrument {
	pub fn get_periods_for_note(&self, note: &Note) -> (u16, u16) {
		use self::Instrument::*;

		match *self {
			PulseWave(duty_on, duty_off) => {
				let freq = note.get_frequency();
				pulsewave_get_periods_for_freq(freq, duty_on, duty_off)
			},
			NoInstrument => (0, 0),
		}
	}

	pub fn get_periods_for_note_with_duration(&self, note: &Note, duration: Duration) -> (u16, u16, u16) {
		use self::Instrument::*;

		match *self {
			PulseWave(duty_on, duty_off) => {
				let freq = note.get_frequency();
				let (hi_period, lo_period) = pulsewave_get_periods_for_freq(freq, duty_on, duty_off);

				let duration_secs = duration.as_secs() as f32 + duration.subsec_nanos() as f32 * 1e-9;
				let duration_cycles = (duration_secs*freq) as u16;
				(hi_period, lo_period, duration_cycles)
			},
			NoInstrument => (0, 0, 0)
		}
	}
}

fn pulsewave_get_periods_for_freq(freq: f32, duty_on : u32, duty_off : u32) -> (u16, u16) {
	let on_time = (duty_on * PERIOD_1HZ) as f32;
	let off_time = (duty_off * PERIOD_1HZ) as f32;
	let scaled_freq = freq * (duty_on + duty_off) as f32;

	let hi_period = on_time/scaled_freq;
	let lo_period = off_time/scaled_freq;

	(hi_period as u16, lo_period as u16)
}