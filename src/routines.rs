use music::{Note, ChannelInstruction};
use music::ChannelInstruction::*;

#[derive(Copy,Clone)]
pub enum Routine {
	StopNote,
	FlatNote{note: Note},
}

impl Routine {
	pub fn tick_value(self, tick: i32) -> Option<ChannelInstruction> {
		use self::Routine::*;

		match self {
			StopNote => if tick == 0 { Some(Stop) } else { None },
			FlatNote {note} => if tick == 0 { Some(Long(note)) } else { None },
		}
	}
}
