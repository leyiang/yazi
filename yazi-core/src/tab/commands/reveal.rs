use yazi_proxy::ManagerProxy;
use yazi_shared::{event::{Cmd, Data}, fs::{File, FilesOp, Url, expand_path}};

use crate::tab::Tab;

struct Opt {
	target: Url,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		let mut target = c.take_first().and_then(Data::into_url).unwrap_or_default();
		if target.is_regular() {
			target = Url::from(expand_path(&target));
		}

		Self { target }
	}
}
impl From<Url> for Opt {
	fn from(target: Url) -> Self { Self { target } }
}

impl Tab {
	#[yazi_codegen::command]
	pub fn reveal(&mut self, opt: Opt) {
		let Some(parent) = opt.target.parent_url() else {
			return;
		};

		self.cd(parent.clone());
		FilesOp::Creating(parent, vec![File::from_dummy(opt.target.clone(), None)]).emit();
		ManagerProxy::hover(Some(opt.target), self.id);
	}
}
