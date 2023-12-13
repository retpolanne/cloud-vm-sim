use std::process::Command;
use std::thread;
use axum::{
    routing::{post},
    http::StatusCode,
    Router,
};

#[tokio::main]
async fn main() {
    let app = Router::new()
	.route("/spawn_qemu", post(spawn_qemu_request));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn spawn_qemu_request() -> &'static str {
    thread::spawn(|| qemu_spawn());
    "QEMU was spawned :)" 
}

fn qemu_spawn() {
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

    if let Ok(mut child) = cmd.spawn() {
	println!("Spawned qemu");
	child.wait_with_output().expect("Linux");
	println!("Linux started!");
    } else {
	println!("Couldn't start qemu");
    }
}
