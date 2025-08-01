//! Async SPI Memory Example
//!
//! To build this example, use:
//! ```
//! cargo build --example async --no-default-features
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
    time::Hertz,
};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    defmt::info!("Starting SPI Memory Example");

    // Spawn the main SPI memory task
    let p = embassy_stm32::init(Default::default());
    let cs = Output::new(p.PA7, Level::High, Speed::High);

    defmt::info!("SPI memory example started");

    let mut spi_config = spi::Config::default();
    spi_config.mode = spi::MODE_0;
    spi_config.frequency = Hertz(8_000_000); // 8 MHz

    // Create async SPI instance with proper DMA channels for SPI3
    // Note: DMA1_CH5 for TX and DMA1_CH0 for RX are valid for SPI3
    let spi = Spi::new(
        p.SPI3, p.PC10, p.PB2, p.PC11, p.DMA1_CH5, p.DMA1_CH0, spi_config,
    );
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(spi);
    let spi_device = SpiDevice::new(&spi_bus, cs);

    // Create SPI memory driver (async version via maybe-async)
    let mut flash = spi_memory::series25::Flash::init(spi_device, embassy_time::Delay)
        .await
        .unwrap();

    flash.release_power_down().await.unwrap();

    defmt::info!("SPI Memory driver initialized");

    defmt::info!("Starting SPI memory tests...");

    // Test 1: Read device ID (async)
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

    // Test 2: Read status register (async)
    match flash.read_status().await {
        Ok(status) => {
            defmt::info!("Status register: 0x{:02x}", status.bits());
        }
        Err(_e) => {
            defmt::error!("Failed to read status");
            debug::exit(debug::EXIT_FAILURE);
        }
    }

    // Test 3: Write and read a small block of data (async)
    let test_address = 0x1000; // Start at 4KB offset
    let test_data = [0xAA, 0x55, 0xF0, 0x0F, 0x12, 0x34, 0x56, 0x78];
    let mut read_buffer = [0u8; 8];

    defmt::info!("Testing write/read at address 0x{:04x}", test_address);

    // Erase the sector first (most SPI flash requires this before writing)
    defmt::info!("Erasing sector...");
    if let Err(_e) = flash.erase_sectors(test_address, 1).await {
        defmt::error!("Failed to erase sector");
        debug::exit(debug::EXIT_FAILURE);
    }

    // Write test data (async)
    defmt::info!("Writing test data: {:02x}", test_data.as_slice());
    if let Err(_e) = flash.write_bytes(test_address, &test_data).await {
        defmt::error!("Failed to write data");
        debug::exit(debug::EXIT_FAILURE);
    }

    // Read back the data (async)
    defmt::info!("Reading back data...");
    if let Err(_e) = flash.read(test_address, &mut read_buffer).await {
        defmt::error!("Failed to read data");
        debug::exit(debug::EXIT_FAILURE);
    }

    defmt::info!("Read data: {:02x}", read_buffer.as_slice());

    // Verify the data matches what we wrote
    if test_data != read_buffer {
        defmt::error!("Write/Read test FAILED!");
        defmt::error!("Expected: {:02x}", test_data.as_slice());
        defmt::error!("Got:      {:02x}", read_buffer.as_slice());
        debug::exit(debug::EXIT_FAILURE);
    }

    defmt::info!("Write/Read test PASSED!");

    // Test 4: Test reading from different addresses (async)
    let test_addresses = [0x0000, 0x1000, 0x2000];
    for &addr in &test_addresses {
        let mut buffer = [0u8; 4];
        if let Err(_e) = flash.read(addr, &mut buffer).await {
            defmt::error!("Failed to read from address 0x{:04x}", addr);
        } else {
            defmt::info!("Data at 0x{:04x}: {:02x}", addr, buffer.as_slice());
        }
    }

    // // Test 5: Erase all memory - this can take some time
    // defmt::info!("Erasing entire memory chip...");
    // if let Err(_e) = flash.erase_all().await {
    //     defmt::error!("Failed to erase entire memory chip");
    //     debug::exit(debug::EXIT_FAILURE);
    // }
    // // Verify that the memory is erased
    // let mut buffer = [0u8; 256];
    // if let Err(_e) = flash.read(0, &mut buffer).await {
    //     defmt::error!("Failed to read back erased memory");
    //     debug::exit(debug::EXIT_FAILURE);
    // } else {
    //     // Verify that all bytes are 0xFF
    //     assert!(
    //         buffer.iter().all(|&b| b == 0xFF),
    //         "Memory not erased properly"
    //     );
    // }
    // defmt::info!("Memory erased successfully!");

    defmt::info!("All tests completed successfully!");

    debug::exit(debug::EXIT_SUCCESS);

    unreachable!();
}
