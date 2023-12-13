use std::process::Command;

fn main() {
    qemu_spawn();
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
	      "-netdev", "user,id=net0,hostfwd=tcp::2222-:22"]);

    if let Ok(child) = cmd.spawn() {
	println!("Spawned qemu with pid {}", child.id());
    } else {
	println!("Couldn't start qemu");
    }
}
