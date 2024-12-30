#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
/// Rusty Jekyll
pub struct CliArgs {
	#[arg(short, long, env="HYDE_DIR")]
	/// The path of the project directory to process.  
	/// If omitted, the current directory will be used.
	pub dir: Option<std::path::PathBuf>,

	#[arg(short, long, env="HYDE_OUT_DIR")]
	/// The path of the output directory to write to.  
	/// The default for the project will be used if omitted.
	pub out: Option<std::path::PathBuf>,
}

// #[derive(Debug, Clone, clap::Subcommand)]
// pub enum SubCommand {
// 	/// Generates shell completions for the CLI.
// 	/// 
// 	/// Either run `source <(ssu completions SHELL)` or add the same line to your shell's configuration file.
// 	Completion { shell: clap_complete::Shell },
// 	List(CmdList),
// 	Shutdown(CmdShutdown),
// 	Startup(CmdStartup),
// 	Rescale(CmdRescale),
// 	NginxConfig(CmdNginxConfig),
// 	#[cfg(debug_assertions)]
// 	Debug,
// }

// #[derive(Debug, Clone, clap::Parser)]
// #[command(visible_alias="ls")]
// /// Lists all nodes.
// pub struct CmdList {
// 	#[arg(short='l', long)]
// 	/// List full details regarding the nodes.
// 	pub full: bool,
// 	#[arg(short, long)]
// 	/// Output in JSON format.
// 	pub json: bool,
// 	#[arg(short, long, value_enum, default_value_t=NodeSortOpt::Created)]
// 	/// Sort the nodes by a specific quality.
// 	pub sort: NodeSortOpt,
// 	#[arg(short, long)]
// 	/// Reverse the ordering of the Nodes.
// 	pub reverse: bool,
// }

// #[derive(Debug, Clone, clap::ValueEnum)]
// pub enum NodeSortOpt {
// 	#[clap(alias="a")]
// 	/// Sort by name alphabetically.
// 	Alphabetical,
// 	#[clap(alias="c")]
// 	/// Sort by creation date of the Node.
// 	Created,
// 	#[clap(alias="s")]
// 	/// Sort by Node's CPU core count.
// 	Cores,
// 	#[clap(alias="m")]
// 	/// Sort by Node's memory.
// 	Memory,
// 	#[clap(alias="s")]
// 	/// Sort by the Node's status.
// 	Status,
// }

// #[derive(Debug, Clone, clap::Parser)]
// #[command()]
// /// Shutdown a node.
// pub struct CmdShutdown {
// 	#[arg(required=true)]
// 	/// The ID or name of the node to shutdown.
// 	/// 
// 	/// Using a name requires an additional request to the API. Use an ID when available.
// 	pub node: String,
// 	#[arg(short, long)]
// 	/// Restart the node after shutting it down.
// 	pub restart: bool,
// 	#[arg(long)]
// 	/// Forces a power down.
// 	/// 
// 	/// This is the equivalent of pulling the power cord on a physical server.
// 	pub force: bool,
// }

// #[derive(Debug, Clone, clap::Parser)]
// #[command()]
// /// Startup a node
// pub struct CmdStartup {
// 	#[arg(required=true)]
// 	/// The ID or name of the node to startup
// 	/// 
// 	/// Using a name requires an additional request to the API. Use an ID when available.
// 	pub node: String,
// }

// #[derive(Debug, Clone, clap::Parser)]
// #[command()]
// /// Rescales a Node from its current shape to a compatible shape. Can also list all compatible shapes.
// pub struct CmdRescale {
// 	#[arg(required=true)]
// 	/// The ID or name of the node to rescale.
// 	/// 
// 	/// Using a name requires an additional request to the API. Use an ID when available.
// 	pub node: String,
// 	#[arg()]
// 	/// The numeric ID of the new shape to rescale the Node to.
// 	/// 
// 	/// Lists all compatible shapes if omitted.
// 	pub shape: Option<u16>,
// }

// #[derive(Debug, Clone, clap::Parser)]
// #[command(alias="nginx")]
// /// Generates Nginx configuration files for SimpleStation servers.
// pub struct CmdNginxConfig {
// 	#[arg(env="SSU_NGINX_CONFIG")]
// 	/// The path to the TOML file containing the server configurations.
// 	pub config: std::path::PathBuf,
// 	/// The Nginx configuration file to write to.
// 	/// 
// 	/// NOTE: This is a destructive action. If the file already exists, it will be overwritten.
// 	/// If omitted, the configuration will be printed to stdout.
// 	#[arg(env="SSU_NGINX_OUTPUT")]
// 	pub output: Option<std::path::PathBuf>,
// }

#[test]
pub fn verify_cmd() {
	use clap::CommandFactory;
	CliArgs::command().debug_assert();
}
