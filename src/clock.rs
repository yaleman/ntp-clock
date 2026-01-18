use time::OffsetDateTime;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HandAngles {
    pub hour: f64,
    pub minute: f64,
    pub second: f64,
}

impl HandAngles {
    pub fn to_radians(self) -> HandAngles {
        HandAngles {
            hour: self.hour.to_radians(),
            minute: self.minute.to_radians(),
            second: self.second.to_radians(),
        }
    }

    pub fn normalize_degrees(self) -> HandAngles {
        HandAngles {
            hour: self.hour.rem_euclid(360.0),
            minute: self.minute.rem_euclid(360.0),
            second: self.second.rem_euclid(360.0),
        }
    }

    pub fn normalize_radians(self) -> HandAngles {
        HandAngles {
            hour: self.hour.rem_euclid(std::f64::consts::TAU),
            minute: self.minute.rem_euclid(std::f64::consts::TAU),
            second: self.second.rem_euclid(std::f64::consts::TAU),
        }
    }
}

pub fn hand_angles(time: OffsetDateTime) -> HandAngles {
    let hour = time.hour() % 12;
    let minute = time.minute();
    let second = time.second();
    let nanos = time.nanosecond();

    let seconds = second as f64 + (nanos as f64 / 1_000_000_000.0);
    let minutes = minute as f64 + (seconds / 60.0);
    let hours = hour as f64 + (minutes / 60.0);

    HandAngles {
        hour: hours * 30.0,
        minute: minutes * 6.0,
        second: seconds * 6.0,
    }
    .normalize_degrees()
}

pub fn hand_angles_radians(time: OffsetDateTime) -> HandAngles {
    hand_angles(time).to_radians().normalize_radians()
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn hand_angles_known_time() {
        let time = datetime!(2025-01-01 03:15:00 +0);
        let angles = hand_angles(time);
        assert!((angles.hour - 97.5).abs() < 1e-9);
        assert!((angles.minute - 90.0).abs() < 1e-9);
        assert!((angles.second - 0.0).abs() < 1e-9);
    }
}
