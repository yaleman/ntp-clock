use crate::{HandAnglesDeg, ServoController};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ServoPwmConfig {
    pub min_pulse_us: u32,
    pub max_pulse_us: u32,
    pub period_us: u32,
    pub max_angle_deg: f32,
}

impl ServoPwmConfig {
    pub fn sg90_50hz() -> Self {
        Self {
            min_pulse_us: 1_000,
            max_pulse_us: 2_000,
            period_us: 20_000,
            max_angle_deg: 180.0,
        }
    }

    pub fn duty_for_angle(&self, angle_deg: f32, max_duty: u32) -> u32 {
        let clamped = angle_deg.clamp(0.0, self.max_angle_deg);
        let pulse_us = self.min_pulse_us as f32
            + (self.max_pulse_us - self.min_pulse_us) as f32 * (clamped / self.max_angle_deg);
        let duty = (pulse_us / self.period_us as f32) * max_duty as f32;
        (duty + 0.5) as u32
    }
}

pub struct PwmServoController<H, M> {
    hour: H,
    minute: M,
    config: ServoPwmConfig,
    hour_max_duty: u32,
    minute_max_duty: u32,
}

impl<H, M, E> PwmServoController<H, M>
where
    H: FnMut(u32) -> Result<(), E>,
    M: FnMut(u32) -> Result<(), E>,
{
    pub fn new(
        hour: H,
        minute: M,
        config: ServoPwmConfig,
        hour_max_duty: u32,
        minute_max_duty: u32,
    ) -> Self {
        Self {
            hour,
            minute,
            config,
            hour_max_duty,
            minute_max_duty,
        }
    }
}

impl<H, M, E> ServoController for PwmServoController<H, M>
where
    H: FnMut(u32) -> Result<(), E>,
    M: FnMut(u32) -> Result<(), E>,
{
    type Error = E;

    fn set_hour_angle(&mut self, angle_deg: f32) -> Result<(), Self::Error> {
        let duty = self.config.duty_for_angle(angle_deg, self.hour_max_duty);
        (self.hour)(duty)
    }

    fn set_minute_angle(&mut self, angle_deg: f32) -> Result<(), Self::Error> {
        let duty = self.config.duty_for_angle(angle_deg, self.minute_max_duty);
        (self.minute)(duty)
    }
}

pub fn angles_to_hand_degrees(angles: ntp_clock::clock::HandAngles) -> HandAnglesDeg {
    angles.into()
}
