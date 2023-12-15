# Cloud VM Simulator

This app simulates a cloud VM such as EC2. It can receive some user-data for cloud-init and it will provision a 
QEMU VM. It can also monitor the VM state and other things. 

To specify a cloud image different than bionic, set this env var: 

```sh
export CLOUD_VM_IMG_PATH=$PWD/whatever.qcow2
```

## TODO

- [x] Create function to start QEMU with the provided user-data
- [x] Serve user-data to QEMU
- [ ] If planning on running multiple VMs, copy the .img to tmp .img file
