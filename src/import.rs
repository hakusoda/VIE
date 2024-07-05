// import refers to importing FROM the studio plugin
use std::{
	io::BufReader,
	fs::{ DirEntry, File },
	path::PathBuf,
	collections::HashMap
};
use serde::{
	Deserialize, Serialize
};

#[derive(Serialize, Deserialize)]
pub struct FsInstance {
	pub name: String,
	pub class: String,
	#[serde(skip_deserializing)]
	pub children: Vec<FsInstance>,
	#[serde(default)]
	pub attributes: Option<HashMap<String, serde_yaml::Value>>,
	#[serde(default)]//, deserialize_with = "deserialize_data_types")]
	pub properties: Option<HashMap<String, serde_yaml::Value>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub script_source: Option<String>
}

/*fn deserialize_data_types<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<HashMap<String, DataType>>, D::Error> {
	let value = serde_yaml::Value::deserialize(deserializer).unwrap();
	let mapping = value.as_mapping().unwrap();

	let mut map: HashMap<String, DataType> = HashMap::new();
	for (key, value) in mapping.iter() {
		map.insert(key.as_str().unwrap().to_string(), if value.is_bool() {
			DataType::Boolean(value.as_bool().unwrap())
		} else if value.is_f64() {
			DataType::Double(value.as_f64().unwrap())
		} else {
			DataType::String(serde_yaml::to_string(value).unwrap().trim_end().into())
		});
		//map.insert(key.as_str().unwrap().to_string(), DataType::String(serde_yaml::to_string(value).unwrap().trim_end().into()));
	}

	Ok(Some(map))
}*/

fn read_instance(path: impl Into<PathBuf>) -> Option<FsInstance> {
	File::open(path.into()).map(|x| serde_yaml::from_reader(BufReader::new(x)).unwrap()).ok()
}

fn get_script_properties(file_name: impl Into<String>) -> Option<HashMap<String, serde_yaml::Value>> {
	let file_name = file_name.into();
	let is_server = file_name.ends_with(".server.luau");
	let is_client = file_name.ends_with(".client.luau");
	if is_server || is_client {
		Some(HashMap::from([
			("RunContext".into(), serde_yaml::to_value(match is_server {
				true => "Server",
				false => "Client"
			}).unwrap())//DataType::EnumItem("Server".into()))
		]))
	} else { None }
}

pub fn read_instance_tree(current_dir: &PathBuf, is_root_dir: bool) -> Vec<FsInstance> {
	let mut items: Vec<FsInstance> = vec![];
	if let Ok(reader) = std::fs::read_dir(current_dir) {
		let entries: Vec<DirEntry> = reader.flatten().collect();
		let mut root_instance = match entries.iter().find(|x| {
			let file_name = x.file_name();
			let file_name = file_name.to_string_lossy();
			file_name == "+instance.vie" || (file_name.starts_with("+instance") && file_name.ends_with(".luau"))
		}) {
			Some(entry) => {
				let file_name = entry.file_name();
				let file_name = file_name.to_string_lossy();
				if file_name.ends_with(".luau") {
					let properties = get_script_properties(file_name);
					Some(FsInstance {
						name: current_dir.file_name().unwrap().to_string_lossy().into(),
						class: match properties.is_some() {
							true => "Script",
							false => "ModuleScript"
						}.into(),
						children: vec![],
						attributes: None,
						properties,
						script_source: Some(std::fs::read_to_string(entry.path()).unwrap())
					})
				} else {
					let mut instance = read_instance(entry.path()).unwrap();
					instance.children = vec![];
					Some(instance)
				}
			},
			None => match is_root_dir {
				true => None,
				false => Some(FsInstance {
					name: current_dir.file_name().unwrap().to_string_lossy().to_string(),
					class: "Folder".into(),
					children: vec![],
					attributes: None,
					properties: None,
					script_source: None
				})
			}
		};

		let mut push_item = |x: FsInstance, items: &mut Vec<FsInstance>| match &mut root_instance {
			Some(instance) => instance.children.push(x),
			None => items.push(x)
		};
		for entry in entries {
			let file_type = entry.file_type().unwrap();
			if file_type.is_file() {
				let file_name = entry.file_name();
				if file_name != "+instance.vie" && file_name != "+instance.luau" && file_name != "+instance.server.luau" && file_name != "+instance.client.luau" {
					let file_name = file_name.to_string_lossy();
					if file_name.ends_with(".reference.vie") {

					} else if file_name.ends_with(".luau") {
						let properties = get_script_properties(file_name.as_ref());
						push_item(FsInstance {
							name: file_name[..file_name.len() - 5].into(),
							class: match properties.is_some() {
								true => "Script",
								false => "ModuleScript"
							}.into(),
							children: vec![],
							attributes: None,
							properties,
							script_source: Some(std::fs::read_to_string(entry.path()).unwrap())
						}, &mut items);
					} else if file_name.ends_with(".vie") {
						push_item(read_instance(entry.path()).unwrap(), &mut items);
					}
				}
			} else if file_type.is_dir() {
				for item in read_instance_tree(&entry.path(), false) {
					push_item(item, &mut items);
				}
			}
		}

		if let Some(instance) = root_instance {
			items.push(instance);
		}
	}

	items
}