use crate::io;

struct Pic {
    offset: u8,
    command_port: u16,
    data_port: u16,
}

impl Pic {
    fn handles_interrupt(&self, id: u8) -> bool {
        self.offset <= id && id < self.offset + 8
    }
    
    unsafe fn end_of_interrupt(&mut self) {
        io::outb(self.command_port, 0x20);
    }
}

pub struct ChainedPics {
    pics: [Pic; 2],
}

impl ChainedPics {
    pub const unsafe fn new(off1: u8, off2: u8) -> ChainedPics {
        ChainedPics {
            pics: [
                Pic {
                    offset: off1,
                    command_port: 0x20,
                    data_port: 0x21,
                },
                Pic {
                    offset: off2,
                    command_port: 0xA0,
                    data_port: 0xA1,
                },
            ]
        }
    }

    pub unsafe fn initialize(&mut self) {
        let wait = || { io::outb(0x80, 0) };

        let saved_mask1 = io::inb(self.pics[0].data_port);
        let saved_mask2 = io::inb(self.pics[1].data_port);

        io::outb(self.pics[0].command_port, 0x11);
        wait();
        io::outb(self.pics[1].command_port, 0x11);
        wait();

        io::outb(self.pics[0].data_port, self.pics[0].offset);
        wait();
        io::outb(self.pics[1].data_port, self.pics[1].offset);
        wait();

        io::outb(self.pics[0].data_port, 4);
        wait();
        io::outb(self.pics[1].data_port, 2);
        wait();

        io::outb(self.pics[0].data_port, 0x01);
        wait();
        io::outb(self.pics[1].data_port, 0x01);
        wait();

        io::outb(self.pics[0].data_port, saved_mask1);
        io::outb(self.pics[1].data_port, saved_mask2);
    }

    pub fn handles_interrupt(&self, id: u8) -> bool {
        self.pics.iter().any(|p| p.handles_interrupt(id))
    }

    pub unsafe fn notify_end_of_interrupt(&mut self, id: u8) {
        if self.handles_interrupt(id) {
            if self.pics[1].handles_interrupt(id) {
                self.pics[1].end_of_interrupt();
            }
            self.pics[0].end_of_interrupt();
        }
    }
}

