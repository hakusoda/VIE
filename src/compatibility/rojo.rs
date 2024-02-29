use std::path::PathBuf;
use serde::Serialize;
use crate::export::PluginInstance;

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
	pub fn from(instance: &PluginInstance) -> Self {
		RojoSourcemapInstance {
			name: instance.name.clone(),
			class: instance.class.clone(),
			paths: instance.file_path.clone().map(|x| vec![x]),
			children: instance.children.as_ref().map(|x| x.iter().map(|x| RojoSourcemapInstance::from(x)).collect())
		}
	}
}