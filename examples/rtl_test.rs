use ctrlc;
use rtlsdr_rs::{error::Result, RtlSdr};
use std::sync::atomic::{AtomicBool, Ordering};
use log::{info, error};
use sdre_rust_logging::SetupLogging;

// enum TestMode {
//     NO_BENCHMARK,
//     TUNER_BENCHMARK,
//     PPM_BENCHMARK,
// }
const DEFAULT_BUF_LENGTH: usize = 16 * 16384;

const SAMPLE_RATE: u32 = 2_048_000;
const FREQ: u64 = 10900000000;

fn main() -> Result<()> {
    // set the log level
    "debug".enable_logging();

    // Create shutdown flag and set it when ctrl-c signal caught
    static SHUTDOWN: AtomicBool = AtomicBool::new(false);
    if let Err(e) = ctrlc::set_handler(|| {
        SHUTDOWN.swap(true, Ordering::Relaxed);
    }) {
        error!("Error setting Ctrl-C handler: {}", e);
    }

    // Open device
    let mut sdr = RtlSdr::open_by_index(0).expect("Unable to open SDR device!");

    let gains = sdr.get_tuner_gains()?;
    info!(
        "Supported gain values ({}): {:?}",
        gains.len(),
        gains
            .iter()
            .map(|g| { *g as f32 / 10.0 })
            .collect::<Vec<_>>()
    );

    // set frequency
    info!("Set frequency to {} MHz", FREQ as f32 / 1_000_000.0);
    sdr.set_center_freq(FREQ)?;

    // Set sample rate
    sdr.set_sample_rate(SAMPLE_RATE)?;
    info!("Sampling at {} S/s", sdr.get_sample_rate());

    // Enable test mode
    info!("Enable test mode");
    sdr.set_testmode(true)?;

    // Reset the endpoint before we try to read from it (mandatory)
    info!("Reset buffer");
    sdr.reset_buffer()?;

    info!("Reading samples in sync mode...");
    let mut buf: [u8; DEFAULT_BUF_LENGTH] = [0; DEFAULT_BUF_LENGTH];
    loop {
        if SHUTDOWN.load(Ordering::Relaxed) {
            break;
        }
        let n = sdr.read_sync(&mut buf);
        if n.is_err() {
           error!("Read error: {:#?}", n);
        } else {
            let n = n.unwrap();
            if n < DEFAULT_BUF_LENGTH {
                error!("Short read ({:#?}), samples lost, exiting!", n);
                break;
            }

            info!("read {} samples!", n);
        }
    }

    info!("Close");
    sdr.close()?;
    Ok(())
}
