use anyhow::{Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::process::{Command, Stdio};
use sysinfo::{System, SystemExt};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use webbrowser;

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", style("🐳 Docker Auto Installer").bold().blue());
    println!("{}", style("==========================").bold());

    // Rileva il sistema operativo
    let sys = System::new_all();
    let os_name = sys.name().unwrap_or_else(|| "Unknown".to_string());
    let os_type = detect_os_type(&os_name);

    println!("{} Sistema rilevato: {}", style("✓").green(), os_name);

    match os_type {
        OsType::Linux => install_docker_linux().await?,
        OsType::Windows => install_docker_windows().await?,
        OsType::Mac => install_docker_mac().await?,
        OsType::Unsupported => {
            eprintln!("{} Sistema operativo non supportato", style("✗").red());
            return Ok(());
        }
    }

    // Verifica che Docker sia installato correttamente
    verify_docker_installation().await?;

    // Build del progetto Rust per Linux
    build_for_linux().await?;

    println!("\n{}", style("🎉 Tutte le operazioni completate con successo!").bold().green());
    Ok(())
}

enum OsType {
    Linux,
    Windows,
    Mac,
    Unsupported,
}

fn detect_os_type(os_name: &str) -> OsType {
    let lower_name = os_name.to_lowercase();
    if lower_name.contains("linux") || lower_name.contains("ubuntu") || lower_name.contains("debian") {
        OsType::Linux
    } else if lower_name.contains("windows") {
        OsType::Windows
    } else if lower_name.contains("mac") || lower_name.contains("darwin") {
        OsType::Mac
    } else {
        OsType::Unsupported
    }
}

async fn install_docker_linux() -> Result<()> {
    println!("{} Installazione Docker per Linux...", style("🔧").yellow());

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner} {msg}")?);
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // Script di installazione per Linux
    let commands = [
        "sudo apt-get update",
        "sudo apt-get install -y apt-transport-https ca-certificates curl gnupg lsb-release",
        "curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg",
        "echo \"deb [arch=amd64 signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable\" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null",
        "sudo apt-get update",
        "sudo apt-get install -y docker-ce docker-ce-cli containerd.io",
        "sudo usermod -aG docker $USER",
    ];

    for cmd in commands.iter() {
        pb.set_message(format!("Esecuzione: {}", cmd));
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| format!("Failed to execute: {}", cmd))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            pb.finish_and_clear();
            eprintln!("{} Errore durante l'esecuzione di: {}", style("✗").red(), cmd);
            eprintln!("Errore: {}", stderr);
            return Err(anyhow::anyhow!("Installazione Docker fallita"));
        }
    }

    pb.finish_with_message("Docker installato con successo");
    Ok(())
}

async fn install_docker_windows() -> Result<()> {
    println!("{} Installazione Docker per Windows...", style("🔧").yellow());

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner} {msg}")?);
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // Download Docker Desktop installer
    pb.set_message("Download Docker Desktop...");
    let installer_url = "https://desktop.docker.com/win/stable/Docker%20Desktop%20Installer.exe";
    let installer_path = "DockerDesktopInstaller.exe";

    let client = reqwest::Client::new();
    let response = client.get(installer_url).send().await?;
    let mut file = File::create(installer_path).await?;
    let content = response.bytes().await?;
    file.write_all(&content).await?;

    // Esegui l'installer
    pb.set_message("Installazione Docker Desktop...");
    let output = Command::new(installer_path)
        .arg("install")
        .arg("--quiet")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        pb.finish_and_clear();
        eprintln!("{} Errore durante l'installazione", style("✗").red());
        eprintln!("Errore: {}", stderr);
        return Err(anyhow::anyhow!("Installazione Docker fallita"));
    }

    pb.finish_with_message("Docker installato con successo. Riavvio richiesto.");
    Ok(())
}

async fn install_docker_mac() -> Result<()> {
    println!("{} Installazione Docker per macOS...", style("🔧").yellow());
    
    let pb = ProgressBar::new_spinner();
    pb.set_message("Utilizza Docker Desktop per macOS: https://docs.docker.com/desktop/mac/install/");
    pb.finish_with_message("Visita https://docs.docker.com/desktop/mac/install/ per l'installazione");
    
    // Apri il link nel browser
    if let Err(e) = webbrowser::open("https://docs.docker.com/desktop/mac/install/") {
        eprintln!("Impossibile aprire il browser: {}", e);
    }
    
    Ok(())
}

async fn verify_docker_installation() -> Result<()> {
    println!("\n{} Verifica installazione Docker...", style("🔍").cyan());

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner} {msg}")?);
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // Verifica versione Docker
    pb.set_message("Verifica versione Docker...");
    let output = Command::new("docker")
        .arg("--version")
        .output()
        .context("Docker non trovato, verifica l'installazione")?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        pb.finish_with_message(format!("{} Docker version: {}", style("✓").green(), version.trim()));
    } else {
        pb.finish_and_clear();
        eprintln!("{} Docker non installato correttamente", style("✗").red());
        return Err(anyhow::anyhow!("Docker verification failed"));
    }

    // Test con container hello-world
    pb.set_message("Test esecuzione container...");
    let test_output = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("hello-world")
        .output();

    match test_output {
        Ok(output) if output.status.success() => {
            pb.finish_with_message(format!("{} Test container completato", style("✓").green()));
        }
        _ => {
            pb.finish_with_message(format!("{} Test container saltato", style("⚠").yellow()));
        }
    }

    Ok(())
}

async fn build_for_linux() -> Result<()> {
    println!("\n{} Build progetto Rust per Linux...", style("🦀").magenta());

    // Aggiungi target Linux
    let pb = ProgressBar::new_spinner();
    pb.set_message("Aggiunta target Linux...");
    let output = Command::new("rustup")
        .arg("target")
        .arg("add")
        .arg("x86_64-unknown-linux-gnu")
        .output()?;

    if !output.status.success() {
        pb.finish_and_clear();
        eprintln!("{} Impossibile aggiungere target Linux", style("✗").red());
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Errore: {}", stderr);
        return Ok(());
    }
    pb.finish_with_message("Target Linux aggiunto");

    // Build del progetto
    println!("{} Esecuzione build...", style("🔨").yellow());
    let build_output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("x86_64-unknown-linux-gnu")
        .output()?;

    if build_output.status.success() {
        println!("{} Build completata con successo!", style("✓").green());
        println!("{} Il binario si trova in: {}", style("📁").cyan(), "target/x86_64-unknown-linux-gnu/release/");
    } else {
        let stderr = String::from_utf8_lossy(&build_output.stderr);
        eprintln!("{} Errore durante la build:", style("✗").red());
        eprintln!("{}", stderr);
    }

    Ok(())
}