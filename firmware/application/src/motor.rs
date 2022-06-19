use embassy::time::{block_for, Duration};
use embassy_nrf::gpio::{AnyPin, Output};

pub struct Motor {
    ain1: Output<'static, AnyPin>,
    ain2: Output<'static, AnyPin>,
    bin1: Output<'static, AnyPin>,
    bin2: Output<'static, AnyPin>,
    standby: Output<'static, AnyPin>,
    speed: u32,
}

impl Motor {
    pub fn new(
        ain1: Output<'static, AnyPin>,
        ain2: Output<'static, AnyPin>,
        bin1: Output<'static, AnyPin>,
        bin2: Output<'static, AnyPin>,
        standby: Output<'static, AnyPin>,
    ) -> Self {
        Self {
            ain1,
            ain2,
            bin1,
            bin2,
            standby,
            speed: 200,
        }
    }

    pub fn enable(&mut self) {
        self.ain1.set_low();
        self.ain2.set_low();
        self.bin1.set_low();
        self.bin2.set_low();
        self.standby.set_high();
    }

    pub fn disable(&mut self) {
        self.standby.set_low();
        self.ain1.set_low();
        self.ain2.set_low();
        self.bin1.set_low();
        self.bin2.set_low();
    }

    pub fn set_speed(&mut self, speed: u32) {
        self.speed = speed;
    }

    pub fn step(&mut self, steps: i16) {
        let delay: u64 = 60 * 1000 * 1000 / steps.abs() as u64 / self.speed as u64;
        if steps > 0 {
            for step in 0..steps {
                self.do_step(step as u16, delay);
            }
        } else if steps < 0 {
            for step in (0..steps.abs()).rev() {
                self.do_step(step as u16, delay);
            }
        }
    }

    fn do_step(&mut self, step: u16, delay: u64) {
        let step = step % 4;
        if step == 0 {
            self.ain1.set_high();
            self.ain2.set_low();
            self.bin1.set_high();
            self.bin2.set_low();
        } else if step == 1 {
            self.ain1.set_low();
            self.ain2.set_high();
            self.bin1.set_high();
            self.bin2.set_low();
        } else if step == 2 {
            self.ain1.set_low();
            self.ain2.set_high();
            self.bin1.set_low();
            self.bin2.set_high();
        } else if step == 3 {
            self.ain1.set_high();
            self.ain2.set_low();
            self.bin1.set_low();
            self.bin2.set_high();
        }
        block_for(Duration::from_micros(delay));
    }
}
