use std::collections::BTreeSet;

use yazi_macro::render;
use yazi_shared::event::Cmd;

use crate::tab::{Mode, Tab};

struct Opt {
	unset: bool,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { unset: c.bool("unset") } }
}

impl Tab {
	#[yazi_codegen::command]
	pub fn visual_mode(&mut self, opt: Opt) {
		let idx = self.current.cursor;
		if opt.unset {
			self.mode = Mode::Unset(idx, BTreeSet::from([idx]));
		} else {
			let is_any_file_selected = match self.mode {
				Mode::Select(_, ref selected_indices) => !selected_indices.is_empty(),
				_ => false,
			};

			if is_any_file_selected {
				self.append_selected( true );
				self.mode = Mode::Normal;
			} else {
				self.mode = Mode::Select(idx, BTreeSet::from([idx]));
			}
		};
		render!();
	}
}
