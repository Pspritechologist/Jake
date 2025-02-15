#![feature(async_closure)]
#![feature(let_chains)]

mod cli;

use std::sync::LazyLock;

use cli::CliArgs;
use iter_tools::Itertools;
use jake_lib::{error::ResultExtensions, JakeConfig};
use notify::Watcher;

pub static ARGS: LazyLock<CliArgs> = LazyLock::new(<CliArgs as clap::Parser>::parse);

fn main() {
	use cli::JakeCommand::*;

	match &ARGS.command {
		Completion { shell } => cli::generate_completion(*shell),
		Build => jake_lib::process_project(&init_config()).handle_as_error(),
		Serve { port } => serve(init_config(), port.unwrap_or(4000)),
		Clean => {
			let JakeConfig { output_dir, .. } = init_config();
			if output_dir.exists() {
				std::fs::remove_dir_all(&output_dir).handle_as_error();
			}
		}
	}
}

fn init_config() -> JakeConfig {
	let paths = &ARGS.path_args;

	let project_dir = paths.dir.as_ref().map_or(std::env::current_dir().unwrap(), |p| std::env::current_dir().unwrap().join(p));

	// Check for the 'jake.yml' file.
	if !project_dir.join("jake.yml").exists() {
		eprintln!("No 'jake.yml' file found in the project directory.");
		std::process::exit(1);
	}

	jake_lib::JakeConfig {
		source_dir: project_dir.join("src"),
		output_dir: paths.out.to_owned().unwrap_or(project_dir.join("site")),
		plugins_dir: project_dir.join("plugins"),
		layout_dir: project_dir.join("layouts"),
		project_dir,
	}
}

fn serve(config: JakeConfig, port: u16) {
	fn msg(msg: &str, err: bool) {
		let col = if err { "\x1b[31m" } else { "\x1b[32m" };
		println!("{col}{msg}\x1b[0m");
	}

	jake_lib::process_project(&config).handle_as_error();

	eprintln!();

	let (tx, rx) = std::sync::mpsc::channel();
	let tx_clone = tx.clone();
	let mut watcher = notify::recommended_watcher(move |ev| tx_clone.send(Msg::Watcher(ev)).unwrap()).unwrap();
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
			.fallback_service(tower_http::services::ServeDir::new(serve_path))
			.layer(reload_layer)
			;

		let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.unwrap();
		axum::serve(listener, app).await.unwrap();
	};

	msg(&format!("Serving site at http://localhost:{}/", port), false);
	
	let _handle = runtime.spawn(server());

	fn cause_reload(ev: notify::Result<notify::Event>) -> bool {
		if let Err(e) = ev {
			msg(&format!("watch error: {e:?}"), true);
			return false;
		}

		matches!(
			ev.unwrap().kind,
			notify::EventKind::Create(_)
			| notify::EventKind::Modify(_)
			| notify::EventKind::Remove(_)
		)
	}

	enum Msg {
		Watcher(notify::Result<notify::Event>),
		Stdin(StdInMsg),
	}

	enum StdInMsg {
		Pause(bool),
		Refresh,
	}
	use StdInMsg::*;

	let std_watcher = async move {
		let stdin = std::io::stdin();

		loop {
			let mut buf = String::new();
			stdin.read_line(&mut buf).unwrap();
			
			match buf.trim() {
				"pause" => {
					msg("\x1b[1APaused", false);
					tx.send(Msg::Stdin(Pause(true))).unwrap()
				},
				"start" => {
					msg("\x1b[1AResumed", false);
					tx.send(Msg::Stdin(Pause(false))).unwrap()
				},
				"refresh" => {
					msg("\x1b[1ARefreshing...", false);
					tx.send(Msg::Stdin(Refresh)).unwrap()
				},
				_ => msg(&format!("\x1bUnknown command: {}", buf.trim()), true),
			}

			buf.clear();
		}
	};

	let _std_handle = runtime.spawn(std_watcher);

	let mut paused = false;

	loop {
		let Ok(res) = rx.recv() else {
			msg("Channel closed unexpectedly", true);
			break;
		};
		std::thread::sleep(std::time::Duration::from_millis(100));

		let (stdins, watches): (Vec<_>, Vec<_>) = rx.try_iter().partition_map(|ev| match ev {
			Msg::Stdin(msg) => iter_tools::Either::Left(msg),
			Msg::Watcher(ev) => iter_tools::Either::Right(ev),
		});

		let mut refresh = false;

		if let Msg::Stdin(msg) = &res {
			match msg {
				Pause(p) => paused = *p,
				Refresh => refresh = true,
			}
		}

		for msg in stdins {
			match msg {
				Pause(p) => paused = p,
				Refresh => refresh = true,
			}
		}

		let reload = refresh || (!paused && (
			if let Msg::Watcher(ev) = res { cause_reload(ev) } else { false }
				|| watches.into_iter().any(cause_reload)
		));

		if !reload { continue; }

		msg("Reloading site...", false);
		jake_lib::process_project(&config).handle_as_error();
		eprintln!();
		msg("Site reloaded", false);

		reload_handle.reload();
	}
}
