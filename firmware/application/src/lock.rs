use embassy::channel::DynamicReceiver;

use crate::motor::Motor;

pub struct Lock {
    motor: Motor,
    steps: u16,
}

impl Lock {
    pub fn new(motor: Motor) -> Self {
        Self { motor, steps: 2900 }
    }

    fn lock(&mut self) {
        self.motor.enable();
        self.motor.step(self.steps as i16);
        self.motor.disable();
    }

    fn unlock(&mut self) {
        self.motor.enable();
        self.motor.step(self.steps as i16 * -1);
        self.motor.disable();
    }

    fn set_steps(&mut self, steps: u16) {
        self.steps = steps % (i16::MAX as u16);
    }

    fn set_speed(&mut self, speed: u32) {
        self.motor.set_speed(speed);
    }
}

pub enum LockCommand {
    Lock,
    Unlock,
    SetSpeed(u32),
    SetSteps(u16),
}

#[embassy::task]
pub async fn lock_task(mut lock: Lock, commands: DynamicReceiver<'static, LockCommand>) {
    loop {
        match commands.recv().await {
            LockCommand::Lock => {
                lock.lock();
            }
            LockCommand::Unlock => {
                lock.unlock();
            }
            LockCommand::SetSpeed(speed) => {
                lock.set_speed(speed);
            }
            LockCommand::SetSteps(steps) => {
                lock.set_steps(steps);
            }
        }
    }
}
