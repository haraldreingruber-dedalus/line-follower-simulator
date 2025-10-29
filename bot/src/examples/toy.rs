use crate::{
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
    device_operation_blocking(DeviceOperation::WaitEnabled);

    write_line("started");

    while device_operation_immediate(DeviceOperation::GetEnabled).get_bool(0) {
        let left_values = device_operation_immediate(DeviceOperation::ReadLineLeft);
        let right_values = device_operation_immediate(DeviceOperation::ReadLineRight);

        let left_v = left_values.get_u8(0);
        let right_v = right_values.get_u8(7);

        let left = left_v < LINE;
        let right = right_v < LINE;

        let vals: Vec<_> = (0..8)
            .into_iter()
            .map(|i| left_values.get_u8(i))
            .chain((0..8).into_iter().map(|i| right_values.get_u8(i)))
            .collect();

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
