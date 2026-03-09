use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

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

    // 1. Compile Kotlin
    let status = Command::new("kotlinc")
        .args(&["*.kt", "-d", "out/", "-Xuse-k2"])
        .current_dir(project_path)
        .status();

    match status {
        Ok(s) if s.success() => (),
        _ => return Err(anyhow::anyhow!("kotlinc failed or not found")),
    }

    // 2. DEX with D8
    let d8_status = Command::new("d8")
        .args(&[
            "out/*.class",
            "--output",
            "out/classes.dex",
            "--min-api",
            "26",
        ])
        .current_dir(project_path)
        .status();

    match d8_status {
        Ok(s) if s.success() => {
            println!("{}", format!("✅ Compiled in {}ms", start.elapsed().as_millis()).green());
            push_to_app(project_path).await?;
        }
        _ => {
            println!("{}", "⚠️ Warning: 'd8' failed. Hot-reload might not work.".yellow());
            println!("   Check 'vibe doctor' for d8 instructions.");
        }
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
        }
        Ok(_) => {
            println!("{}", "❌ App returned an error.".red());
        }
        Err(_) => {
            println!("{}", "❌ Error: Could not connect to VibeView App on localhost:8888".red());
            println!("   Ensure the VibeView app is OPEN on your phone.");
        }
    }

    Ok(())
}

fn run_doctor() -> Result<()> {
    println!("{}", "🩺 VibeView Doctor".bold().cyan());

    check_tool("kotlinc", "pkg install kotlin", &["-version"]);
    check_tool("cargo", "pkg install rust", &["--version"]);
    check_tool("d8", "Install with: pkg install build-tools (from its-pointless repo)", &["--version"]);

    println!("\n{}", "Check complete!".cyan());
    Ok(())
}

fn check_tool(name: &str, install_msg: &str, test_args: &[&str]) {
    // Try to run the tool directly. If it exists in PATH, it will start.
    let status = Command::new(name)
        .args(test_args)
        .output();

    match status {
        Ok(_) => {
            println!("  {} {} is installed", "✓".green(), name);
        }
        Err(_) => {
            println!("  {} {} is NOT installed. Action: {}", "✗".red(), name, install_msg);
        }
    }
}
