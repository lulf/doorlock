use embassy::channel::DynamicReceiver;

use crate::motor::Motor;

#[derive(PartialEq)]
pub enum State {
    Locked,
    Unlocked,
}

pub struct Lock {
    motor: Motor,
    state: State,
    steps: u8,
}

impl Lock {
    pub fn new(motor: Motor) -> Self {
        Self {
            motor,
            state: State::Unlocked,
            steps: 10,
        }
    }

    fn lock(&mut self) {
        if self.state == State::Unlocked {
            self.motor.enable();
            self.motor.step(self.steps as i8);
            self.motor.disable();
        }
    }

    fn unlock(&mut self) {
        if self.state == State::Locked {
            self.motor.enable();
            self.motor.step(self.steps as i8 * -1);
            self.motor.disable();
        }
    }

    fn set_steps(&mut self, steps: u8) {
        self.steps = steps % 127;
    }
    
    fn set_speed(&mut self, speed: u8) {
        self.motor.set_speed(speed);
    }
}

pub enum LockCommand {
    Lock,
    Unlock,
    SetSpeed(u8),
    SetSteps(u8),
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
