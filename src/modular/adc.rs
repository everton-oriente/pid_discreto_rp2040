// ADC file for the modular project.
/*!
 * -----------------------------------------------------------------------------
 *  Project     : ADC file for the modular project.
 *  File        : adc.rs
 *  Created by  : Everton Oriente
 *  Date        : 2025-07-22
 *  * -----------------------------------------------------------------------------
 *  Description :
 *      The module is responsible about to acquire and send the information regarding the temperature,
 *      humidity and luminosity from the sensors.
 *
 *  Target MCU  : Raspberry Pi Pico W (RP2040 and CYW43)
 *  Framework   : Embassy, no_std
 *
 */

use defmt::*; // For logging via RTT
use embassy_rp::adc::{Adc, Async, Channel};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::watch::{DynReceiver, Watch};
use embassy_time::Timer;
use heapless::HistoryBuf;

use {defmt_rtt as _, panic_probe as _}; // RTT logging and panic handler



// Development of a ring buffer using heapless and embassy_sync
pub struct HeaplessMutexRingBuffer {
    mutex: Mutex<ThreadModeRawMutex, HistoryBuf<u16, 16>>,
}

impl HeaplessMutexRingBuffer {
    pub fn new() -> Self {
        Self {
            mutex: Mutex::new(HistoryBuf::new()),
        }
    }

    pub async fn add(&self, value: u16) {
        let mut ringbuffer = self.mutex.lock().await;
        ringbuffer.write(value); // Handle overflow if necessary
    }

    pub async fn get_all(&self)-> u32 {

            let ringbuffer = self.mutex.lock().await;
            let avg = ringbuffer.iter().copied().sum::<u16>() as u32 / (ringbuffer.len() as u32);
            avg
}
/*
    pub async fn len(self)-> usize {
        let ringbuffer = self.mutex.lock().await;
        ringbuffer.len()

    }

    pub async fn is_empty(self) -> bool {
        let mut ringbuffer = self.mutex.lock().await;
        ringbuffer.is_empty()
    }

    pub async fn clear(self)  {
        let mut ringbuffer = self.mutex.lock().await;
        ringbuffer.clear(); // Clear the buffer
    }
    */
}


const ADC0_CONSUMERS: usize = 3;
static ADC0_CHANNEL: Watch<ThreadModeRawMutex, u16, ADC0_CONSUMERS> = Watch::new();

pub fn get_receiver_adc0() -> Option<DynReceiver<'static, u16>> {
    ADC0_CHANNEL.dyn_receiver()
}
/*
const ADC1_CONSUMERS: usize = 1;
static ADC1_CHANNEL: Watch<ThreadModeRawMutex, u16, ADC1_CONSUMERS> = Watch::new();

pub fn get_receiver_adc1() -> Option<DynReceiver<'static, u16>> {
    ADC1_CHANNEL.dyn_receiver()
}
*/

/*
const ADC2_CONSUMERS: usize = 1;
static ADC2_CHANNEL: Watch<ThreadModeRawMutex, u16, ADC2_CONSUMERS> = Watch::new();

pub fn get_receiver_adc2() -> Option<DynReceiver<'static, u16>> {
    ADC2_CHANNEL.dyn_receiver()
}
*/

const ADCTEMP_CONSUMERS: usize = 2;
static ADCTEMP_CHANNEL: Watch<ThreadModeRawMutex, u16, ADCTEMP_CONSUMERS> = Watch::new();

pub fn get_receiver_adctemp() -> Option<DynReceiver<'static, u16>> {
    ADCTEMP_CHANNEL.dyn_receiver()
}

// This task is used to read all the ADC channels regarding the RPI Pico W, where ADC0-ADC2 can be used to measure anything from 0 to 3.3V.
// ADC3 is used to measure the temperature die of the RP2040 or RP2350.
#[embassy_executor::task]
pub async fn read_adc_channels(
    adc_mutex: &'static Mutex<ThreadModeRawMutex,
    Adc<'static, Async>>,
    mut chan_0: Channel<'static>, // Ref Temp
    //mut chan_1: Channel<'static>,
    //mut chan_2: Channel<'static>,
    mut chan_temp: Channel<'static>, // Temperature of the DIE
) {

     let temp_res = HeaplessMutexRingBuffer::new( );
     temp_res.add(4094).await;
    loop {
        info!("Reading Temperature...");
        let tx_adc0 = ADC0_CHANNEL.sender();
        // let tx_adc1 = ADC1_CHANNEL.sender();
        // let tx_adc2 = ADC2_CHANNEL.sender();
        let tx_adctemp = ADCTEMP_CHANNEL.sender();

        let result_adc_0 = {
            let mut adc_channel_0 = adc_mutex.lock().await;
            adc_channel_0.read(&mut chan_0).await
        };

        match result_adc_0 {
            Ok(value) => {
                info!("Reading Reference Temperature Resistor...");
                info!("RefTempRes: {}", value);
                temp_res.add(value).await;
                let avg_temp_res = temp_res.get_all().await;
                info!("RefTempResAvg: {}", avg_temp_res);
                // Send the value to the channel
                tx_adc0.send(avg_temp_res as u16);
            }
            Err(e) => error!("ADC read error: {}", e),
        }
        /*
        info!("Reading ADC_1...");
        let result_adc_1 = {
            let adc_refcell_adc_channel_1 = adc_mutex.lock().await;
            let mut adc_channel_1 = adc_refcell_adc_channel_1.borrow_mut();
            adc_channel_1.read(&mut chan_1).await
        };

        match result_adc_1 {
            Ok(value) => {
                info!("Luminosity: {}", value);
                // Send the value to the channel
                tx_adc1.send(value);
            }
            Err(e) => error!("ADC read error: {}", e),
        }
        */
        /*
        info!("Reading ADC_2...");
        let result_adc_2 = {
            let adc_refcell_adc_channel_2 = adc_mutex.lock().await;
            let mut adc_channel_2 = adc_refcell_adc_channel_2.borrow_mut();
            adc_channel_2.read(&mut chan_2).await
        };

        match result_adc_2 {
            Ok(value) => {
                info!("Luminosity: {}", value);
                // Send the value to the channel
                tx_adc2.send(value);
            }
            Err(e) => error!("ADC read error: {}", e),
        }
        */
        // The temperature should never rises up to 75°C, because this is the limit of the chip.
        info!("Reading temperature of the die...");
        let result_adc_temp = {
            let mut adc_channel_temp = adc_mutex.lock().await;
            adc_channel_temp.read(&mut chan_temp).await
        };

        match result_adc_temp {
            Ok(raw) => {
                let voltage = raw as f32 * 3.3 / 4096.0;
                let temp = 27.0 - (voltage - 0.706) / 0.001721;
                info!("Temp Die: {} °C (raw: {})", temp, raw);
                tx_adctemp.send(raw);
            }
            Err(e) => error!("Temp read error: {}", e),
        }

        Timer::after_millis(1_000).await; // Wait for 5 minutes before reading again
    }
}
