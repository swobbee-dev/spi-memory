//! Async SPI Memory Example
//!
//! To build this example, use:
//! ```
//! cargo build --example sync --features blocking
//! ```

#![no_std]
#![no_main]

use core::cell::RefCell;
use cortex_m_rt::entry;
use cortex_m_semihosting::debug;
use defmt_rtt as _;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_stm32::mode::Blocking;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    spi::{self, Spi},
    time::Hertz,
};
use embassy_sync::blocking_mutex::{Mutex, raw::NoopRawMutex};
use panic_probe as _;

#[entry]
fn main() -> ! {
    defmt::info!("Starting SPI Memory Example");

    let p = embassy_stm32::init(Default::default());

    let cs = Output::new(p.PA7, Level::High, Speed::High);

    let mut spi_config = spi::Config::default();
    spi_config.mode = spi::MODE_0;
    spi_config.frequency = Hertz(8_000_000); // 8 MHz

    let spi = Spi::new_blocking(p.SPI3, p.PC10, p.PB2, p.PC11, spi_config);
    let spi_bus = Mutex::<NoopRawMutex, RefCell<Spi<'static, Blocking>>>::new(spi.into());
    let spi_device = SpiDevice::new(&spi_bus, cs);

    // Create SPI memory driver
    let mut flash = spi_memory::series25::Flash::init(spi_device, embassy_time::Delay).unwrap();

    defmt::info!("SPI Memory driver initialized");

    defmt::info!("Starting SPI memory tests...");

    // Test 1: Read device ID
    match flash.read_jedec_id() {
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

    // Test 2: Read status register
    match flash.read_status() {
        Ok(status) => {
            defmt::info!("Status register: 0x{:02x}", status.bits());
        }
        Err(_e) => {
            defmt::error!("Failed to read status");
            debug::exit(debug::EXIT_FAILURE);
        }
    }

    // Test 3: Write and read a small block of data
    let test_address = 0x1000; // Start at 4KB offset
    let test_data = [0xAA, 0x55, 0xF0, 0x0F, 0x12, 0x34, 0x56, 0x78];
    let mut read_buffer = [0u8; 8];

    defmt::info!("Testing write/read at address 0x{:04x}", test_address);

    // Erase the sector first (most SPI flash requires this before writing)
    defmt::info!("Erasing sector...");
    if let Err(_e) = flash.erase_sectors(test_address, 1) {
        defmt::error!("Failed to erase sector");
        debug::exit(debug::EXIT_FAILURE);
    }

    // Write test data
    defmt::info!("Writing test data: {:02x}", test_data.as_slice());
    if let Err(_e) = flash.write_bytes(test_address, &test_data) {
        defmt::error!("Failed to write data");
        debug::exit(debug::EXIT_FAILURE);
    }

    // Read back the data
    defmt::info!("Reading back data...");
    if let Err(_e) = flash.read(test_address, &mut read_buffer) {
        defmt::error!("Failed to read data");
        debug::exit(debug::EXIT_FAILURE);
    }

    defmt::info!("Read data: {:02x}", read_buffer.as_slice());

    // Verify the data
    if test_data == read_buffer {
        defmt::info!("Write/Read test PASSED!");
    } else {
        defmt::error!("Write/Read test FAILED!");
        defmt::error!("Expected: {:02x}", test_data.as_slice());
        defmt::error!("Got:      {:02x}", read_buffer.as_slice());
    }

    // Test 4: Test reading from different addresses
    let test_addresses = [0x0000, 0x1000, 0x2000];
    for &addr in &test_addresses {
        let mut buffer = [0u8; 4];
        if let Err(_e) = flash.read(addr, &mut buffer) {
            defmt::error!("Failed to read from address 0x{:04x}", addr);
        } else {
            defmt::info!("Data at 0x{:04x}: {:02x}", addr, buffer.as_slice());
        }
    }

    defmt::info!("All tests completed successfully!");

    debug::exit(debug::EXIT_SUCCESS);
    unreachable!();
}
