use std::process::Command;
use std::collections::HashMap;
use std::thread;
use std::str;
use axum::{
    extract::{Path},
    routing::{post, get},
    Router,
};

#[tokio::main]
async fn main() {
    let var = "abc";
    let app = Router::new()
	.route("/spawn_qemu/:name",
	       post(
		   move |path: Path<String>, body: String|
		   spawn_qemu_request(path, body)))
	.route("/:name/user-data",
	       get(user_data_request));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn spawn_qemu_request(Path(name): Path<String>, user_data: String) -> &'static str {
    println!("{}", user_data);
    thread::spawn(|| qemu_spawn(name));
    "QEMU was spawned :)" 
}

async fn user_data_request(Path(name): Path<String>) -> String {
    name
}

fn qemu_spawn(name: String) {
    println!("{}", name);
    let mut cmd = if cfg!(target_arch = "aarch64") {
	Command::new("qemu-system-aarch64")
    } else {
	Command::new("qemu-system-x86_64")
    };

    if cfg!(target_arch = "aarch64") {
	cmd.arg("-M");
	cmd.arg("virt,accel=hvf");
    } else {
	cmd.arg("-M");
	cmd.arg("accel=hvf");
    }

    cmd.args(["-m", "2G",
	      "-cpu", "host",
	      "-serial", "stdio",
	      "-display", "none",
	      "-device", "virtio-scsi-pci,id=scsi",
	      "-device", "e1000,netdev=net0",
	      "-hda", "./bionic-server-cloudimg-amd64.img",
	      "-netdev", "user,id=net0,hostfwd=tcp::2222-:22"]);

    if let Ok(child) = cmd.spawn() {
	println!("Spawned qemu");
	child.wait_with_output().expect("Linux");
	println!("Linux started!");
    } else {
	println!("Couldn't start qemu");
    }
}
