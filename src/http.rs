use std::net::SocketAddr;
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

use crate::{
	export::{ PluginInstance, write_plugin_instance },
	compatibility::rojo::RojoSourcemapInstance
};

#[derive(Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
enum Payload {
	Import,
	Export(PluginInstance)
}

type GenericError = Box<dyn std::error::Error + Send + Sync>;

async fn main_service(request: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, GenericError> {
	let body = request.collect().await?.aggregate();
	let payload: Payload = serde_json::from_reader(body.reader())?;

	match payload {
		Payload::Export(mut root_instance) => {
			let current_dir = std::env::current_dir().unwrap();

			let config = crate::config::read_config_file(current_dir.join("vie.config.yml"));
			let root_dir = match config.root_path {
				Some(value) => match value.is_relative() {
					true => current_dir.join(value),
					false => value
				},
				None => current_dir.clone()
			};
			let src_dir = root_dir.join("src");
			let _ = std::fs::remove_dir_all(&src_dir);
			write_plugin_instance(&mut root_instance, &current_dir, &src_dir, true);

			if config.compatibility.rojo_sourcemap {
				let sourcemap = RojoSourcemapInstance::from(&root_instance);
				std::fs::write(current_dir.join("sourcemap.json"), serde_json::to_string(&sourcemap).unwrap()).unwrap();
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