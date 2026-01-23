#![no_std]

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HandAnglesDeg {
    pub hour: f32,
    pub minute: f32,
}

impl From<ntp_clock::clock::HandAngles> for HandAnglesDeg {
    fn from(value: ntp_clock::clock::HandAngles) -> Self {
        HandAnglesDeg {
            hour: value.hour as f32,
            minute: value.minute as f32,
        }
    }
}

impl HandAnglesDeg {
    pub fn normalized(self) -> Self {
        Self {
            hour: wrap_degrees(self.hour),
            minute: wrap_degrees(self.minute),
        }
    }
}

pub trait ServoController {
    type Error;

    fn set_hour_angle(&mut self, angle_deg: f32) -> Result<(), Self::Error>;
    fn set_minute_angle(&mut self, angle_deg: f32) -> Result<(), Self::Error>;
}

pub trait LimitSwitches {
    fn hour_triggered(&self) -> bool;
    fn minute_triggered(&self) -> bool;
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ZeroOffsets {
    pub hour: f32,
    pub minute: f32,
}

impl ZeroOffsets {
    pub fn apply(self, angles: HandAnglesDeg) -> HandAnglesDeg {
        HandAnglesDeg {
            hour: wrap_degrees(angles.hour + self.hour),
            minute: wrap_degrees(angles.minute + self.minute),
        }
    }
}

pub struct ClockMechanism<C, S> {
    controller: C,
    switches: S,
    offsets: ZeroOffsets,
    last_commanded: HandAnglesDeg,
}

pub mod hardware;

impl<C, S> ClockMechanism<C, S>
where
    C: ServoController,
    S: LimitSwitches,
{
    pub fn new(controller: C, switches: S) -> Self {
        Self {
            controller,
            switches,
            offsets: ZeroOffsets::default(),
            last_commanded: HandAnglesDeg::default(),
        }
    }

    pub fn apply_hand_angles(&mut self, angles: HandAnglesDeg) -> Result<(), C::Error> {
        let angles = angles.normalized();
        self.last_commanded = angles;
        let adjusted = self.offsets.apply(angles);
        self.controller.set_hour_angle(adjusted.hour)?;
        self.controller.set_minute_angle(adjusted.minute)?;
        Ok(())
    }

    pub fn update_zeroing(&mut self) {
        if self.switches.hour_triggered() {
            self.offsets.hour = -self.last_commanded.hour;
        }
        if self.switches.minute_triggered() {
            self.offsets.minute = -self.last_commanded.minute;
        }
        self.offsets.hour = wrap_degrees(self.offsets.hour);
        self.offsets.minute = wrap_degrees(self.offsets.minute);
    }

    pub fn offsets(&self) -> ZeroOffsets {
        self.offsets
    }

    pub fn into_parts(self) -> (C, S) {
        (self.controller, self.switches)
    }
}

fn wrap_degrees(angle: f32) -> f32 {
    let mut value = angle % 360.0;
    if value < 0.0 {
        value += 360.0;
    }
    value
}
