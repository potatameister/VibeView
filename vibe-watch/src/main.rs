use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tokio::process::Command;
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> notify::Result<()> {
    let path = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");

    println!("Watching: {}", path);

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    
    // We use a debounce-like mechanism to avoid multiple triggers for a single save
    let last_trigger = Arc::new(Mutex::new(std::time::Instant::now() - Duration::from_secs(5)));

    let mut watcher = RecommendedWatcher::new(move |res: Result<notify::Event, notify::Error>| {
        match res {
            Ok(event) => {
                if let notify::EventKind::Modify(_) = event.kind {
                    let _ = tx.blocking_send(());
                }
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }, Config::default())?;

    watcher.watch(Path::new(&path), RecursiveMode::Recursive)?;

    while let Some(_) = rx.recv().await {
        let mut last = last_trigger.lock().await;
        if last.elapsed() > Duration::from_millis(500) {
            println!("File change detected! Triggering compile...");
            *last = std::time::Instant::now();
            
            // Trigger the vibe-compile script (which we will create next)
            let status = Command::new("bash")
                .arg("-c")
                .arg("./vibe-compile.sh")
                .current_dir(&path)
                .status()
                .await;

            match status {
                Ok(s) => println!("Compile finished with status: {}", s),
                Err(e) => println!("Failed to trigger compile: {}", e),
            }
        }
    }

    Ok(())
}
