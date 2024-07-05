// export refers to exporting FROM the studio plugin
use std::{
	path::PathBuf,
	collections::HashMap
};
use serde::{ Serialize, Deserialize };
use indicatif::ProgressBar;
use crate::data::{ DataType, serialize_data_types };

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PluginExportItem {
	Instance(PluginInstance),
	ModelReference {
		name: String,
		class: String,
		asset_id: u64
	}
}

impl PluginExportItem {
	pub fn total_instance_count(&self) -> u64 {
		match self {
			PluginExportItem::Instance(instance) => instance.total_instance_count(),
			PluginExportItem::ModelReference { name: _, class: _, asset_id: _ } => 1
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct PluginInstance {
	pub name: String,
	pub class: String,
	#[serde(skip_serializing)]
	pub source: Option<String>,
	#[serde(skip_serializing)]
	pub children: Option<Vec<PluginExportItem>>,
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

fn get_script_extension(instance: &PluginInstance) -> String {
	match instance.class.as_str() {
		"Script" => match crate::cast!(instance.properties.as_ref().unwrap().get("RunContext").unwrap(), DataType::EnumItem).as_str() {
			"Server" => ".server",
			"Client" => ".client",
			_ => ""
		},
		"LocalScript" => ".client",
		_ => ""
	}.to_string() + ".luau"
}

pub fn write_plugin_item(item: &mut PluginExportItem, root_dir: &PathBuf, current_dir: &PathBuf, is_root_dir: bool, progress_bar: &ProgressBar) {
	match item {
		PluginExportItem::Instance(instance) => {
			let current_dir = match !is_root_dir && instance.children.is_some() {
				true => current_dir.join(&instance.name),
				false => current_dir.clone()
			};
			if instance_is_script(instance) {
				let name = match is_root_dir || instance.children.is_some()  {
					true => "+instance".into(),
					false => instance.name.clone()
				} + &get_script_extension(instance);
				let path = current_dir.join(&name);
				std::fs::create_dir_all(path.parent().unwrap()).unwrap();
				std::fs::write(&path, instance.source.as_ref().unwrap()).unwrap();

				if let Some(properties) = &mut instance.properties {
					properties.remove("Source");
					properties.remove("RunContext");
					if properties.is_empty() {
						instance.properties = None;
					}
				}

				if instance.attributes.is_some() || instance.properties.is_some() {
					std::fs::write(current_dir.join(name.replace(".luau", ".vie")), serde_yaml::to_string(instance).unwrap().trim_end()).unwrap();
				}
		
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
				for item in children.iter_mut() {
					write_plugin_item(item, &root_dir, &current_dir, false, &progress_bar);
				}
			}
		},
		PluginExportItem::ModelReference { name, class: _, asset_id } => {
			let path = match is_root_dir {
				true => current_dir.join("+reference.vie"),
				false => current_dir.join(format!("{name}.reference.vie"))
			};
			std::fs::create_dir_all(path.parent().unwrap()).unwrap();
			std::fs::write(path, serde_yaml::to_string(&serde_json::json!({
				"kind": "model",
				"asset_id": asset_id
			})).unwrap().trim_end()).unwrap();
		}
	}
}