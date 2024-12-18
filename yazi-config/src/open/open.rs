use std::{collections::HashMap, path::Path, str::FromStr};

use anyhow::Context;
use indexmap::IndexSet;
use serde::{Deserialize, Deserializer};
use yazi_shared::MIME_DIR;

use super::Opener;
use crate::{Preset, open::OpenRule};

#[derive(Debug)]
pub struct Open {
	rules:   Vec<OpenRule>,
	openers: HashMap<String, IndexSet<Opener>>,
}

impl Open {
	pub fn openers<P, M>(&self, path: P, mime: M) -> Option<IndexSet<&Opener>>
	where
		P: AsRef<Path>,
		M: AsRef<str>,
	{
		let is_dir = mime.as_ref() == MIME_DIR;
		self.rules.iter().find_map(|rule| {
			if rule.mime.as_ref().is_some_and(|p| p.match_mime(&mime))
				|| rule.name.as_ref().is_some_and(|p| p.match_path(&path, is_dir))
			{
				let openers = rule
					.use_
					.iter()
					.filter_map(|use_| self.openers.get(use_))
					.flatten()
					.collect::<IndexSet<_>>();

				if openers.is_empty() { None } else { Some(openers) }
			} else {
				None
			}
		})
	}

	#[inline]
	pub fn block_opener<P, M>(&self, path: P, mime: M) -> Option<&Opener>
	where
		P: AsRef<Path>,
		M: AsRef<str>,
	{
		self.openers(path, mime).and_then(|o| o.into_iter().find(|o| o.block))
	}

	pub fn common_openers(&self, targets: &[(impl AsRef<Path>, impl AsRef<str>)]) -> Vec<&Opener> {
		let grouped: Vec<_> = targets.iter().filter_map(|(p, m)| self.openers(p, m)).collect();
		let flat: IndexSet<_> = grouped.iter().flatten().copied().collect();
		flat.into_iter().filter(|&o| grouped.iter().all(|g| g.contains(o))).collect()
	}
}

impl FromStr for Open {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		toml::from_str(s).context("Failed to parse the [open] or [opener] section in your yazi.toml")
	}
}

impl<'de> Deserialize<'de> for Open {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct Outer {
			opener: HashMap<String, Vec<Opener>>,
			open:   OuterOpen,
		}
		#[derive(Deserialize)]
		struct OuterOpen {
			rules:         Vec<OpenRule>,
			#[serde(default)]
			prepend_rules: Vec<OpenRule>,
			#[serde(default)]
			append_rules:  Vec<OpenRule>,
		}

		let mut outer = Outer::deserialize(deserializer)?;

		if outer.open.append_rules.iter().any(|r| r.any_file()) {
			outer.open.rules.retain(|r| !r.any_file());
		}
		if outer.open.append_rules.iter().any(|r| r.any_dir()) {
			outer.open.rules.retain(|r| !r.any_dir());
		}

		let openers = outer
			.opener
			.into_iter()
			.map(|(k, v)| (k, v.into_iter().filter_map(|o| o.take()).collect::<IndexSet<_>>()))
			.collect();

		Ok(Self {
			#[rustfmt::skip]
			rules: Preset::mix(outer.open.prepend_rules, outer.open.rules, outer.open.append_rules).collect(),
			openers,
		})
	}
}
