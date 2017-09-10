use music::{Note, ChannelInstruction};
use music::ChannelInstruction::*;

#[derive(Copy,Clone)]
pub enum Routine {
	StopNote,
	FlatNote,
	Arpeggio {x: u8, y: u8},
}

impl Routine {
	// Returns some if a sound is to be played, may mutate the state either way.
	pub fn tick_value(self, tick: i32, state: &mut ChannelInstruction) -> Option<ChannelInstruction> {
		use self::Routine::*;

		match self {
			StopNote => if tick == 0 { Some(Stop) } else { None },
			FlatNote => if tick == 0 { Some(*state) } else { None },
			Arpeggio{ x, y } => match *state {
				// DOES NOT mutate the state
				Short(note) | Long(note) => {
					let new_note = Short(match tick%2 {
						0 => note,
						1 => note + x,
						2 => note + y,
						_ => unreachable!(),
					});

					Some(new_note)
				},
				_ => None,
			}
		}
	}
}
