use std::{
	net::SocketAddr,
	path::PathBuf
};
use serde::Deserialize;
use tokio::net::TcpListener;
use hyper::{
	body::{Buf, Bytes},
	server::conn::http1,
	service::service_fn,
	Request, Response
};
use hyper_util::rt::TokioIo;
use http_body_util::{ Full, BodyExt };

use crate::data::{
	Instance,
	RojoSourcemapInstance
};

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Payload {
	Import,
	Export {
		data: Vec<Instance>
	}
}

type GenericError = Box<dyn std::error::Error + Send + Sync>;

fn instance_is_script(instance: &Instance) -> bool {
	let class = &instance.class;
	class == "Script" || class == "LocalScript" || class == "ModuleScript"
}

fn write_instance(instance: &mut Instance, root_dir: &PathBuf, current_dir: &PathBuf, is_root_dir: bool) {
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

	if let Some(children) = &mut instance.children {
		for instance in children.iter_mut() {
			write_instance(instance, &root_dir, &current_dir, false);
		}
	}
}

async fn main_service(request: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, GenericError> {
	let body = request.collect().await?.aggregate();
	let payload: Payload = serde_json::from_reader(body.reader())?;

	match payload {
		Payload::Export { mut data } => {
			let root = data.first_mut().unwrap();
			let current_dir = std::env::current_dir().unwrap();

			let config = crate::config::read_config_file(current_dir.join("vie.config.yml"));
			let root_dir = match config.root_path {
				Some(value) => match value.is_relative() {
					true => current_dir.join(value),
					false => value
				},
				None => current_dir
			};
			let src_dir = root_dir.join("src");
			std::fs::remove_dir_all(&src_dir).unwrap();

			write_instance(root, &root_dir, &src_dir, true);

			if config.compatibility.rojo_sourcemap {
				let sourcemap = RojoSourcemapInstance::from(root);
				std::fs::write(root_dir.join("sourcemap.json"), serde_json::to_string(&sourcemap).unwrap()).unwrap();
			}
		},
		_ => unimplemented!()
	}
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let address = SocketAddr::from(([127, 0, 0, 1], 3143));
	let listener = TcpListener::bind(address).await?;
	println!("Click Import or Export in the Roblox Studio Plugin to continue...");

	let (stream, _) = listener.accept().await?;
	let io = TokioIo::new(stream);
	if let Err(err) = http1::Builder::new()
		.keep_alive(false)
		.serve_connection(io, service_fn(main_service))
		.await
	{
		println!("Error serving connection: {:?}", err);
	}

	drop(listener);
	println!("success!");
	Ok(())
}