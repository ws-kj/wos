use crate::io;
use lazy_static::lazy_static;
use spin::Mutex;

#[derive(Default)]
pub struct Timer {
    pub ticks: usize,
}

lazy_static! {
    pub static ref TIMER: Mutex<Timer> = Mutex::new(Default::default());
}

pub fn init(freq: usize) {
    let divisor = 1193180 / freq;

    unsafe { io::outb(0x43, 0x36); }

    let l = (divisor & 0xFF) as u8;
    let h = ((divisor>>8) & 0xFF) as u8;

    unsafe {
        io::outb(0x40, l);
        io::outb(0x40, h);
    }
}

pub fn wait(ticks: usize) {
    let eticks: usize;
    unsafe { TIMER.force_unlock() }
    eticks = TIMER.lock().ticks + ticks;
    while TIMER.lock().ticks < eticks {} 
}

pub fn tick() {
    unsafe { TIMER.force_unlock() }
    TIMER.lock().ticks += 1;
}

