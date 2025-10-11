use clap::Parser;
use tracing::{error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use vectdb::cli::{Cli, Commands};
use vectdb::config::{get_default_config_path, Config};
use vectdb::Result;

#[tokio::main]
async fn main() {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize logging
    if let Err(e) = init_logging(&cli.log_level) {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    }

    info!("VectDB starting...");

    // Load configuration
    let config = match Config::load(cli.config.clone()) {
        Ok(config) => {
            info!("Configuration loaded successfully");
            config
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Execute the command
    if let Err(e) = execute_command(cli.command, config).await {
        error!("Command failed: {}", e);
        std::process::exit(1);
    }

    info!("VectDB finished successfully");
}

/// Initialize the tracing subscriber for logging
fn init_logging(log_level: &str) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .try_init()
        .map_err(|e| vectdb::VectDbError::Other(format!("Failed to initialize logging: {}", e)))?;

    Ok(())
}

/// Execute the appropriate command
async fn execute_command(command: Commands, config: Config) -> Result<()> {
    match command {
        Commands::Init { force } => {
            info!("Initializing VectDB configuration");
            handle_init(force, config).await
        }
        Commands::Ingest {
            source,
            model,
            chunk_size,
            overlap,
            recursive,
        } => {
            info!("Starting ingestion from: {:?}", source);
            handle_ingest(source, model, chunk_size, overlap, recursive, config).await
        }
        Commands::Search {
            query,
            top_k,
            threshold,
            explain,
            format,
        } => {
            info!("Searching for: {}", query);
            handle_search(query, top_k, threshold, explain, format, config).await
        }
        Commands::Serve { port, host } => {
            info!("Starting web server on {}:{}", host, port);
            handle_serve(host, port, config).await
        }
        Commands::Stats => {
            info!("Displaying database statistics");
            handle_stats(config).await
        }
        Commands::Optimize => {
            info!("Optimizing database");
            handle_optimize(config).await
        }
        Commands::Models => {
            info!("Listing available Ollama models");
            handle_models(config).await
        }
    }
}

/// Handle the init command
async fn handle_init(force: bool, config: Config) -> Result<()> {
    let config_path = get_default_config_path()
        .ok_or_else(|| vectdb::VectDbError::Config("Could not determine config directory".to_string()))?;

    if config_path.exists() && !force {
        return Err(vectdb::VectDbError::Config(
            format!("Configuration file already exists at {:?}. Use --force to overwrite.", config_path)
        ));
    }

    config.save(&config_path)?;
    println!("Configuration initialized at: {:?}", config_path);
    println!("\nDefault configuration:");
    println!("{}", toml::to_string_pretty(&config).unwrap());

    Ok(())
}

/// Handle the ingest command (placeholder)
async fn handle_ingest(
    _source: std::path::PathBuf,
    _model: String,
    _chunk_size: usize,
    _overlap: usize,
    _recursive: bool,
    _config: Config,
) -> Result<()> {
    println!("Ingestion functionality will be implemented in Phase 4");
    println!("This will load documents, chunk them, generate embeddings, and store in the database.");
    Ok(())
}

/// Handle the search command (placeholder)
async fn handle_search(
    _query: String,
    _top_k: usize,
    _threshold: f32,
    _explain: bool,
    _format: String,
    _config: Config,
) -> Result<()> {
    println!("Search functionality will be implemented in Phase 5");
    println!("This will perform semantic search and return ranked results.");
    Ok(())
}

/// Handle the serve command (placeholder)
async fn handle_serve(_host: String, _port: u16, _config: Config) -> Result<()> {
    println!("Web server functionality will be implemented in Phase 6");
    println!("This will start an HTTP server with REST API and web UI.");
    Ok(())
}

/// Handle the stats command
async fn handle_stats(config: Config) -> Result<()> {
    use vectdb::VectorStore;

    let store = VectorStore::new(&config.database.path)?;
    let stats = store.get_stats()?;

    println!("=== VectDB Statistics ===\n");
    println!("Database:");
    println!("  Path: {:?}", config.database.path);
    println!("  Size: {} KB ({} bytes)", stats.db_size_bytes / 1024, stats.db_size_bytes);
    println!();
    println!("Content:");
    println!("  Documents:  {}", stats.document_count);
    println!("  Chunks:     {}", stats.chunk_count);
    println!("  Embeddings: {}", stats.embedding_count);
    println!();

    if stats.document_count > 0 {
        let avg_chunks = stats.chunk_count as f64 / stats.document_count as f64;
        println!("Averages:");
        println!("  Chunks per document: {:.2}", avg_chunks);

        if stats.embedding_count > 0 {
            let coverage = (stats.embedding_count as f64 / stats.chunk_count as f64) * 100.0;
            println!("  Embedding coverage: {:.1}%", coverage);
        }
    }

    Ok(())
}

/// Handle the optimize command
async fn handle_optimize(config: Config) -> Result<()> {
    use vectdb::VectorStore;

    println!("Optimizing database...");

    let store = VectorStore::new(&config.database.path)?;

    println!("  Running VACUUM...");
    store.vacuum()?;

    println!("  Running ANALYZE...");
    store.analyze()?;

    println!("✓ Database optimization complete");

    Ok(())
}

/// Handle the models command (placeholder)
async fn handle_models(_config: Config) -> Result<()> {
    println!("Models listing functionality will be implemented in Phase 3");
    println!("This will query Ollama API and list available embedding models.");
    Ok(())
}
