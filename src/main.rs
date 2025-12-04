
//! This example tests the RP Pico W onboard LED via the CYW43 Wi-Fi chip.
//!
//! It does not work with the original RP Pico board (non-W version). See `blinky.rs` for that.

#![no_std] // Don't link the standard library (needed for embedded targets)
#![no_main] // Disable normal main entry point; we use `#[embassy_executor::main]` instead

// Import required crates and modules
 
mod modular;

use defmt::*; // For logging via RTT
use embassy_executor::Spawner;

use embassy_rp::adc::{Adc, Async, Channel, Config as AdcConfig, InterruptHandler as AdcIrq};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{AnyPin, Flex, Level, Output, Pull};
use embassy_rp::i2c::{Config as I2c_config, I2c, InterruptHandler};
use embassy_rp::peripherals::{I2C0};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Timer};
use static_cell::StaticCell;
use embassy_rp::pwm::{Config as PwmConfig, Pwm};
use {defmt_rtt as _, panic_probe as _}; // RTT logging and panic handler



bind_interrupts!(struct Irqs {

    // Bind the interrupt handler to  ADC IRQ
    ADC_IRQ_FIFO => AdcIrq;

    // Bind the interrupt handler to  I2C IRQ
    I2C0_IRQ => InterruptHandler<I2C0>;

});


/// Main async entry point
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize Embassy peripherals and clocks
    let p = embassy_rp::init(Default::default());

    // Create an Output to the LED
    let blinky_led = Output::new(p.PIN_25, Level::Low); // Led external

    // Configure I2C
    let sda = p.PIN_20;
    let scl = p.PIN_21;

    let mut i2c_config = I2c_config::default();
    i2c_config.frequency = 100_000;
    let i2c = I2c::new_async(p.I2C0, scl, sda, Irqs, i2c_config);

    // Create an ADC peripheral
    let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    // Create a mutex to protect the ADC
    static ADC: StaticCell<Mutex<ThreadModeRawMutex, Adc<'static, Async>>> = StaticCell::new();
    // Initialize the ADC
    let adc_mutex = ADC.init(Mutex::new(adc));
    // Create the ADC thats read the internal temperature of the DIE, like usage in the watchdog feed, if the temperature goes high we turn off the system
    let temp_adc = Channel::new_temp_sensor(p.ADC_TEMP_SENSOR);
    // Create the ADC 0 to read the luminosity of the system
    let lum_adc_0 = Channel::new_pin(p.PIN_26, Pull::Down);
    // Create the ADC 1 to read the ADC 1
    //let adc_1 = Channel::new_pin(p.PIN_27, Pull::Down);
    // Create the ADC 2 to read the ADC 2
    //let adc_2 = Channel::new_pin(p.PIN_28, Pull::Down);

    // Create a GPIO to read the DHT11/DHT22
    //let dht_pin = Flex::new(AnyPin::from(p.PIN_22));

    // Demonstrate PWM by setting duty cycle
    //
    // Using GP4 in Slice2, make sure to use an appropriate resistor.
    let desired_freq_hz = 100_000;
    let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
    let divider = 8u8;
    let period = (clock_freq_hz / (desired_freq_hz * divider as u32)) as u16 - 1; 

    let mut c = PwmConfig::default();
    c.top = period;
    c.divider = divider.into();

    let slice_1 = p.PWM_SLICE1;
    let pin_2 = p.PIN_2;


    let pwm_temp = Pwm::new_output_a(slice_1, pin_2, c.clone());// Small delay to let the PWM initialize properly


    // Spawn the LED task
    info!("Starting LED toggle task");
    unwrap!(spawner.spawn(modular::toogle_led(blinky_led)));
    Timer::after_millis(100).await; // Small delay to let the LED task start properly

    // Spawn the luminosity task to read the luminosity
    info!("Starting luminosity ADC task");
    unwrap!(spawner.spawn(modular::read_adc_channels(adc_mutex, lum_adc_0, temp_adc))); // Here you should add in compliance how many adc are going to use
    Timer::after_millis(100).await; // Small delay to let the ADC task start properly

    // Spawn the process_adc_channel_0 task
    info!("Starting ADC channel 0 processing task");
    unwrap!(spawner.spawn(modular::process_adc_channel_0())); // Here you should add in compliance how many adc are going to use
    Timer::after_millis(100).await; // Small delay to let the ADC task start properly

    // Spawn the process_adc_channel_temp task
    info!("Starting ADC temperature processing task");
    unwrap!(spawner.spawn(modular::process_adc_channel_temp())); // Here you should add in compliance how many adc are going to use
    Timer::after_millis(100).await; // Small delay to let the ADC task start properly

    // Spawn the I2C Display task
    info!("Starting OLED display task");
    unwrap!(spawner.spawn(modular::oled_task(i2c)));
    Timer::after_millis(100).await; // Small delay to let the OLED task start properly

    // Spawn the PWM task
    info!("Starting PWM task");
    unwrap!(spawner.spawn(modular::pwm_set_dutycycle(pwm_temp)));
    Timer::after_millis(100).await; // Small delay to let the PWM task



}
