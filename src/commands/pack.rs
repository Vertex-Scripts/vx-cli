use std::{env, fmt::format, fs, path::PathBuf};
use std::io::{Error, ErrorKind, Write};
use std::iter::Zip;
use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context};
use glob::Pattern;
use rlua::{FromLuaMulti, Lua, RluaCompat};

use crate::commands::pack::PackageManger::NotFound;

pub const INTERCEPTOR_STR: &'static str = include_str!("interceptor.lua");

#[derive(Debug)]
enum PackageManger {
    Npm,
    Yarn,
    Pnpm,
    NotFound,
}

#[derive(Debug, Clone)]
struct FxManifest {
    ignored_paths: Vec<String>,
    ui_page: Option<String>,
}

impl FxManifest {
    fn new() -> Self {
        Self {
            ignored_paths: vec![
                ".git/**".into(),
                ".vscode/**".into(),
                ".gitattributes".into(),
                "README".into(),
                "README.md".into(),
                "LICENSE".into(),
            ],
            ui_page: None,
        }
    }
}

struct PackContext {
    root_path: PathBuf,
    manifest: FxManifest,
}

impl PackContext {
    fn new(root_path: PathBuf, manifest: FxManifest) -> Self {
        Self {
            root_path,
            manifest,
        }
    }
}

pub fn handle_command(mut root_path: PathBuf) -> anyhow::Result<()> {
    let manifest = read_fxmanifest_file(&mut root_path)?;
    let context = PackContext::new(root_path, manifest);

    if let Some(ui_page) = &context.manifest.ui_page {
        log::info!("Found UI page: {}, looking for web project", ui_page);
        if cfg!(target_os = "windows") {
            build_web_project(&context)?;
        } else {
            return Err(anyhow!("Unfortunately, only Windows is supported for now"));
        }
    }

    create_archive(&context)?;
    Ok(())
}

fn read_fxmanifest_file(path: &mut PathBuf) -> anyhow::Result<FxManifest> {
    let final_path = path.join("fxmanifest.lua");
    log::info!("Reading manifest from {}", final_path.display());

    let content = fs::read_to_string(final_path)?;

    let lua = Lua::new();
    let manifest = Arc::new(Mutex::new(FxManifest::new()));
    let manifest_clone = manifest.clone();

    let interceptor_function = lua.create_function(move |_, args: (String, String)| {
        let key = args.0;
        let value = args.1;
        log::debug!("Intercepted: {} -> {}", key, value);

        let mut manifest = manifest_clone.lock().unwrap();
        match key.as_str() {
            "ui_page" => {
                manifest.ui_page = Some(value);
            }
            "vx_ignore" => {
                manifest.ignored_paths.push(value);
            }
            _ => {}
        }

        Ok(())
    })?;

    lua.globals().set("_INTERCEPTOR", interceptor_function)?;
    lua.load(format!("{INTERCEPTOR_STR}\n{content}")).exec()?;

    let result = manifest.lock().unwrap();
    Ok(result.clone())
}

fn build_web_project(context: &PackContext) -> anyhow::Result<()> {
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
        NotFound => {
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

    // log::info!("Building web project...");
    // run_command(
    //     package_manager_command,
    //     &["build"],
    //     &context.root_path.join("web"),
    // )?;

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
        Ok(NotFound)
    };
}

fn create_archive(context: &PackContext) -> anyhow::Result<()> {
    let archive_name = context
        .root_path
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap();

    let archive_path = context.root_path.join(format!("{}.zip", archive_name));
    let file = fs::File::create(&archive_path)?;
    let mut zip = zip::ZipWriter::new(file);

    // loop through all the files in the directory with its subdirectories
    'walker: for entry in walkdir::WalkDir::new(&context.root_path) {
        let entry = entry?;
        let path = entry.path();

        if path.eq(&archive_path) {
            continue;
        }

        for ignored_path in &context.manifest.ignored_paths {
            let pattern = Pattern::new(&format!("**/{ignored_path}"))?;
            if pattern.matches_path(path) && !path.to_str().unwrap().contains("web\\dist") {
                log::debug!("Ignoring path: {}", path.display());
                continue 'walker;
            }
        }

        if path.is_file() {
            let mut relative_path = path.strip_prefix(&context.root_path).unwrap().to_path_buf();
            log::debug!("Adding file: {}", path.display());

            zip.start_file(
                relative_path.to_str().unwrap(),
                zip::write::FileOptions::default(),
            )?;

            let mut file = fs::File::open(path)?;
            std::io::copy(&mut file, &mut zip)?;
        }
    }

    zip.finish()?;
    Ok(())
}
