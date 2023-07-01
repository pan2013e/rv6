use crate::task::{TASK_MANAGER, forkret};

pub fn schedule() {
    extern "C" {
        fn swtch(_old: usize, _new: usize);
    }

    println!("Scheduling");
    let mut tm = TASK_MANAGER.lock();
    let (old, new): (usize, usize) = tm.switch_task();
    drop(tm);
    forkret();
    // unsafe {
    //     swtch(old, new);
    // }
}