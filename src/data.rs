use std::{
	hash::Hash,
	collections::HashMap
};
use serde::{
	ser::SerializeMap,
	Serialize, Serializer, Deserialize
};
use serde_yaml::value::TaggedValue;
use linked_hash_map::LinkedHashMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value")]
pub enum DataType {
	#[serde(rename = "string")]
	String(String),
	#[serde(rename = "double", alias = "number")]
	Double(f64),
	#[serde(rename = "float")]
	Float(f32),
	#[serde(rename = "int")]
	Integer(i32),
	#[serde(rename = "int64")]
	Integer64(i64),
	#[serde(rename = "boolean")]
	Boolean(bool),
	EnumItem(String),
	Instance,
	CFrame(String),
	Vector3([f64; 3]),
	Vector2([f64; 2]),
	#[serde(rename = "nil")]
	Nil,
	Color3([u8; 3]),
	BrickColor(String),
	ColorSequence(String),
	UDim([f32; 2]),
	UDim2([[f32; 2]; 2]),
	Font {
		style: String,
		family: String,
		weight: String
	},
	Rect(String)
}

fn linked_map_from<K: Eq + Hash, V, const N: usize>(array: [(K, V); N]) -> LinkedHashMap<K, V> {
	let mut map = LinkedHashMap::new();
	for (key, value) in array {
		map.insert(key, value);
	}

	map
}

fn udim_to_json(udim: [f32; 2]) -> serde_json::Value {
	serde_json::json!({
		"scale": udim[0],
		"offset": udim[1] as i64
	})
}

fn tag_value<T: Serialize>(tag: impl Into<String>, value: T) -> TaggedValue {
	TaggedValue {
		tag: serde_yaml::value::Tag::new(tag),
		value: serde_yaml::to_value(value).unwrap()
	}
}

pub fn serialize_data_types<S: Serializer>(data_types: &Option<HashMap<String, DataType>>, serializer: S) -> Result<S::Ok, S::Error> {
	match data_types {
		Some(data_types) => {
			let mut sorted_data_types = data_types.iter().collect::<Vec<(&String, &DataType)>>();
			sorted_data_types.sort_by(|a, b| a.0.partial_cmp(b.0).unwrap());

			let mut map = serializer.serialize_map(Some(data_types.len())).unwrap();
			for (key, value) in sorted_data_types {
				match value {
					DataType::String(value) |
					DataType::EnumItem(value) |
					DataType::CFrame(value) |
					DataType::BrickColor(value) |
					DataType::ColorSequence(value) |
					DataType::Rect(value) => map.serialize_entry(key, value).unwrap(),
					DataType::Double(value) => map.serialize_entry(key, value).unwrap(),
					DataType::Float(value) => map.serialize_entry(key, value).unwrap(),
					DataType::Integer(value) => map.serialize_entry(key, value).unwrap(),
					DataType::Integer64(value) => map.serialize_entry(key, value).unwrap(),
					DataType::Boolean(value) => map.serialize_entry(key, value).unwrap(),
					DataType::Vector3([x, y, z]) =>
						map.serialize_entry(key, &tag_value("Vector3", linked_map_from([
							("x", x),
							("y", y),
							("z", z)
						]))).unwrap(),
					DataType::Vector2([x, y]) =>
						map.serialize_entry(key, &tag_value("Vector2", linked_map_from([
							("x", x),
							("y", y)
						]))).unwrap(),
					DataType::Color3([r, g, b]) =>
						map.serialize_entry(key, &tag_value("Color3", linked_map_from([
							("red", r),
							("green", g),
							("blue", b)
						]))).unwrap(),
					DataType::UDim(value) =>
						map.serialize_entry(key, &tag_value("UDim", udim_to_json(*value))).unwrap(),
					DataType::UDim2(value) => 
						map.serialize_entry(key, &tag_value("UDim2", [
							udim_to_json(value[0]),
							udim_to_json(value[1])
						])).unwrap(),
					DataType::Font { style, family, weight } =>
						map.serialize_entry(key, &tag_value("Font", linked_map_from([
							("style", style),
							("family", family),
							("weight", weight)
						]))).unwrap(),
					_ => ()
				};
			}
			map.end()
		},
		_ => serializer.serialize_none()
	}
}