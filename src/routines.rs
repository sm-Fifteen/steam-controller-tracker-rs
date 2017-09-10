use music::{Note, ChannelInstruction};
use music::ChannelInstruction::*;

#[derive(Copy,Clone)]
pub enum Routine {
	StopNote,
	FlatNote,
}

impl Routine {
	// Returns some if a sound is to be played, may mutate the state either way.
	pub fn tick_value(self, tick: i32, state: &mut ChannelInstruction) -> Option<ChannelInstruction> {
		use self::Routine::*;

		match self {
			StopNote => if tick == 0 { Some(Stop) } else { None },
			FlatNote => if tick == 0 { Some(*state) } else { None },
		}
	}
}
