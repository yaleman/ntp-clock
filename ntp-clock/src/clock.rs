use crate::packets::NtpResponse;

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
            hour: normalize(self.hour, 360.0),
            minute: normalize(self.minute, 360.0),
            second: normalize(self.second, 360.0),
        }
    }

    pub fn normalize_radians(self) -> HandAngles {
        HandAngles {
            hour: normalize(self.hour, core::f64::consts::TAU),
            minute: normalize(self.minute, core::f64::consts::TAU),
            second: normalize(self.second, core::f64::consts::TAU),
        }
    }
}

fn normalize(value: f64, modulo: f64) -> f64 {
    let mut wrapped = value % modulo;
    if wrapped < 0.0 {
        wrapped += modulo;
    }
    wrapped
}

pub fn hand_angles(ntp_response: &NtpResponse) -> HandAngles {
    let total_seconds = ntp_response.ref_time / 1_000_000_000;
    let nanos = (ntp_response.ref_time % 1_000_000_000) as f64;

    let hour = (total_seconds / 3600) % 12;
    let minute = (total_seconds / 60) % 60;
    let second = total_seconds % 60;
    let seconds = second as f64 + (nanos / 1_000_000_000.0);
    let minutes = minute as f64 + (seconds / 60.0);
    let hours = hour as f64 + (minutes / 60.0);

    HandAngles {
        hour: hours * 30.0,
        minute: minutes * 6.0,
        second: seconds * 6.0,
    }
    .normalize_degrees()
}

pub fn hand_angles_radians(ntp_response: &NtpResponse) -> HandAngles {
    hand_angles(ntp_response).to_radians().normalize_radians()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_angles_known_time() {
        let ntp_response = NtpResponse::from_nanos(1_735_701_300_000_000_000u64);
        let angles = hand_angles(&ntp_response);
        assert!((angles.hour - 97.5).abs() < 1e-9);
        assert!((angles.minute - 90.0).abs() < 1e-9);
        assert!((angles.second - 0.0).abs() < 1e-9);
    }
}
