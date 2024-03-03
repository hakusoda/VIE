use std::path::PathBuf;
use serde::Serialize;
use crate::export::PluginExportItem;

#[derive(Serialize)]
pub struct RojoSourcemapInstance {
	name: String,
	#[serde(rename = "className")]
	class: String,
	#[serde(rename = "filePaths", skip_serializing_if = "Option::is_none")]
	paths: Option<Vec<PathBuf>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	children: Option<Vec<RojoSourcemapInstance>>
}

impl RojoSourcemapInstance {
	pub fn from(item: &PluginExportItem) -> Option<Self> {
		match item {
			PluginExportItem::Instance(instance) => Some(RojoSourcemapInstance {
				name: instance.name.clone(),
				class: instance.class.clone(),
				paths: instance.file_path.clone().map(|x| vec![x]),
				children: instance.children.as_ref().map(|x| x.iter().filter_map(|x| RojoSourcemapInstance::from(x)).collect())
			}),
			PluginExportItem::ModelReference { name, class, asset_id: _ } => Some(RojoSourcemapInstance {
				name: name.clone(),
				class: class.clone(),
				paths: None,
				children: None
			})
		}
	}
}