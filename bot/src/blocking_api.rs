use crate::{
    line_follower_robot::{
        devices::{DeviceOperation, device_operation_blocking, device_operation_immediate},
        diagnostics::{CsvColumn, write_file, write_line},
    },
    value_ext::DeviceValueExt,
};

/// Get the current values of all line sensors.
pub fn get_line_sensors() -> [u8; 16] {
    let l = device_operation_immediate(DeviceOperation::ReadLineLeft);
    let r = device_operation_immediate(DeviceOperation::ReadLineRight);
    (0..8)
        .into_iter()
        .map(|i| l.get_u8(i))
        .chain((0..8).into_iter().map(|i| r.get_u8(i)))
        .enumerate()
        .fold([0; 16], |mut acc, (i, val)| {
            acc[i] = val;
            acc[i + 8] = val;
            acc
        })
}

/// Get the current values of motor angles (returns left and right angles with 16 bits of precision).
pub fn get_motor_angles() -> (u16, u16) {
    let values = device_operation_immediate(DeviceOperation::ReadMotorAngles);
    let left = values.get_u16(0);
    let right = values.get_u16(1);
    (left, right)
}

/// Get the current values of the gyro (returns pitch, roll, and yaw speed values in deg/s).
pub fn read_gyro() -> (i16, i16, i16) {
    let values = device_operation_immediate(DeviceOperation::ReadGyro);
    let pitch = values.get_i16(0);
    let roll = values.get_i16(1);
    let yaw = values.get_i16(2);
    (pitch, roll, yaw)
}

/// Get the current absolute euler angles (returns pitch, roll, and yaw values in deg).
pub fn get_imu_fused_data() -> (i16, i16, i16) {
    let values = device_operation_immediate(DeviceOperation::ReadImuFusedData);
    let pitch = values.get_i16(0);
    let roll = values.get_i16(1);
    let yaw = values.get_i16(2);
    (pitch, roll, yaw)
}

/// Get the current time in microseconds.
pub fn get_time_us() -> u32 {
    device_operation_immediate(DeviceOperation::GetTime).get_u32(0)
}

/// Sleep for the given time in microseconds.
pub fn sleep_for(time_us: u32) {
    device_operation_blocking(DeviceOperation::SleepFor(time_us));
}

/// Sleep until the given time in microseconds.
pub fn sleep_until(time_us: u32) {
    device_operation_blocking(DeviceOperation::SleepUntil(time_us));
}

/// Check if the remote is enabled.
pub fn remote_enabled() -> bool {
    device_operation_immediate(DeviceOperation::GetEnabled).get_u8(0) != 0
}

/// Wait for the remote to be enabled.
pub fn wait_remote_enabled() {
    device_operation_blocking(DeviceOperation::WaitEnabled);
}

/// Wait for the remote to be disabled.
pub fn wait_remote_disabled() {
    device_operation_blocking(DeviceOperation::WaitDisabled);
}

/// Log a message to the console.
pub fn console_log(text: &str) {
    write_line(text);
}

/// Write data into a binary file
pub fn write_plain_file(name: &str, data: &[u8]) {
    write_file(name, data, None);
}

/// Write data into a CSV file with the given specification
pub fn write_csv_file(name: &str, data: &[u8], spec: &[CsvColumn]) {
    write_file(name, data, Some(spec));
}
