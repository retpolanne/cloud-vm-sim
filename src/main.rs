use std::process::Command;
use std::collections::HashMap;
use std::thread;
use std::str;
use std::fs::File;
use std::fs;
use axum::{
    extract::{Path, Extension},
    routing::{post, get},
    Router,
};
use std::sync::{Arc, RwLock};

struct VMState {
    vm_map: HashMap<String, String>,
}

#[tokio::main]
async fn main() {
    let shared_state : Arc<RwLock<VMState>> = Arc::new(RwLock::new(VMState {
	vm_map: HashMap::new(),
    }));
    let app = Router::new()
	.route("/spawn_qemu/:name",
	       post(spawn_qemu_request))
	.route("/:name/user-data",
	       get(user_data_request))
	.layer(Extension(shared_state));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn spawn_qemu_request(
    Extension(state) : Extension<Arc<RwLock<VMState>>>, Path(name): Path<String>, user_data: String
) -> &'static str {
    let user_data_clone = user_data.clone();
    state.write().unwrap().vm_map.insert(name.clone(), user_data_clone);
    thread::spawn(|| qemu_spawn(name));
    "QEMU was spawned :)" 
}

async fn user_data_request(
    Extension(state) : Extension<Arc<RwLock<VMState>>>, Path(name): Path<String>
) -> String {
    let vm_map = &state.read().unwrap().vm_map;

    if let Some(value) = vm_map.get(&name) {
        value.clone()
    } else {
	format!("Couldn't get map for {}", name)
    }
}

fn qemu_spawn(name: String) {
    let _ = fs::create_dir_all("/tmp/qemu-logs");
    let log_name = format!("/tmp/qemu-logs/{}.log", name.clone());
    let log = File::create(log_name).expect("failed to open log");
    let err_log_name = format!("/tmp/qemu-logs/{}-stderr.log", name.clone());
    let err_log = File::create(err_log_name).expect("failed to open log");
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
	      "--smbios", format!("type=1,serial=ds='nocloud-net;s=http://10.0.2.2:3000/{}'", name).as_str(),
	      "-netdev", "user,id=net0,hostfwd=tcp::2222-:22"]);

    cmd.stdout(log);
    cmd.stderr(err_log);

    if let Ok(child) = cmd.spawn() {
	println!("Spawned qemu");
	child.wait_with_output().expect("Linux");
	println!("Linux started!");
    } else {
	println!("Couldn't start qemu");
    }
}
