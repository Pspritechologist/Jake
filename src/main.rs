#![feature(async_closure)]

use notify::Watcher;

fn main() {
	let args = <hyde::cli::CliArgs as clap::Parser>::parse();
	let project_dir = args.dir.map_or(std::env::current_dir().unwrap(), |p| std::env::current_dir().unwrap().join(p));
	let source_dir = project_dir.join("src");
	let output_dir = args.out.unwrap_or(project_dir.join("site"));
	let plugins_dir = project_dir.join("plugins");
	let layout_dir = project_dir.join("layouts");

	let config = hyde::HydeConfig {
		project_dir,
		output_dir,
		source_dir,
		plugins_dir,
		layout_dir,
	};

	hyde::process_dir(&config).unwrap();

	let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
	let mut watcher = notify::recommended_watcher(tx).unwrap();
	watcher.watch(&config.source_dir, notify::RecursiveMode::Recursive).unwrap();

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
				Ok(_) => {
					to_reload = true;
				}
			}
		}

		if to_reload {
			hyde::process_dir(&config).unwrap();

			reload_handle.reload();
			to_reload = false;
		}
	}

	// for res in rx {
	// 	match res {
	// 		Err(e) => println!("watch error: {e:?}"),
	// 		Ok(notify::Event { kind: notify::EventKind::Access(_), ..}) |
	// 		Ok(notify::Event { kind: notify::EventKind::Any, ..}) |
	// 		Ok(notify::Event { kind: notify::EventKind::Other, ..}) => (),
	// 		Ok(_) => {
	// 			reload_handle.reload();
	// 			// handle.abort();
	// 			// hyde::process_dir(&config).unwrap();
	// 			// handle = runtime.spawn(start_webserver(config.output_dir.clone()));
	// 		}
	// 	}
	// }
}
