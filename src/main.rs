use std::process::Command;
use std::collections::HashMap;
use std::thread;
use std::str;
use std::fs::{File, copy};
use std::fs;
use std::env;
use axum::{
    extract::{Path, Extension},
    routing::{post, get},
    Router,
};
use tower_http::trace::TraceLayer;
use tracing_subscriber;
use std::sync::{Arc, RwLock};
use rand::{thread_rng, Rng};

struct VMState {
    vm_map: HashMap<String, String>,
}

fn router() -> Router {
    let shared_state : Arc<RwLock<VMState>> = Arc::new(RwLock::new(VMState {
	vm_map: HashMap::new(),
    }));
    Router::new()
	.route("/spawn_qemu/:name",
	       post(spawn_qemu_request))
	.route("/:name/user-data",
	       get(user_data_request))
	.route("/:name/meta-data",
	       get(meta_data_request))
	.layer(TraceLayer::new_for_http())	
	.layer(Extension(shared_state))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
	.with_max_level(tracing::Level::DEBUG)
	.init();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Cloud VM Simulator started");
    axum::serve(listener, router()).await.unwrap();
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

async fn meta_data_request(
    Path(name): Path<String>
) -> String {
    format!("instance-id: {}", name)
}

fn qemu_spawn(name: String) {
    let _ = fs::create_dir_all("/tmp/qemu-logs");
    let _ = fs::create_dir_all("/tmp/qemu-disks");
    let log_name = format!("/tmp/qemu-logs/{}.log", name.clone());
    let log = File::create(log_name.clone()).expect("failed to open log");
    let err_log_name = format!("/tmp/qemu-logs/{}-stderr.log", name.clone());
    let err_log = File::create(err_log_name.clone()).expect("failed to open log");
    let vm_image = env::var("CLOUD_VM_IMG_PATH")
	.unwrap_or("./bionic-server-cloudimg-amd64.img".to_string());
    let append = env::var("KERNEL_APPEND");
    let vmlinuz = env::var("KERNEL_VMLINUZ_PATH");
    println!("Running image {}", vm_image.clone());
    let nocloud_addr = format!("http://10.0.2.2:3000/{}/", name);
    let cmdline_addr = format!("http://10.0.2.2:3000/{}/user-data", name);
    let ssh_port = thread_rng().gen_range(32768..60999);
    println!("Using ssh port {}", ssh_port.clone());
    let qemu_disk = format!("/tmp/qemu-disks/{}-{}.qcow", name.clone(), ssh_port.clone());
    println!("Copying disk {} to {}", vm_image.clone(), qemu_disk.clone());
    copy(vm_image.clone(), qemu_disk.clone()).expect("Failed to copy disk");
    let expose = env::var("EXPOSE_PORT");
    let expose_random = thread_rng().gen_range(32768..60999);

    let mut cmd = if cfg!(target_arch = "aarch64") {
	Command::new("qemu-system-aarch64")
    } else {
	Command::new("qemu-system-x86_64")
    };

    if cfg!(target_arch = "aarch64") {
	cmd.arg("-M");
	cmd.arg("virt,accel=hvf");
	cmd.arg("-pflash");
	cmd.arg("flash0.img");
	cmd.arg("-pflash");
	cmd.arg("flash1.img");
    } else {
	cmd.arg("-M");
	cmd.arg("accel=hvf");
    }

    if append.is_ok() {
	println!("Running image {} with append {} and kernel {}", qemu_disk.clone(), append.clone().unwrap(), vmlinuz.clone().unwrap());
	cmd.arg("-append");
	cmd.arg(format!("{} cloud-config-url={}", append.unwrap(), cmdline_addr));
	if vmlinuz.is_ok() {
	    cmd.arg("-kernel");
	    cmd.arg(vmlinuz.unwrap());
	}
    }

    cmd.args(["-m", "2G",
	      "-cpu", "host",
	      "-serial", "stdio",
	      "-display", "none",
	      "-device", "virtio-scsi-pci,id=scsi",
	      "-device", "e1000,netdev=net0",
	      "-hda", &qemu_disk,
	      "-smbios", format!("type=1,serial=ds=nocloud-net;s={}", nocloud_addr).as_str()]);

    if expose.is_ok() {
	println!("Will expose port {} as {}", expose.clone().unwrap(), expose_random.clone());
	cmd.arg("-netdev");
	cmd.arg(format!("user,id=net0,hostfwd=tcp::{}-:{},hostfwd=tcp::{}-:22", expose_random.clone(), expose.clone().unwrap(), ssh_port.clone()).as_str());
    } else {
	cmd.arg("-netdev");
	cmd.arg(format!("user,id=net0,hostfwd=tcp::{}-:22", ssh_port).as_str());
    }

    cmd.stdout(log);
    cmd.stderr(err_log);

    if let Ok(child) = cmd.spawn() {
	println!("Spawned qemu - logging to {}", log_name);
	child.wait_with_output().expect("Error with this qemu machine");
    } else {
	println!("Couldn't start qemu - error at {}", err_log_name);
    }
}
