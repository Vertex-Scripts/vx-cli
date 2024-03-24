use std::fs;
use std::path::PathBuf;

use anyhow::anyhow;
use glob::Pattern;

use crate::commands::pack::manifest::{read_fxmanifest_file, FxManifest};
use crate::commands::pack::web::build_web_project;

mod manifest;
mod web;

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
            let relative_path = path.strip_prefix(&context.root_path).unwrap().to_path_buf();
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
