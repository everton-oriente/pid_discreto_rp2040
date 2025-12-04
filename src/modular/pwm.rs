// PWM file for the modular project.
/*!
 * -----------------------------------------------------------------------------
 *  Project     : PWM file for the modular project.
 *  File        : pwm.rs
 *  Created by  : Everton Oriente
 *  Date        : 2025-12-01
 *  * -----------------------------------------------------------------------------
 *  Description :
 *      The module is responsible about to acquire  the information about the temperature of the system, the reference of the temperature
 *      and update the output through the PWM to control the temperature of the system and  the info to the display OLED,
 *      regarding the values about the output to control the system.
 *
 *  Target MCU  : Raspberry Pi Pico W (RP2040 and CYW43)
 *  Framework   : Embassy, no_std
 *
 */

 use defmt::info;
 use embassy_rp::pwm::{Pwm, SetDutyCycle};
 use embassy_time::{Timer};


/// Demonstrate PWM by setting duty cycle
///
/// Using GP4 in Slice2, make sure to use an appropriate resistor.
#[embassy_executor::task]
pub async fn pwm_set_dutycycle(mut pwm: Pwm<'static>){

    loop {
        // 100% duty cycle, fully on
        pwm.set_duty_cycle_fully_on().unwrap();
        Timer::after_secs(1).await;
        info!("PWM FULLY ON");

        // 66% duty cycle. Expressed as simple percentage.
        //let now = Instant::now();
        pwm.set_duty_cycle_percent(50).unwrap(); // 115 micro seconds to configure the new output for the pwm
        //let elapsed = now.elapsed();
        //info!("PWM 50% - Elapsed: {} us", elapsed.as_micros());
        Timer::after_secs(1).await;




    }
}