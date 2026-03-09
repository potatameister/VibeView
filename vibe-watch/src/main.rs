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
    /// Initialize a new VibeView project
    Init { name: String },
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
        Commands::Init { name } => init_project(&name)?,
        Commands::Start { path } => start_watcher(&path).await?,
        Commands::Build { path } => {
            let path_buf = PathBuf::from(path);
            compile_and_push(&path_buf).await?;
        }
        Commands::Doctor => run_doctor()?,
    }

    Ok(())
}

fn init_project(name: &str) -> Result<()> {
    let root = PathBuf::from(name);
    if root.exists() {
        println!("{}", format!("Error: Directory '{}' already exists", name).red());
        return Ok(());
    }

    println!("{}", format!("🚀 Initializing project: {}", name).cyan());

    fs::create_dir_all(root.join("out"))?;

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
    println!("Next steps:");
    println!("  1. cd {}", name);
    println!("  2. vibe start");

    Ok(())
}

async fn start_watcher(path_str: &str) -> Result<()> {
    let path = PathBuf::from(path_str).canonicalize()?;
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
        .status()
        .context("Failed to execute kotlinc")?;

    if !status.success() {
        return Err(anyhow::anyhow!("kotlinc failed"));
    }

    // 2. DEX with D8
    // We try to find d8, for now assuming it's in path or warning
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
            println!("{}", "⚠️ Warning: 'd8' not found or failed. Skipping DEX step.".yellow());
            println!("Please ensure 'd8' is in your PATH for hot-reload to work.");
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
        Ok(response) => {
            println!("{}", format!("❌ App returned error: {}", response.status()).red());
        }
        Err(_) => {
            println!("{}", "❌ Error: Could not connect to VibeView App on localhost:8888".red());
            println!("   Make sure the VibeView app is open on your phone.");
        }
    }

    Ok(())
}

fn run_doctor() -> Result<()> {
    println!("{}", "🩺 VibeView Doctor".bold().cyan());

    check_tool("kotlinc", "pkg install kotlin");
    check_tool("d8", "Install Android Command Line Tools in Termux");
    check_tool("cargo", "pkg install rust");

    println!("\n{}", "Check complete!".cyan());
    Ok(())
}

fn check_tool(name: &str, install_msg: &str) {
    let status = Command::new("command")
        .args(&["-v", name])
        .output()
        .is_ok();

    if status {
        println!("  {} {} is installed", "✓".green(), name);
    } else {
        println!("  {} {} is NOT installed. Action: {}", "✗".red(), name, install_msg);
    }
}
