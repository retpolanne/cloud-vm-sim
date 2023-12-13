# Cloud VM Simulator

This app simulates a cloud VM such as EC2. It can receive some user-data for cloud-init and it will provision a 
QEMU VM. It can also monitor the VM state and other things. 

## TODO

- [ ] Create function to start QEMU with the provided user-data
- [ ] Serve user-data to QEMU
- [ ] If planning on running multiple VMs, copy the .img to tmp .img file
