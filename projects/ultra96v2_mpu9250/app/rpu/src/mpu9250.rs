
#![allow(dead_code)]

use super::*;


const MPU9250_ADDRESS: u8 =     0x68;    // 7bit address
const AK8963_ADDRESS: u8 =      0x0C;    // Address of magnetometer

pub struct Mpu9250<T: I2cAccess> {
    i2c: T,
}

pub struct Mpu9250SensorData {
    pub accel: [i16; 3],
    pub gyro: [i16; 3],
    pub temperature: i16,
}

impl<T: I2cAccess> Mpu9250<T> {
    pub fn new(i2c: T) -> Self {
        // 起動
        i2c.write(MPU9250_ADDRESS, &[0x6b, 0x00]);
        i2c.write(MPU9250_ADDRESS, &[0x37, 0x02]);

        Self { i2c }
    }

    pub fn read_who_am_i(&self) -> u8 {
        self.i2c.write(MPU9250_ADDRESS, &[0x75]);
        let mut who_am_i: [u8; 1] = [0u8; 1];
        self.i2c.read(MPU9250_ADDRESS, &mut who_am_i);
        who_am_i[0]
    }

    pub fn read_sensor_data(&self) -> Mpu9250SensorData {
        let mut buf = [0u8; 14];
        self.i2c.write(MPU9250_ADDRESS, &[0x3b]);
        self.i2c.read(MPU9250_ADDRESS, &mut buf);
    
        let accel0       = ((buf[ 0] as i16) << 8) | (buf[ 1] as i16);
        let accel1       = ((buf[ 2] as i16) << 8) | (buf[ 3] as i16);
        let accel2       = ((buf[ 4] as i16) << 8) | (buf[ 5] as i16);
        let temperature  = ((buf[ 6] as i16) << 8) | (buf[ 7] as i16);
        let gyro0        = ((buf[ 8] as i16) << 8) | (buf[ 9] as i16);
        let gyro1        = ((buf[10] as i16) << 8) | (buf[11] as i16);
        let gyro2        = ((buf[12] as i16) << 8) | (buf[13] as i16);

        Mpu9250SensorData {
            accel: [accel0, accel1, accel2],
            gyro: [gyro0, gyro1, gyro2],
            temperature: temperature,
        }
    }
}
