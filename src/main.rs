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

/// Handle the ingest command
async fn handle_ingest(
    source: std::path::PathBuf,
    model: String,
    chunk_size: usize,
    overlap: usize,
    recursive: bool,
    config: Config,
) -> Result<()> {
    use vectdb::{IngestionService, OllamaClient, VectorStore};
    use vectdb::domain::ChunkStrategy;

    println!("Starting ingestion from: {:?}\n", source);

    // Initialize services
    let store = VectorStore::new(&config.database.path)?;
    let ollama = OllamaClient::new(config.ollama.base_url.clone(), config.ollama.timeout_seconds)?;

    // Check Ollama connection
    if !ollama.health_check().await? {
        println!("❌ Cannot connect to Ollama at {}", config.ollama.base_url);
        println!("\nMake sure Ollama is running:");
        println!("  ollama serve");
        return Ok(());
    }

    // Check if model exists
    if !ollama.has_model(&model).await? {
        println!("❌ Model '{}' not found in Ollama", model);
        println!("\nPull the model first:");
        println!("  ollama pull {}", model);
        return Ok(());
    }

    println!("✓ Connected to Ollama");
    println!("✓ Model '{}' available\n", model);

    let mut service = IngestionService::new(store, ollama);

    // Determine chunk strategy
    let strategy = ChunkStrategy::FixedSize {
        size: chunk_size,
        overlap,
    };

    // Collect files to ingest
    let files = collect_files(&source, recursive)?;

    if files.is_empty() {
        println!("No files found to ingest.");
        return Ok(());
    }

    println!("Found {} file(s) to process\n", files.len());

    // Process files
    let mut total_chunks = 0;
    let mut total_embeddings = 0;
    let mut skipped = 0;

    for (idx, file) in files.iter().enumerate() {
        println!("[{}/{}] Processing: {:?}", idx + 1, files.len(), file);

        match service.ingest_file(file, &model, strategy).await {
            Ok(result) => {
                if result.skipped {
                    println!("  ⊘ Skipped (duplicate or empty)");
                    skipped += 1;
                } else {
                    println!("  ✓ {} chunks, {} embeddings", result.chunks_created, result.embeddings_created);
                    total_chunks += result.chunks_created;
                    total_embeddings += result.embeddings_created;
                }
            }
            Err(e) => {
                println!("  ❌ Error: {}", e);
                skipped += 1;
            }
        }
        println!();
    }

    // Summary
    println!("=== Ingestion Complete ===");
    println!("Files processed: {}", files.len());
    println!("Files skipped:   {}", skipped);
    println!("Chunks created:  {}", total_chunks);
    println!("Embeddings:      {}", total_embeddings);

    Ok(())
}

/// Collect files to ingest
fn collect_files(source: &std::path::Path, recursive: bool) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();

    if source.is_file() {
        files.push(source.to_path_buf());
    } else if source.is_dir() {
        if recursive {
            for entry in walkdir::WalkDir::new(source)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if is_supported_file(path) {
                        files.push(path.to_path_buf());
                    }
                }
            }
        } else {
            for entry in std::fs::read_dir(source)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    let path = entry.path();
                    if is_supported_file(&path) {
                        files.push(path);
                    }
                }
            }
        }
    } else {
        return Err(vectdb::VectDbError::InvalidInput(format!(
            "Source is not a file or directory: {:?}",
            source
        )));
    }

    Ok(files)
}

/// Check if file is supported
fn is_supported_file(path: &std::path::Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "txt" | "md" | "markdown")
    } else {
        false
    }
}

/// Handle the search command
async fn handle_search(
    query: String,
    top_k: usize,
    threshold: f32,
    explain: bool,
    format: String,
    config: Config,
) -> Result<()> {
    use vectdb::{OllamaClient, SearchService, VectorStore};
    use vectdb::services::search::{format_results_csv, format_results_json, format_results_text};

    // Initialize services
    let store = VectorStore::new(&config.database.path)?;
    let ollama = OllamaClient::new(config.ollama.base_url.clone(), config.ollama.timeout_seconds)?;

    // Check Ollama connection
    if !ollama.health_check().await? {
        println!("❌ Cannot connect to Ollama at {}", config.ollama.base_url);
        println!("\nMake sure Ollama is running:");
        println!("  ollama serve");
        return Ok(());
    }

    // Check if we have any embeddings
    let stats = store.get_stats()?;
    if stats.embedding_count == 0 {
        println!("❌ No embeddings found in database");
        println!("\nIngest some documents first:");
        println!("  vectdb ingest <file-or-directory>");
        return Ok(());
    }

    let service = SearchService::new(store, ollama);

    // Perform search
    let model = &config.ollama.default_model;
    let results = service.search(&query, model, top_k, threshold).await?;

    // Format and display results
    let output = match format.as_str() {
        "json" => format_results_json(&results)?,
        "csv" => format_results_csv(&results),
        _ => format_results_text(&results, explain),
    };

    println!("{}", output);

    Ok(())
}

/// Handle the serve command
async fn handle_serve(host: String, port: u16, config: Config) -> Result<()> {
    println!("Starting VectDB web server...");
    println!("Web UI: http://{}:{}", host, port);
    println!("API:    http://{}:{}/api", host, port);
    println!("\nPress Ctrl+C to stop\n");

    vectdb::server::serve(host, port, config).await
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

/// Handle the models command
async fn handle_models(config: Config) -> Result<()> {
    use vectdb::OllamaClient;

    println!("Connecting to Ollama at {}...\n", config.ollama.base_url);

    let client = OllamaClient::new(
        config.ollama.base_url.clone(),
        config.ollama.timeout_seconds,
    )?;

    // Check if Ollama is available
    if !client.health_check().await? {
        println!("❌ Ollama service is not available at {}", config.ollama.base_url);
        println!("\nMake sure Ollama is running:");
        println!("  brew services start ollama");
        println!("  or");
        println!("  ollama serve");
        return Ok(());
    }

    println!("✓ Connected to Ollama\n");

    // List available models
    let models = client.list_models().await?;

    if models.is_empty() {
        println!("No models found. Pull a model first:");
        println!("  ollama pull nomic-embed-text");
        return Ok(());
    }

    println!("Available Models ({}):\n", models.len());

    for model in &models {
        let size_mb = model.size as f64 / (1024.0 * 1024.0);
        println!("  • {}", model.name);
        println!("    Size: {:.1} MB", size_mb);
        println!("    Modified: {}", model.modified_at);
        println!();
    }

    // Show recommended models
    let recommended = vec!["nomic-embed-text", "all-minilm", "mxbai-embed-large"];
    let has_recommended: Vec<_> = models
        .iter()
        .filter(|m| recommended.iter().any(|r| m.name.contains(r)))
        .collect();

    if has_recommended.is_empty() {
        println!("Recommended embedding models:");
        for rec in recommended {
            println!("  ollama pull {}", rec);
        }
    } else {
        println!("✓ Using recommended model: {}", config.ollama.default_model);
    }

    Ok(())
}
