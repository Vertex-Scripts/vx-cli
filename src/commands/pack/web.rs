use std::path::{Path, PathBuf};

use anyhow::anyhow;

use crate::commands::pack::PackContext;

#[derive(Debug)]
enum PackageManger {
    Npm,
    Yarn,
    Pnpm,
    NotFound,
}

pub fn build_web_project(context: &PackContext) -> anyhow::Result<()> {
    let ui_page = match context.manifest.ui_page {
        Some(ref page) => {
            if !page.eq("web/dist/index.html") {
                return Err(anyhow!("As of now, only web/dist/index.html is supported"));
            }

            page.as_str()
        }
        None => return Err(anyhow!("No UI page found")),
    };

    let mut package_manager = detect_package_manager(context.root_path.join("web"))?;
    match package_manager {
        PackageManger::NotFound => {
            log::error!("No lockfile found in web project, using npm as default");
            package_manager = PackageManger::Npm;
        }
        _ => {
            log::info!("Found package manager: {:?}", package_manager);
        }
    }

    let package_manager_command = match package_manager {
        PackageManger::Npm => "npm",
        PackageManger::Yarn => "yarn",
        PackageManger::Pnpm => "pnpm",
        _ => return Err(anyhow!("No package manager found")),
    };

    log::info!("Installing web project dependencies...");
    run_command(
        package_manager_command,
        &["install"],
        &context.root_path.join("web"),
    )?;

    log::info!("Building web project...");
    run_command(
        package_manager_command,
        &["build"],
        &context.root_path.join("web"),
    )?;

    Ok(())
}

fn run_command(command: &str, args: &[&str], cwd: &Path) -> anyhow::Result<()> {
    let output = std::process::Command::new("cmd")
        .arg("/C")
        .arg(command)
        .args(args)
        .current_dir(cwd)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!(
            "Command failed with exit code {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

fn detect_package_manager(path: PathBuf) -> anyhow::Result<PackageManger> {
    return if Path::new(&path).join("package-lock.json").exists() {
        Ok(PackageManger::Npm)
    } else if Path::new(&path).join("yarn.lock").exists() {
        Ok(PackageManger::Yarn)
    } else if Path::new(&path).join("pnpm-lock.yaml").exists() {
        Ok(PackageManger::Pnpm)
    } else {
        Ok(PackageManger::NotFound)
    };
}
