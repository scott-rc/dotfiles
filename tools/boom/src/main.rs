use std::path::{Path, PathBuf};
use std::process;

use clap::{Parser, Subcommand};

mod client;

#[derive(Parser)]
#[command(name = "boom", about = "Kubernetes deploy tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Deploy(DeployArgs),
    GlobalDeploy(GlobalDeployArgs),
    Restart(RestartArgs),
    Render(RenderArgs),
}

#[derive(Parser)]
struct DeployArgs {
    #[arg(short, long)]
    namespace: String,
    #[arg(long)]
    dir: PathBuf,
    #[arg(long)]
    values: Option<PathBuf>,
    #[arg(long)]
    context: Option<String>,
    #[arg(long)]
    selector: Option<String>,
    #[arg(long, default_value_t = 300)]
    global_timeout: u64,
    #[arg(long, default_value_t = true)]
    verify_result: bool,
    #[arg(long)]
    prune: bool,
}

#[derive(Parser)]
struct GlobalDeployArgs {
    #[arg(long)]
    dir: PathBuf,
    #[arg(long)]
    values: Option<PathBuf>,
    #[arg(long)]
    context: Option<String>,
    #[arg(long)]
    selector: Option<String>,
    #[arg(long, default_value_t = 300)]
    global_timeout: u64,
    #[arg(long, default_value_t = true)]
    verify_result: bool,
    #[arg(long)]
    prune: bool,
}

#[derive(Parser)]
struct RestartArgs {
    #[arg(short, long)]
    namespace: String,
    #[arg(long)]
    context: Option<String>,
    #[arg(long, value_delimiter = ',')]
    deployments: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    statefulsets: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    daemonsets: Vec<String>,
    #[arg(long, default_value_t = 300)]
    global_timeout: u64,
    #[arg(long, default_value_t = true)]
    verify_result: bool,
}

#[derive(Parser)]
struct RenderArgs {
    #[arg(long)]
    template_dir: Option<String>,
    #[arg(long)]
    bindings: Vec<String>,
    #[arg(long)]
    bindings_file: Option<String>,
    #[arg(long)]
    current_sha: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Deploy(args) => deploy(args).await,
        Commands::GlobalDeploy(args) => global_deploy(args).await,
        Commands::Restart(args) => restart(args).await,
        Commands::Render(args) => render(args).await,
    }
}

async fn deploy(args: DeployArgs) {
    let client = client::build_client(args.context.as_deref()).await;

    let yaml = if let Some(ref values_path) = args.values {
        let bindings = boom::render::load_bindings_file(values_path);
        let templates = boom::render::load_templates(&args.dir);
        match boom::render::render_templates(&templates, &bindings) {
            Ok(output) => output,
            Err(e) => {
                eprintln!("boom: {e}");
                process::exit(1);
            }
        }
    } else {
        let templates = boom::render::load_templates(&args.dir);
        let bindings = std::collections::HashMap::new();
        match boom::render::render_templates(&templates, &bindings) {
            Ok(output) => output,
            Err(e) => {
                eprintln!("boom: {e}");
                process::exit(1);
            }
        }
    };

    let resources = boom::manifest::parse_manifests(&yaml);
    let ok = boom::deploy::run(
        client.clone(),
        &args.namespace,
        resources.clone(),
        args.verify_result,
        args.global_timeout,
    )
    .await;
    if ok {
        if args.prune {
            boom::output::info("[boom] pruning stale resources");
            let deployed: Vec<boom::prune::ResourceDescriptor> = resources
                .iter()
                .map(|r| boom::prune::ResourceDescriptor {
                    name: r.name.clone(),
                    kind: r.kind.clone(),
                    namespace: r.namespace.clone().unwrap_or(args.namespace.clone()),
                })
                .collect();
            let stale = boom::prune::identify_stale(&deployed, &deployed);
            if stale.is_empty() {
                boom::output::success("[boom] no stale resources to prune");
            } else {
                boom::output::info(&format!("[boom] pruning {} stale resources", stale.len()));
                if let Err(e) = boom::prune::execute(&client, &stale).await {
                    boom::output::error(&format!("[boom] prune failed: {e}"));
                    process::exit(1);
                }
            }
        }
        process::exit(0);
    } else {
        process::exit(1);
    }
}

async fn global_deploy(args: GlobalDeployArgs) {
    let client = client::build_client(args.context.as_deref()).await;

    let yaml = if let Some(ref values_path) = args.values {
        let bindings = boom::render::load_bindings_file(values_path);
        let templates = boom::render::load_templates(&args.dir);
        match boom::render::render_templates(&templates, &bindings) {
            Ok(output) => output,
            Err(e) => {
                eprintln!("boom: {e}");
                process::exit(1);
            }
        }
    } else {
        let templates = boom::render::load_templates(&args.dir);
        let bindings = std::collections::HashMap::new();
        match boom::render::render_templates(&templates, &bindings) {
            Ok(output) => output,
            Err(e) => {
                eprintln!("boom: {e}");
                process::exit(1);
            }
        }
    };

    let resources = boom::manifest::parse_manifests(&yaml);
    boom::global_deploy::run(client, resources, args.verify_result, args.global_timeout).await;
}

async fn restart(args: RestartArgs) {
    let client = client::build_client(args.context.as_deref()).await;
    boom::restart::run(
        client,
        &args.namespace,
        &args.deployments,
        &args.statefulsets,
        &args.daemonsets,
        args.verify_result,
        args.global_timeout,
    )
    .await;
}

#[allow(clippy::unused_async)]
async fn render(args: RenderArgs) {
    let dir = args.template_dir.as_deref().unwrap_or(".");

    let mut bindings = if let Some(ref file) = args.bindings_file {
        boom::render::load_bindings_file(Path::new(file))
    } else {
        std::collections::HashMap::new()
    };

    for (k, v) in boom::render::parse_bindings(&args.bindings) {
        bindings.insert(k, v);
    }

    if args.current_sha {
        let output = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap_or_else(|e| {
                eprintln!("boom: failed to run git rev-parse HEAD: {e}");
                process::exit(1);
            });
        if !output.status.success() {
            eprintln!("boom: git rev-parse HEAD failed");
            process::exit(1);
        }
        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
        bindings.insert("current_sha".to_string(), sha);
    }

    let templates = boom::render::load_templates(Path::new(dir));
    match boom::render::render_templates(&templates, &bindings) {
        Ok(output) => println!("{output}"),
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    }
}
