#![warn(clippy::pedantic)]
#![allow(clippy::implicit_return)]

use crate::icon::{IconResolver, default_icon};
use env_logger::Builder;
use hyprland::dispatch::{Dispatch, DispatchType, WindowIdentifier};
use hyprland::prelude::HyprData;
use hyprland::shared::Address;
use log::error;
use std::env;
use std::io::Write;
use std::process::{Command, Stdio};
use sysinfo::{Pid, System};

mod icon;

fn main() -> anyhow::Result<()> {
    gtk4::init()?;
    Builder::new()
        .parse_filters(&env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned()))
        .init();

    let mut resolver = IconResolver::new();
    let clients = hyprland::data::Clients::get()?;

    let mut sys = System::new_all();
    sys.refresh_all();
    let wofi_windows: Vec<_> = clients
        .iter()
        .map(|c| {
            #[allow(clippy::cast_sign_loss)]
            let icon = get_process_name(&sys, c.pid as u32).map(|name| resolver.icon_path(&name));
            format!(
                "img:{}:text:{}\t{}: {}",
                icon.unwrap_or(default_icon()),
                c.address,
                c.workspace.name,
                c.title
            )
        })
        .collect();

    let mut wofi = Command::new("worf")
        .arg("--show=dmenu")
        .arg("--prompt=Focus Window")
        .arg("-i")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start wofi");
    if let Some(mut stdin) = wofi.stdin.take() {
        for option in wofi_windows {
            log::debug!("adding window {option}");
            if let Err(e) = stdin.write(format!("{option}\n").as_bytes()) {
                error!("failed to write data to wofi stdin{e}");
            }
        }
    } else {
        wofi.wait()?;
    }

    let output = wofi.wait_with_output()?;
    if !output.stdout.is_empty() {
        let output = String::from_utf8_lossy(&output.stdout);
        let x = output
            .split_once("text:")
            .and_then(|(_, v)| v.split_whitespace().next())
            .unwrap_or_default()
            .to_owned();
        let address = Address::new(x);
        let window_ident = WindowIdentifier::Address(address);

        Dispatch::call(DispatchType::FocusWindow(window_ident))?;
    }
    Ok(())
}

fn get_process_name(sys: &System, pid: u32) -> Option<String> {
    sys.process(Pid::from_u32(pid))
        .map(|x| x.name().to_string_lossy().into_owned())
}
