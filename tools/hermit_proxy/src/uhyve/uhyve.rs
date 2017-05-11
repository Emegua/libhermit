//! This file contains the entry point to the Unikernel Hypervisor. The uhyve utilizes KVM to
//! create a Virtual Machine and load the kernel.

use std::ptr;
use std::fs::{File,OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use libc;

use uhyve::{Error, Result, NameIOCTL};
use uhyve::vm::VirtualMachine;

/// The normal way of defining a IOCTL interface is provided by C macros. In Rust we have our own
/// flawed macro system. The module below wraps a bunch of functions which are generated by the
/// ioctl! macro and need to be wrapped further to provide a safe interface.
pub mod ioctl {
    use std::mem;
    use uhyve::kvm_header::{KVMIO, kvm_msr_list, kvm_cpuid2_header, kvm_memory_region, kvm_dirty_log, kvm_memory_alias, kvm_userspace_memory_region, kvm_regs,kvm_sregs};
    
    ioctl!(get_version with io!(KVMIO,  0x00));
    ioctl!(create_vm with io!(KVMIO, 0x01));
    ioctl!(read get_msr_index_list with KVMIO, 0x02; kvm_msr_list);
    ioctl!(check_extension with io!(KVMIO, 0x03));
    ioctl!(get_vcpu_mmap_size with io!(KVMIO, 0x04));
    
    ioctl!(readwrite get_supported_cpuid with KVMIO, 0x05; kvm_cpuid2_header);
    ioctl!(read get_emulated_cpuid with KVMIO,0x09; kvm_cpuid2_header);
    ioctl!(write set_cpuid2 with KVMIO, 0x90; kvm_cpuid2_header);

    ioctl!(create_vcpu with io!(KVMIO, 0x41));
    ioctl!(read get_dirty_log with KVMIO, 0x42;  kvm_dirty_log);
    ioctl!(write set_memory_alias with KVMIO, 0x43; kvm_memory_alias);
    ioctl!(set_nr_mmu_pages with io!(KVMIO, 0x44));
    ioctl!(get_nr_mmu_pages with io!(KVMIO, 0x45));
    
    ioctl!(write set_memory_region with KVMIO, 0x40; kvm_memory_region);
    ioctl!(write set_user_memory_region with KVMIO, 0x46; kvm_userspace_memory_region);

    ioctl!(create_irqchip with io!(KVMIO, 0x60));

    ioctl!(run with io!(KVMIO, 0x80));
    ioctl!(read get_regs with KVMIO, 0x81; kvm_regs);
    ioctl!(write set_regs with KVMIO, 0x82; kvm_regs);
    ioctl!(read get_sregs with KVMIO, 0x83; kvm_sregs);
    ioctl!(write set_sregs with KVMIO, 0x84; kvm_sregs);

}

/// KVM is freezed at version 12, so all others are invalid
#[derive(Debug)]
pub enum Version{
    Version12,
    Unsupported
}

/// This is the entry point of our module, it connects to the KVM device and wraps the functions
/// which accept the global file descriptor.
pub struct Uhyve {
    file: File
}

impl Uhyve {
    // Connects to the KVM hypervisor, by opening the virtual device /dev/kvm
    pub fn new() -> Uhyve {
        
        let kvm_file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_CLOEXEC)
            .open("/dev/kvm").unwrap();
  
        debug!("UHYVE - The connection to KVM was established.");
        
        Uhyve { file: kvm_file }
    }

    // Acquires the KVM version to seperate ancient systems
    pub fn version(&self) -> Result<Version> {
        unsafe {
            match ioctl::get_version(self.file.as_raw_fd(), ptr::null_mut()) {
                Ok(12) => Ok(Version::Version12),
                Ok(_)  => Ok(Version::Unsupported),
                Err(_) => Err(Error::IOCTL(NameIOCTL::GetVersion))
            }
        }
    }

    // Creates a new virtual machine and forwards the new fd to an object
    pub fn create_vm(&self, size: usize) -> Result<VirtualMachine> {
        unsafe {
            match ioctl::create_vm(self.file.as_raw_fd(), ptr::null_mut()) {
                Ok(vm_fd) => VirtualMachine::new(self.file.as_raw_fd(), vm_fd, size),
                Err(_) => Err(Error::InternalError)
            }
        }


    }
}
