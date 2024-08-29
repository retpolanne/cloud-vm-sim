# Cloud VM Simulator

This app simulates a cloud VM such as EC2. It can receive some user-data for cloud-init and it will provision a 
QEMU VM. It can also monitor the VM state and other things. 

To specify a cloud image different than bionic, set this env var: 

```sh
export CLOUD_VM_IMG_PATH=$PWD/whatever.qcow2
```

If you want to expose ports from the guest

``` sh
export EXPOSE_PORTS=32001,32002,5005,2375
```

With this, can even expose the Docker daemon port (2375) to build images on the guest docker.

``` sh
DOCKER_HOST="localhost:2375" docker build . 
```

For append options, set these variables (make sure you have a vmlinuz file or a valid kernel as well):

```sh
export KERNEL_APPEND="root=/dev/sda"
export KERNEL_VMLINUZ_PATH="./vmlinuz"
```

Then run `cloud-vm-sim` or build it from source with `cargo build --release`.

It will start a server on port 3000. To launch an instance, you can:

``` sh
cat user-data.yml | curl -X POST localhost:3000/spawn_qemu/your-vm-name-here --data-binary @-
```

