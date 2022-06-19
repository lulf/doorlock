use embassy::channel::DynamicReceiver;
use embassy::time::{block_for, Duration, Instant, Timer};
use embassy_nrf::config::Config;
use embassy_nrf::gpio::{AnyPin, Level, Output, OutputDrive, Pin};
use embassy_nrf::interrupt::Priority;
use embassy_nrf::peripherals::PWM0;
use embassy_nrf::pwm::{Prescaler, SimplePwm};
use embassy_nrf::Peripherals;

pub enum MotorCommand {
    Forward(Speed),
    Stop,
    Reverse(Speed),
}

impl MotorCommand {
    pub fn new(value: i8) -> MotorCommand {
        if value == 0 {
            MotorCommand::Stop
        } else if value > 0 {
            MotorCommand::Forward(Speed::new(value))
        } else {
            MotorCommand::Reverse(Speed::new(value))
        }
    }
}

pub enum Speed {
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
}

impl Speed {
    pub fn new(value: i8) -> Speed {
        let value = (value as i16).abs();
        if value >= 110 {
            Speed::_6
        } else if value >= 88 {
            Speed::_5
        } else if value >= 66 {
            Speed::_4
        } else if value >= 44 {
            Speed::_3
        } else if value >= 22 {
            Speed::_2
        } else {
            Speed::_1
        }
    }

    fn duty(&self) -> u16 {
        match self {
            Self::_1 => 2500,
            Self::_2 => 2000,
            Self::_3 => 1500,
            Self::_4 => 1000,
            Self::_5 => 500,
            Self::_6 => 0,
        }
    }
}

pub struct Motor {
    ain1: Output<'static, AnyPin>,
    ain2: Output<'static, AnyPin>,
    bin1: Output<'static, AnyPin>,
    bin2: Output<'static, AnyPin>,
    //pwm: SimplePwm<'static, PWM0>,
    standby: Output<'static, AnyPin>,
}

impl Motor {
    pub fn new(
        ain1: Output<'static, AnyPin>,
        ain2: Output<'static, AnyPin>,
        bin1: Output<'static, AnyPin>,
        bin2: Output<'static, AnyPin>,
        //pwm: SimplePwm<'static, PWM0>,
        standby: Output<'static, AnyPin>,
    ) -> Self {
        Self {
            ain1,
            ain2,
            bin1,
            bin2,
            //pwm,
            standby,
        }
    }

    pub fn enable(&mut self) {
        //self.pwm.set_prescaler(Prescaler::Div128);
        //self.pwm.set_max_duty(32767);
        //self.pwm.set_duty(0, 32767);
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

    pub fn forward(&mut self) {
        //defmt::info!("Forward speed is {}", s);
        const DELAY: u64 = 1;
        self.ain1.set_high();
        self.ain2.set_low();
        self.bin1.set_high();
        self.bin2.set_low();
        block_for(Duration::from_millis(DELAY));

        self.ain1.set_low();
        self.ain2.set_high();
        self.bin1.set_high();
        self.bin2.set_low();
        block_for(Duration::from_millis(DELAY));

        self.ain1.set_low();
        self.ain2.set_high();
        self.bin1.set_low();
        self.bin2.set_high();
        block_for(Duration::from_millis(DELAY));

        self.ain1.set_high();
        self.ain2.set_low();
        self.bin1.set_low();
        self.bin2.set_high();
        block_for(Duration::from_millis(DELAY));
    }

    pub fn reverse(&mut self, speed: Speed) {
        let s = speed.duty();
        defmt::info!("Reverse speed is {}", s);
        //self.pwm.set_duty(0, s);
        /*self.ain1.set_high();
        self.ain2.set_low();
        self.bin1.set_high();
        self.bin2.set_low();*/
    }
}

#[embassy::task]
pub async fn motor_task(mut motor: Motor, commands: DynamicReceiver<'static, MotorCommand>) {
    loop {
        defmt::info!("Cycle start");
        motor.enable();
        for _i in 1..100 {
            motor.forward();
        }
        motor.disable();
        Timer::after(Duration::from_secs(4)).await;
        /*
        motor.ain1.set_high();
        motor.ain2.set_low();
        motor.bin1.set_high();
        motor.bin2.set_low();

        motor.ain1.set_low();
        motor.ain2.set_low();
        motor.bin1.set_low();
        motor.bin2.set_low();

        motor.ain1.set_low();
        motor.ain2.set_high();
        block_for(Duration::from_micros(100000));
        motor.bin1.set_low();
        motor.bin2.set_high();
        block_for(Duration::from_micros(1000000));

        motor.ain1.set_low();
        motor.ain2.set_low();
        motor.bin1.set_low();
        motor.bin2.set_low();
        let c = commands.recv().await;
        match c {
            MotorCommand::Forward(speed) => {
                motor.enable();
                motor.forward(speed);
            }
            MotorCommand::Stop => {
                motor.disable();
            }
            MotorCommand::Reverse(speed) => {
                motor.enable();
                motor.reverse(speed);
            }
        }*/
    }
}
