use music::Note;

#[derive(Copy,Clone)]
pub enum Routine {
	StopNote,
	FlatNote{note: Note},
}

pub enum RoutineResult {
	Play(Note),
	Stop,
	Nothing,
}

impl Routine {
	pub fn tick_value(self, row_tick: i32, speed: i32) -> RoutineResult {
		use self::RoutineResult::*;
		use self::Routine::*;

		match self {
			StopNote => if row_tick == 0 { Stop } else { Nothing },
			FlatNote {note} => if row_tick == 0 { Play(note) } else { Nothing },
		}
	}
}
