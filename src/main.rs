
#![no_std]
#![no_main]

use panic_halt as _;

use stm32f4xx_hal as hal;

use cortex_m_rt::{entry};



use crate::hal::{delay::Delay,  pac, prelude::*};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use shared_bus::{BusManagerSimple};

use bme280;

use heapless::String;



#[entry]
fn main() -> ! {
    if let (Some(dp), Some(_cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        let syst = _cp.SYST;
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(168.mhz()).freeze();

        let gpioa = dp.GPIOA.split();
        let mut led = gpioa.pa6.into_push_pull_output();
        let _ = led.set_high(); // Turn off
        let delay = Delay::new(syst, clocks);

        let gpiob = dp.GPIOB.split();



        let scl = gpiob.pb6.into_alternate_af4_open_drain();
        let sda = gpiob.pb7.into_alternate_af4_open_drain();
        

        let i2c = hal::i2c::I2c::new(dp.I2C1, (scl, sda), 400.khz(), clocks);
        let bus = BusManagerSimple::new(i2c);

        let interface = I2CDisplayInterface::new(bus.acquire_i2c());
        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        display.init().unwrap();
        display.flush().unwrap();
        let mut sensor =  bme280::BME280::new_primary(bus.acquire_i2c(), delay);
        sensor.init().unwrap();

        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        loop
        {
            display.clear();
            let _measurements = sensor.measure().unwrap();
            
            let mut text: String<16> =String::from("temp: ");
            let temp_val: String<3> = String::from(_measurements.temperature as u8);
            
            text.push_str(temp_val.as_str()).unwrap();
            text.push_str(".").unwrap();
            let fraction = ((_measurements.temperature - ((_measurements.temperature as u8)as f32))*100f32) as u8;
            if fraction < 10 {
                text.push_str("0").unwrap();
            }
            let temp_val: String<3> = String::from(fraction);
            text.push_str(temp_val.as_str()).unwrap();
            text.push_str(" C").unwrap();
            
            // let text = format!("temp: {} C", _measurements.temperature);
            
            Text::with_baseline(text.as_str(), Point::zero(), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

            text.clear();
            text.push_str("press ").unwrap();
            let press_val: String<6> = String::from((_measurements.pressure / 100f32) as u32);
            text.push_str(press_val.as_str()).unwrap();
            text.push_str("hPa").unwrap();

            Text::with_baseline(text.as_str(), Point::new(0, 16), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

            text.clear();
            text.push_str("humid ").unwrap();
            let press_val: String<3> = String::from(_measurements.humidity  as u32);
            text.push_str(press_val.as_str()).unwrap();
            text.push_str("%").unwrap();

            Text::with_baseline(text.as_str(), Point::new(0, 32), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        

            display.flush().unwrap();

        }
    }

    loop {}
}
