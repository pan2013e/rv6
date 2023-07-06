extern crate alloc;

use crate::context::{Context, TrapFrame};
use crate::sync::SpinLock;
use alloc::vec::Vec;
use core::arch::asm;

lazy_static! {
    pub static ref PROC_MANAGER: SpinLock<Processes> =
        SpinLock::new(Processes::new(), "ProcManagerLock");
}

pub struct Processes {
    pub procs: Vec<Process>,
    pub current_pid: usize,
}

#[no_mangle]
pub fn loop_print() -> ! {
    use config::syscall::*;
    loop {
        let pid: usize;
        unsafe {
            asm!(
                "ecall",
                out("a0") pid,
                in("a7") SYSCALL_GETPID,
            );
        }
        let string = alloc::format!("Hello from process {}\n", pid);
        unsafe {
            asm!(
                "ecall",
                inlateout("a0") 1 => _,
                in("a1") string.as_ptr(),
                in("a2") string.len(),
                in("a7") SYSCALL_WRITE,
            );
        }
        for _ in 0..10000 {
            for _ in 0..10000 {
                unsafe {
                    asm!("nop");
                }
            }
        }
        // exit
        unsafe {
            asm!(
                "ecall",
                inlateout("a0") 0 => _,
                in("a7") SYSCALL_EXIT,
            );
        }
    }
}

#[no_mangle]
pub fn forkret() -> ! {
    unsafe {
        riscv::register::sepc::write(loop_print as usize);
        asm!("sret")
    }
    unreachable!()
}

/// Spawn proc 0
/// - We don't need to & must not store any context here
/// as the current context belongs to the kernel main thread.
/// - The kernel thread should never involve in context switching.
pub fn init() -> ! {
    let mut pm = PROC_MANAGER.lock();
    pm.init();
    let sp = pm.procs[0].context.sp;
    let ra = pm.procs[0].context.ra;
    drop(pm);
    unsafe {
        asm!(
            "mv sp, {}",
            "mv ra, {}",
            "ret",
            in(reg) sp,
            in(reg) ra
        );
    }
    unreachable!()
}

impl Processes {
    pub const fn new() -> Self {
        Self {
            procs: Vec::new(),
            current_pid: 0,
        }
    }

    pub fn init(&mut self) {
        for _ in 0..5 {
            self.create_task();
        }
    }

    pub fn create_task(&mut self) -> &mut Process {
        let pid = self.procs.len();
        let proc = Process::new(pid);
        self.procs.push(proc);
        &mut self.procs[pid]
    }

    pub fn switch_task(&mut self) -> (usize, usize) {
        let current_pid = self.current_pid;
        let next_pid = (current_pid + 1) % self.procs.len();
        let ctx_new: usize;
        let ctx_old: usize;
        {
            let next_task = &mut self.procs[next_pid];
            next_task.set_state(ProcState::Running);
            ctx_new = &next_task.context as *const Context as usize;
        }
        {
            let current_task = &mut self.procs[current_pid];
            current_task.set_state(ProcState::Ready);
            ctx_old = &current_task.context as *const Context as usize;
        }
        self.current_pid = next_pid;
        (ctx_old, ctx_new)
    }
}

#[derive(Default)]
pub enum ProcState {
    Running,
    #[default]
    Ready,
    Blocked,
    Exited,
}

/// Process control block
/// Tasks run in user mode, but use kernel memory for now 
/// because we don't have virtual memory yet.
#[rustfmt::skip]
#[repr(align(4096))]
pub struct Process {
    /// process id
    pub pid:            usize,
    /// task state
    pub state:          ProcState,
    /// kernel stack
    pub kstack:         usize,
    pub context:        Context,
    pub trapframe:      TrapFrame,
}

impl Process {
    pub fn new(pid: usize) -> Self {
        let mut proc = Self {
            pid,
            state: ProcState::default(),
            kstack: 0,
            context: Context::default(),
            trapframe: TrapFrame::default(),
        };
        proc.kstack = &proc as *const Process as usize + 4096;
        proc.context.sp = proc.kstack;
        proc.context.ra = forkret as usize;
        proc
    }

    pub fn set_state(&mut self, state: ProcState) {
        self.state = state;
    }
}