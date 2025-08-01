//! Async SPI Memory Example
//!
//! To build this example, use:
//! ```
//! cargo build --example write_verify_async --no-default-features
//! ```

#![no_std]
#![no_main]

use cortex_m_semihosting::debug;
use defmt;
use defmt_rtt as _; // Global logger
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    spi::{self, Spi},
    {Config, rcc, time::Hertz},
};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let mut config = Config::default();
    configure_clock(&mut config);
    let p = embassy_stm32::init(config);

    let cs = Output::new(p.PA7, Level::High, Speed::High);

    let mut spi_config = spi::Config::default();
    spi_config.mode = spi::MODE_0;
    spi_config.frequency = Hertz(54_000_000); // 54 MHz

    let spi = Spi::new(
        p.SPI3, p.PC10, p.PB2, p.PC11, p.DMA1_CH5, p.DMA1_CH0, spi_config,
    );
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(spi);
    let spi_device = SpiDevice::new(&spi_bus, cs);

    // Create SPI memory driver (async version via maybe-async)
    let mut flash = spi_memory::series25::Flash::init(spi_device, embassy_time::Delay)
        .await
        .unwrap();

    defmt::info!("Starting SPI memory tests...");

    match flash.read_jedec_id().await {
        Ok(id) => {
            defmt::info!(
                "JEDEC ID - Manufacturer: 0x{:02x}, Device: {:02x}",
                id.mfr_code(),
                id.device_id()
            );
        }
        Err(_e) => {
            defmt::error!("Failed to read JEDEC ID");
            debug::exit(debug::EXIT_FAILURE);
        }
    }

    let mut fail_count = 0u32;

    const ITERATIONS: usize = 100;
    for i in 0..ITERATIONS {
        let mut inb = [0u8; 512];
        let mut inb_copy = [0u8; 512];
        let mut outb = [0u8; 512];
        for (n, b) in inb.iter_mut().enumerate() {
            *b = (n + i) as u8;
        }
        inb_copy.copy_from_slice(&inb);
        const ADDR: u32 = 0x0000_0000;
        match i {
            0..=90 => {
                defmt::info!("sector erase");
                flash.erase_sectors(ADDR, 1).await.unwrap();
            }
            91..=98 => {
                defmt::info!("block erase");
                flash.erase_block(ADDR).await.unwrap();
            }
            99..=100 => {
                defmt::info!("chip erase");
                flash.erase_all().await.unwrap();
            }
            _ => (),
        }
        // inb will get overwritten below!
        flash.write_bytes(ADDR, &mut inb).await.unwrap();
        flash.read(ADDR, &mut outb).await.unwrap();
        if outb != inb_copy {
            defmt::error!("Failed verification");
            // defmt::error!("wrote: {}", inb_copy.hex());
            // defmt::error!("read:  {}", outb.hex());
            fail_count += 1;
        }
        defmt::info!("write iteration {}/{}", i + 1, ITERATIONS);
    }

    if fail_count > 0 {
        defmt::error!("num failures: {}", fail_count);
    }
    defmt::info!("DONE");

    debug::exit(debug::EXIT_SUCCESS);

    unreachable!();
}

fn configure_clock(config: &mut Config) {
    config.rcc.hse = Some(rcc::Hse {
        freq: Hertz(8_000_000),
        mode: rcc::HseMode::Oscillator,
    });
    config.rcc.pll_src = rcc::PllSource::HSE;
    config.rcc.pll = Some(rcc::Pll {
        prediv: rcc::PllPreDiv::DIV4,
        mul: rcc::PllMul::MUL216,
        divp: Some(rcc::PllPDiv::DIV2),
        divq: Some(rcc::PllQDiv::DIV2),
        divr: Some(rcc::PllRDiv::DIV2),
    });
    config.rcc.sys = rcc::Sysclk::PLL1_P;
    config.rcc.ahb_pre = rcc::AHBPrescaler::DIV1;
    config.rcc.apb1_pre = rcc::APBPrescaler::DIV4;
    config.rcc.apb2_pre = rcc::APBPrescaler::DIV2;
}
