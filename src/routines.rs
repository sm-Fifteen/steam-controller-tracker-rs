use music::Note;

pub enum RoutineResult {
	Play(Note),
	Stop,
	Nothing,
}

pub trait Routine {
	/// Returns the the next note to play
	/// and move state forwards by one of that routine's ticks.
	fn tick_update(&mut self) -> RoutineResult;
	
	// Keep the routine going for another row?
	// fn continue_effect(&mut self);
}

pub struct StopNote {
	tick_counter : u16,
}

pub struct FlatNote {
	tick_counter : u16,
	note: Note,
}

impl Routine for StopNote {
	fn tick_update(&mut self) -> RoutineResult {
		if self.tick_counter == 0 {
			RoutineResult::Stop
		} else {
			RoutineResult::Nothing
		}
	}
}

impl StopNote {
	pub fn new() -> StopNote {
		StopNote {
			tick_counter: 0,
		}
	}
}

impl Routine for FlatNote {
	fn tick_update(&mut self) -> RoutineResult {
		if self.tick_counter == 0 {
			RoutineResult::Play(self.note)
		} else {
			RoutineResult::Nothing
		}
	}
}

impl FlatNote {
	pub fn new(note: Note) -> FlatNote {
		FlatNote {
			tick_counter: 0,
			note,
		}
	}
}