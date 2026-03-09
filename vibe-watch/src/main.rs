use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "vibe")]
#[command(about = "VibeView Master CLI - Hot Bytecode Injection for Termux", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a VibeView project (in a new or current folder)
    Init { 
        /// Optional name of the project folder
        name: Option<String> 
    },
    /// Start the hot-reload watcher
    Start {
        #[arg(default_value = ".")]
        path: String,
    },
    /// Perform a one-time build and push
    Build {
        #[arg(default_value = ".")]
        path: String,
    },
    /// Check if the environment is ready
    Doctor,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => init_project(name.as_deref())?,
        Commands::Start { path } => start_watcher(&path).await?,
        Commands::Build { path } => {
            let path_buf = PathBuf::from(path);
            compile_and_push(&path_buf).await?;
        }
        Commands::Doctor => run_doctor()?,
    }

    Ok(())
}

fn init_project(name: Option<&str>) -> Result<()> {
    let root = match name {
        Some(n) => {
            let path = PathBuf::from(n);
            if !path.exists() {
                fs::create_dir_all(&path)?;
            }
            path
        }
        None => PathBuf::from("."),
    };

    println!("{}", format!("🚀 Initializing VibeView project in: {:?}", root).cyan());

    if !root.join("out").exists() {
        fs::create_dir_all(root.join("out"))?;
    }

    let snippet_content = r#"package com.potatameister.vibeview

import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme

object VibeSnippet {
    @Composable
    fun getContent() {
        Column {
            Text(
                text = "VibeView is LIVE!",
                style = MaterialTheme.typography.headlineMedium
            )
            Text(
                text = "Edit VibeSnippet.kt and save to see changes.",
                style = MaterialTheme.typography.bodyLarge
            )
        }
    }
}
"#;

    fs::write(root.join("VibeSnippet.kt"), snippet_content)?;
    
    println!("{}", "✅ Project initialized successfully!".green());
    println!("Usage:");
    println!("  vibe start");

    Ok(())
}

async fn start_watcher(path_str: &str) -> Result<()> {
    let path = match PathBuf::from(path_str).canonicalize() {
        Ok(p) => p,
        Err(_) => {
            println!("{}", format!("Error: Path '{}' not found", path_str).red());
            return Ok(());
        }
    };
    
    println!("{}", format!("👀 Watching: {:?}", path).cyan());

    // INITIAL PUSH
    println!("{}", "🚀 Performing initial sync...".yellow());
    if let Err(e) = compile_and_push(&path).await {
        println!("{}", format!("❌ Initial sync failed: {}", e).red());
    }

    let (tx, mut rx) = mpsc::channel(1);
    let mut last_trigger = Instant::now() - Duration::from_secs(5);

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                if let notify::EventKind::Modify(_) = event.kind {
                    let _ = tx.blocking_send(());
                }
            }
        },
        Config::default(),
    )?;

    watcher.watch(&path, RecursiveMode::Recursive)?;

    while let Some(_) = rx.recv().await {
        if last_trigger.elapsed() > Duration::from_millis(1000) {
            println!("{}", "File change detected! ⚡".yellow());
            last_trigger = Instant::now();
            if let Err(e) = compile_and_push(&path).await {
                println!("{}", format!("❌ Build failed: {}", e).red());
            }
        }
    }

    Ok(())
}

async fn compile_and_push(project_path: &Path) -> Result<()> {
    let start = Instant::now();
    println!("{}", "🔨 Compiling...".blue());

    let out_dir = project_path.join("out");
    if !out_dir.exists() {
        fs::create_dir(&out_dir)?;
    }

    // 1. Manually find all .kt files because kotlinc doesn't support recursive wildcards natively
    let mut kt_files = Vec::new();
    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|s| s == "kt").unwrap_or(false))
    {
        kt_files.push(entry.path().to_string_lossy().into_owned());
    }

    if kt_files.is_empty() {
        return Err(anyhow::anyhow!("No .kt files found in project directory"));
    }

    // 2. Compile Kotlin
    let status = Command::new("kotlinc")
        .args(&["-d", "out/", "-Dkotlin.colors.enabled=false"])
        .args(&kt_files)
        .current_dir(project_path)
        .status();

    match status {
        Ok(s) if s.success() => (),
        _ => return Err(anyhow::anyhow!("kotlinc failed")),
    }

    // 3. DEX with D8 or DX
    let mut dex_cmd = Command::new("d8");
    dex_cmd.args(&["out/*.class", "--output", "out/classes.dex", "--min-api", "26"]);
    
    let mut status = dex_cmd.current_dir(project_path).status();

    if status.is_err() {
        status = Command::new("dx")
            .args(&["--dex", "--output=out/classes.dex", "out/*.class"])
            .current_dir(project_path)
            .status();
    }

    match status {
        Ok(s) if s.success() => {
            println!("{}", format!("✅ Compiled in {}ms", start.elapsed().as_millis()).green());
            push_to_app(project_path).await?;
        }
        _ => return Err(anyhow::anyhow!("DEXing failed")),
    }

    Ok(())
}

async fn push_to_app(project_path: &Path) -> Result<()> {
    let dex_path = project_path.join("out/classes.dex");
    if !dex_path.exists() {
        return Err(anyhow::anyhow!("classes.dex not found"));
    }

    let bytes = fs::read(dex_path)?;
    let client = reqwest::Client::new();
    
    println!("{}", "📡 Pushing to VibeView Shell...".blue());
    
    let res = client.post("http://127.0.0.1:8888/push")
        .body(bytes)
        .send()
        .await;

    match res {
        Ok(response) if response.status().is_success() => {
            println!("{}", "🚀 Successfully pushed to App!".green().bold());
            
            // AUTO-FOREGROUND
            let _ = Command::new("am")
                .args(&["start", "--user", "0", "-n", "com.potatameister.vibeview/.MainActivity"])
                .output();
        }
        _ => {
            println!("{}", "❌ Error: Could not connect to VibeView App on localhost:8888".red());
        }
    }

    Ok(())
}

fn run_doctor() -> Result<()> {
    println!("{}", "🩺 VibeView Doctor".bold().cyan());

    check_tool("kotlinc", "pkg install kotlin", &["-version"]);
    check_tool("cargo", "pkg install rust", &["--version"]);
    
    if !check_tool_silent("d8", &["--version"]) && !check_tool_silent("dx", &["--version"]) {
        println!("  {} DEX tool (d8/dx) is NOT installed. Action: pkg install dx", "✗".red());
    } else {
        println!("  {} DEX tool is installed", "✓".green());
    }

    println!("\n{}", "Check complete!".cyan());
    Ok(())
}

fn check_tool(name: &str, install_msg: &str, test_args: &[&str]) {
    if check_tool_silent(name, test_args) {
        println!("  {} {} is installed", "✓".green(), name);
    } else {
        println!("  {} {} is NOT installed. Action: {}", "✗".red(), name, install_msg);
    }
}

fn check_tool_silent(name: &str, test_args: &[&str]) -> bool {
    Command::new(name)
        .args(test_args)
        .output()
        .is_ok()
}
