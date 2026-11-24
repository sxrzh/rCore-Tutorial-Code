//! Types related to task management & Functions for completely changing TCB
use super::TaskContext;
use super::{kstack_alloc, pid_alloc, KernelStack, PidHandle};
use crate::config::{TRAP_CONTEXT_BASE, PAGE_SIZE, INITIAL_PRIORITY};
use crate::mm::{MapPermission, MemorySet, PhysAddr, PhysPageNum, VirtAddr, KERNEL_SPACE};
use crate::sync::UPSafeCell;
use crate::trap::{trap_handler, TrapContext};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::cell::RefMut;
use core::cmp::min;

use crate::task::stride::*;
use crate::task::manager::TASK_MANAGER;

/// Task control block structure
///
/// Directly save the contents that will not change during running
pub struct TaskControlBlock {
    // Immutable
    /// Process identifier
    pub pid: PidHandle,

    /// Kernel stack corresponding to PID
    pub kernel_stack: KernelStack,

    /// Mutable
    inner: UPSafeCell<TaskControlBlockInner>,
}

impl TaskControlBlock {
    /// Get the mutable reference of the inner TCB
    pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }
    /// Get the address of app's page table
    pub fn get_user_token(&self) -> usize {
        let inner = self.inner_exclusive_access();
        inner.memory_set.token()
    }
    /// Add the stride by one step
    pub fn stride_step(&self) {
        self.inner_exclusive_access().stride_step()
    }
    /// Set priority
    pub fn set_priority(&self, prio: usize) -> bool {
        self.inner_exclusive_access().set_priority(prio)
    }
}

impl PartialEq for TaskControlBlock {
    fn eq(&self, other: &Self) -> bool {
        self.inner_exclusive_access().stride == other.inner_exclusive_access().stride
    }
}
impl Eq for TaskControlBlock {}
impl Ord for TaskControlBlock {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner_exclusive_access().stride.cmp(&other.inner_exclusive_access().stride)
    }
}
impl PartialOrd for TaskControlBlock {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct TaskControlBlockInner {
    /// The physical page number of the frame where the trap context is placed
    pub trap_cx_ppn: PhysPageNum,

    /// Application data can only appear in areas
    /// where the application address space is lower than base_size
    pub base_size: usize,

    /// Save task context
    pub task_cx: TaskContext,

    /// Maintain the execution status of the current process
    pub task_status: TaskStatus,

    /// Application address space
    pub memory_set: MemorySet,

    /// Parent process of the current process.
    /// Weak will not affect the reference count of the parent
    pub parent: Option<Weak<TaskControlBlock>>,

    /// A vector containing TCBs of all child processes of the current process
    pub children: Vec<Arc<TaskControlBlock>>,

    /// It is set when active exit or execution error occurs
    pub exit_code: i32,

    /// Heap bottom
    pub heap_bottom: usize,

    /// Program break
    pub program_brk: usize,

    /// Stride value for task scheduling
    pub stride: Stride,

    /// Pass value for task scheduling
    pub stride_pass: StridePass,
}

impl TaskControlBlockInner {
    /// get the trap context
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    /// get the user token
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }
    
    pub fn translate_address(&self, virt_addr: VirtAddr, is_user: bool, need_write: bool) -> Option<PhysAddr> {
        let ppn = &self.memory_set
                      .translate_permitted(virt_addr.floor(), is_user, need_write)?
                      .ppn();
        let phys_addr: PhysAddr = (ppn.0 * PAGE_SIZE + virt_addr.page_offset()).into();
        Some(phys_addr)
    }

    pub fn get_memory(&self, virt_addr: VirtAddr, is_user: bool) -> Option<u8> {
        let phys_addr: PhysAddr = self.translate_address(virt_addr, is_user, false)?;
        unsafe { Some(*(phys_addr.0 as *const u8)) }
    }
    
    pub fn set_memory(&self, virt_addr: VirtAddr, byte: u8, is_user: bool) -> Option<()> {
        let phys_addr: PhysAddr = self.translate_address(virt_addr, is_user, true)?;
        unsafe { *(phys_addr.0 as *mut u8) = byte; }
        Some(())
    }

    pub fn map(&mut self, start: VirtAddr, len: usize, permission: MapPermission) -> bool {
        self.memory_set.map(start, len, permission)
    }

    pub fn unmap(&mut self, start: VirtAddr, len: usize) -> bool {
        self.memory_set.unmap(start, len)
    }

    /// calculate stride_pass from priority
    pub fn calc_pass(prio: usize) -> StridePass {
        StridePass::from_prio(prio)
    }
    
    /// implement stride_step
    pub fn stride_step(&mut self) {
        self.stride.add(self.stride_pass)
    }

    /// implement set_priority
    pub fn set_priority(&mut self, prio: usize) -> bool {
        if prio >= 2 {
            self.stride_pass = Self::calc_pass(prio);
            true
        }
        else {
            false
        }
    }
}

impl TaskControlBlock {
    /// Create a new process with default stride
    fn new_with_default_stride(elf_data: &[u8], default_stride: Stride) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT_BASE).into())
            .unwrap()
            .ppn();
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = kstack_alloc();
        let kernel_stack_top = kernel_stack.get_top();
        // push a task context which goes to trap_return to the top of kernel stack
        let task_control_block = Self {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: user_sp,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                    heap_bottom: user_sp,
                    program_brk: user_sp,
                    stride: TASK_MANAGER.exclusive_access()
                                .min_stride()
                                .map(|s| Stride(min(s.0, default_stride.0)))
                                .unwrap_or(default_stride),
                    stride_pass: TaskControlBlockInner::calc_pass(INITIAL_PRIORITY),
                })
            },
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }

    /// Create a new process
    ///
    /// At present, it is only used for the creation of initproc and spawn
    pub fn new(elf_data: &[u8]) -> Self {
        Self::new_with_default_stride(elf_data, Stride::default())
    }

    /// Load a new elf to replace the original application address space and start execution
    pub fn exec(&self, elf_data: &[u8]) {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT_BASE).into())
            .unwrap()
            .ppn();

        // **** access current TCB exclusively
        let mut inner = self.inner_exclusive_access();
        // substitute memory_set
        inner.memory_set = memory_set;
        // update trap_cx ppn
        inner.trap_cx_ppn = trap_cx_ppn;
        // initialize base_size
        inner.base_size = user_sp;
        // initialize trap_cx
        let trap_cx = inner.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            self.kernel_stack.get_top(),
            trap_handler as usize,
        );
        // **** release inner automatically
    }

    /// parent process fork the child process
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        // ---- access parent PCB exclusively
        let mut parent_inner = self.inner_exclusive_access();
        // copy user space(include trap context)
        let memory_set = MemorySet::from_existed_user(&parent_inner.memory_set);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT_BASE).into())
            .unwrap()
            .ppn();
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = kstack_alloc();
        let kernel_stack_top = kernel_stack.get_top();
        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: parent_inner.base_size,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
                    heap_bottom: parent_inner.heap_bottom,
                    program_brk: parent_inner.program_brk,
                    stride: TASK_MANAGER.exclusive_access()
                                .min_stride()
                                .map(|s| Stride(min(s.0, parent_inner.stride.0)))
                                .unwrap_or(parent_inner.stride),
                    stride_pass: parent_inner.stride_pass,
                })
            },
        });
        // add child
        parent_inner.children.push(task_control_block.clone());
        // modify kernel_sp in trap_cx
        // **** access child PCB exclusively
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;
        // return
        task_control_block
        // **** release child PCB
        // ---- release parent PCB
    }

    /// spawn a child process
    pub fn spawn(self: &Arc<Self>, elf_data: &[u8]) -> Arc<Self> {
        let mut parent_inner = self.inner_exclusive_access();
        
        let new_block = Arc::new(Self::new_with_default_stride(elf_data, parent_inner.stride));
        let mut new_block_inner = new_block.inner_exclusive_access();
        new_block_inner.parent = Some(Arc::downgrade(self));

        parent_inner.children.push(new_block.clone());

        drop(new_block_inner);
        new_block
    }

    /// get pid of process
    pub fn getpid(&self) -> usize {
        self.pid.0
    }

    /// change the location of the program break. return None if failed.
    pub fn change_program_brk(&self, size: i32) -> Option<usize> {
        let mut inner = self.inner_exclusive_access();
        let heap_bottom = inner.heap_bottom;
        let old_break = inner.program_brk;
        let new_brk = inner.program_brk as isize + size as isize;
        if new_brk < heap_bottom as isize {
            return None;
        }
        let result = if size < 0 {
            inner
                .memory_set
                .shrink_to(VirtAddr(heap_bottom), VirtAddr(new_brk as usize))
        } else {
            inner
                .memory_set
                .append_to(VirtAddr(heap_bottom), VirtAddr(new_brk as usize))
        };
        if result {
            inner.program_brk = new_brk as usize;
            Some(old_break)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
/// task status: UnInit, Ready, Running, Exited
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Zombie,
}
