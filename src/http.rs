use std::net::SocketAddr;
use std::path::PathBuf;
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

use crate::data::Instance;

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

fn write_instance(instance: &Instance, current_dir: &PathBuf, is_root_dir: bool) {
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
		std::fs::write(path, instance.source.as_ref().unwrap()).unwrap();
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

	if let Some(children) = &instance.children {
		for instance in children.iter() {
			write_instance(instance, &current_dir, false);
		}
	}
}

async fn hello(request: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, GenericError> {
	let body = request.collect().await?.aggregate();
	let payload: Payload = serde_json::from_reader(body.reader())?;
	match payload {
		Payload::Export { data } => {
			let root = data.first().unwrap();
			let root_dir = std::env::current_dir().unwrap().join("src");
			std::fs::remove_dir_all(&root_dir).unwrap();

			write_instance(root, &root_dir, true);
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
		.serve_connection(io, service_fn(hello))
		.await
	{
		println!("Error serving connection: {:?}", err);
	}

	drop(listener);
	println!("success!");
	Ok(())
}