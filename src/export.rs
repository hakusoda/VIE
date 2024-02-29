// export refers to exporting FROM the studio plugin
use std::{
	path::PathBuf,
	collections::HashMap
};
use serde::{ Serialize, Deserialize };
use indicatif::ProgressBar;
use crate::data::{ DataType, serialize_data_types };

#[derive(Serialize, Deserialize)]
pub struct PluginInstance {
	pub name: String,
	pub class: String,
	#[serde(skip_serializing)]
	pub source: Option<String>,
	#[serde(skip_serializing)]
	pub children: Option<Vec<PluginInstance>>,
	#[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_data_types")]
	pub properties: Option<HashMap<String, DataType>>,
	#[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_data_types")]
	pub attributes: Option<HashMap<String, DataType>>,

	#[serde(skip)]
	pub file_path: Option<PathBuf>
}

impl PluginInstance {
	pub fn total_instance_count(&self) -> u64 {
		let mut count = 1;
		if let Some(children) = &self.children {
			for child in children.iter() {
				count += child.total_instance_count();
			}
		}

		count
	}
}

fn instance_is_script(instance: &PluginInstance) -> bool {
	let class = &instance.class;
	class == "Script" || class == "LocalScript" || class == "ModuleScript"
}

pub fn write_plugin_instance(instance: &mut PluginInstance, root_dir: &PathBuf, current_dir: &PathBuf, is_root_dir: bool, progress_bar: &ProgressBar) {
	let current_dir = match !is_root_dir && instance.children.is_some() {
		true => current_dir.join(&instance.name),
		false => current_dir.clone()
	};
	if instance_is_script(instance) {
		let path = match is_root_dir {
			true => current_dir.join("+instance.luau"),
			false => match instance.children.is_some() {
				true => current_dir.join("+instance.luau"),
				false => current_dir.join(format!("{}.luau", instance.name))
			}
		};
		std::fs::create_dir_all(path.parent().unwrap()).unwrap();
		std::fs::write(&path, instance.source.as_ref().unwrap()).unwrap();

		instance.file_path = Some(path.strip_prefix(root_dir).unwrap().to_path_buf());
	} else {
		let path = match is_root_dir {
			true => current_dir.join("+instance.vie"),
			false => match instance.children.is_some() {
				true => current_dir.join("+instance.vie"),
				false => current_dir.join(format!("{}.vie", instance.name))
			}
		};
		if instance.class != "Folder" || instance.properties.is_some() {
			std::fs::create_dir_all(path.parent().unwrap()).unwrap();
			std::fs::write(path, serde_yaml::to_string(instance).unwrap().trim_end()).unwrap();
		}
	}

	progress_bar.inc(1);

	if let Some(children) = &mut instance.children {
		for instance in children.iter_mut() {
			write_plugin_instance(instance, &root_dir, &current_dir, false, &progress_bar);
		}
	}
}