#![allow(dead_code)]

use axhal::arch::TrapFrame;
use axhal::trap::{register_trap_handler, SYSCALL, PAGE_FAULT};
use axhal::paging::MappingFlags;
use axerrno::LinuxError;
use axhal::mem::VirtAddr;
use axtask::{TaskExtMut, TaskExtRef};

use crate::task::TaskExt;

const SYS_EXIT: usize = 93;

#[register_trap_handler(SYSCALL)]
fn handle_syscall(tf: &TrapFrame, syscall_num: usize) -> isize {
    ax_println!("handle_syscall ...");
    let ret = match syscall_num {
        SYS_EXIT => {
            ax_println!("[SYS_EXIT]: process is exiting ..");
            axtask::exit(tf.arg0() as _)
        },
        _ => {
            ax_println!("Unimplemented syscall: {}", syscall_num);
            -LinuxError::ENOSYS.code() as _
        }
    };
    ret
}

#[register_trap_handler(PAGE_FAULT)]
fn handle_page_fault(
    vaddr: VirtAddr,
    access_flags: MappingFlags,
    _is_user: bool,
) -> bool {
    let task = axtask::current();                  // ✅ 保存临时值
    let task_ref = task.as_task_ref();             // ✅ 现在引用是安全的
    let task_inner = task_ref.inner();             // ✅ OK
    let ext = task_inner.task_ext();                    // ✅ OK
    let mut aspace = ext.aspace.lock();            // ✅ OK

    // 分配物理页帧
    if aspace.handle_page_fault(vaddr, access_flags) {
        return true;
    }

    panic!(
        "Unhandled page fault @ {:#x} with access {:?}",
        vaddr, access_flags
    );
}
