use crate::{
    blocking_api::{get_line_sensors, remote_enabled, wait_remote_enabled},
    line_follower_robot::{
        devices::{
            DeviceOperation, device_operation_blocking, device_operation_immediate,
            set_motors_power,
        },
        diagnostics::write_line,
    },
    value_ext::DeviceValueExt,
};

const LINE: u8 = 80;
const PWM_MAX: i16 = 300;
const PWM_MIN: i16 = -300;
const MAX_TIME: u32 = 3_000_000;

pub fn toy_run() {
    wait_remote_enabled();

    write_line("started");

    while remote_enabled() {
        let vals = get_line_sensors();

        let left_v = vals[0];
        let right_v = vals[15];

        let left = left_v < LINE;
        let right = right_v < LINE;

        // write_line(&format!(
        //     " - val {} {} [{} {}] line {} {}",
        //     left_v, right_v, vals[0], vals[15], left, right
        // ));

        write_line(&format!("LINE {:?}", vals));

        let (pwm_l, pwm_r) = match (left, right) {
            (true, _) => (PWM_MIN, PWM_MAX),
            (_, true) => (PWM_MAX, PWM_MIN),
            _ => (PWM_MAX - 10, PWM_MAX + 10),
        };
        set_motors_power(pwm_l, pwm_r);

        device_operation_blocking(DeviceOperation::SleepFor(1000));
        if device_operation_immediate(DeviceOperation::GetTime).get_u32(0) > MAX_TIME {
            write_line("timeout");
            break;
        }
    }
}
