#![feature(async_closure)]
#![feature(let_chains)]

mod cli;

use std::sync::LazyLock;

use cli::{CliArgs, HydePathArgs};
use hyde_lib::{error::ResultExtensions, HydeConfig};
use notify::Watcher;

// pub static ARGS: LazyLock<CliArgs> = LazyLock::new(<CliArgs as clap::Parser>::parse);

fn main() {
	use cli::HydeCommand::*;

	let args = <CliArgs as clap::Parser>::parse();

	match args.command {
		Completion { shell } => cli::generate_completion(shell),
		Build(paths) => hyde_lib::process_project(&init_config(paths)).handle_as_error(),
		Serve(paths) => serve(init_config(paths)),
		Clean(paths) => if let HydeConfig { output_dir, .. } = init_config(paths) && output_dir.exists() {
			std::fs::remove_dir_all(&output_dir).handle_as_error();
		}
	}
}

fn init_config(paths: HydePathArgs) -> HydeConfig {
	let project_dir = paths.dir.map_or(std::env::current_dir().unwrap(), |p| std::env::current_dir().unwrap().join(p));

	hyde_lib::HydeConfig {
		source_dir: project_dir.join("src"),
		output_dir: paths.out.unwrap_or(project_dir.join("site")),
		plugins_dir: project_dir.join("plugins"),
		layout_dir: project_dir.join("layouts"),
		project_dir,
	}
}

fn serve(config: HydeConfig) {
	let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
	let mut watcher = notify::recommended_watcher(tx).unwrap();
	watcher.watch(&config.source_dir, notify::RecursiveMode::Recursive).unwrap();
	watcher.watch(&config.layout_dir, notify::RecursiveMode::Recursive).unwrap();
	watcher.watch(&config.plugins_dir, notify::RecursiveMode::Recursive).unwrap();

	let runtime = tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.unwrap();

	let reload_layer = tower_livereload::LiveReloadLayer::new();
	let reload_handle = reload_layer.reloader();

	let serve_path = config.output_dir.clone();
	let server = async move || {
		let app = axum::Router::new()
			.nest_service("/", tower_http::services::ServeDir::new(serve_path))
			.layer(reload_layer)
			;

		let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
		axum::serve(listener, app).await.unwrap();
	};
	
	let _handle = runtime.spawn(server());

	let mut to_reload = false;

	loop {
		while let Ok(res) = rx.try_recv() {
			match res {
				Err(e) => println!("watch error: {e:?}"),
				Ok(notify::Event { kind: notify::EventKind::Access(_), ..}) |
				Ok(notify::Event { kind: notify::EventKind::Any, ..}) |
				Ok(notify::Event { kind: notify::EventKind::Other, ..}) => (),
				Ok(_) => to_reload = true,
			}
		}

		if to_reload {
			if let Err(e) = hyde_lib::process_project(&config) {
				println!("{e}");
			}

			reload_handle.reload();
			to_reload = false;
		}

		std::thread::sleep(std::time::Duration::from_millis(100));
	}
}
