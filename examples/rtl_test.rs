use ctrlc;
use log::{error, info, warn};
use rtlsdr_rs::{error::Result, RtlSdr, TunerGain};
use sdre_rust_logging::SetupLogging;
use std::{
    process::exit,
    sync::atomic::{AtomicBool, Ordering},
};

// enum TestMode {
//     NO_BENCHMARK,
//     TUNER_BENCHMARK,
//     PPM_BENCHMARK,
// }
const DEFAULT_BUF_LENGTH: usize = 16 * 16384;

const SAMPLE_RATE: u32 = 2_048_000;
const FREQ: u32 = 1090000000;

#[derive(Debug)]
enum ArgParseError {
    InvalidLogLevel(String),
    UnknownArg(String),
    BadFrequency(String),
    BadValue(String),
    SerialAndIndexBothSet(String),
}

#[derive(Debug)]
struct Args {
    log_level: String,
    display_number_of_samples: bool,
    display_buffer: bool,
    frequency: u32,
    parts_per_million: i32,
    gain: f32,
    serial: Option<String>,
    index: Option<usize>,
}

impl Args {
    fn try_parse<It: Iterator<Item = String>>(
        mut arg_it: It,
    ) -> std::result::Result<Args, ArgParseError> {
        // Skip program name
        let _ = arg_it.next();

        let mut log_level = "info".to_string();
        let mut display_number_of_samples = false;
        let mut display_buffer = false;
        let mut frequency = FREQ;
        let mut parts_per_million = 0;
        let mut gain: f32 = -1.0;
        let mut serial = None;
        let mut index = None;

        while let Some(arg) = arg_it.next() {
            match arg.as_str() {
                "--loglevel" | "-l" => {
                    log_level = arg_it
                        .next()
                        .ok_or(ArgParseError::InvalidLogLevel(arg.clone()))?;
                },
                "--display-number-of-samples" | "-d" => {
                    display_number_of_samples = true;
                },
                "--display-buffer" | "-b" => {
                    display_buffer = true;
                },
                "--frequency" | "-f" => {
                    frequency = arg_it
                        .next()
                        .ok_or(ArgParseError::BadFrequency(arg.clone()))?
                        .parse()
                        .map_err(|_| ArgParseError::BadFrequency(arg.clone()))?;
                },
                "--parts-per-million" | "-p" => {
                    parts_per_million = arg_it
                        .next()
                        .ok_or(ArgParseError::BadValue(arg.clone()))?
                        .parse()
                        .map_err(|_| ArgParseError::BadValue(arg.clone()))?;
                },
                "--gain" | "-g" => {
                    gain = arg_it
                        .next()
                        .ok_or(ArgParseError::BadValue(format!("Gain {}", arg.clone())))?
                        .parse()
                        .map_err(|_| ArgParseError::BadValue(format!("Gain {}", arg.clone())))?;
                },
                "--serial" | "-s" => {
                    serial = Some(
                        arg_it
                            .next()
                            .ok_or(ArgParseError::BadValue(format!("Serial {}", arg.clone())))?,
                    );
                },
                "--index" | "-i" => {
                    index = Some(
                        arg_it
                            .next()
                            .ok_or(ArgParseError::BadValue(format!("Index {}", arg.clone())))?
                            .parse()
                            .map_err(|_| ArgParseError::BadValue(format!("Index {}", arg.clone())))?,
                    );
                },
                "--help" => {
                    println!("{}", Args::help());
                    exit(0);
                }
                _ => {
                    return Err(ArgParseError::UnknownArg(arg));
                }
            }
        }

        if serial.is_some() && index.is_some() {
            return Err(ArgParseError::SerialAndIndexBothSet(
                "Serial and index cannot both be set".to_string(),
            ));
        }

        if serial.is_none() && index.is_none() {
            warn!("No serial or index set, using index 0");
            index = Some(0);
        }

        Ok(Args {
            log_level,
            display_number_of_samples,
            display_buffer,
            frequency,
            parts_per_million,
            gain: gain * 10.0,
            serial,
            index,
        })
    }

    fn parse<It: Iterator<Item = String>>(arg_it: It) -> Args {
        match Self::try_parse(arg_it) {
            Ok(v) => v,
            Err(e) => {
                println!("Argument parsing failed: {e:?}");
                println!("{}", Args::help());
                exit(1);
            }
        }
    }

    fn help() -> String {
        format!(
            "Usage: {}\n
            --serial / -s: The serial number of the device to use.\n
            --index / -i: The index of the device to use.\n
            --loglevel / -l: The log level to use. Default is 'info'.\n
            --frequency / -f: The frequency to tune to. In hertz. Default is 1090000000.\n
            --display-number-of-samples / -d: Display the number of samples read.\n
            --display-buffer / -b: Display the buffer read.\n
            --parts-per-million / -p: The parts per million error to set. Default is 0.\n
            --gain / -g: The gain to set. Default is 0.\n
            --help / -h: Display this help message.",
            env!("CARGO_PKG_NAME")
        )
    }
}

fn main() -> Result<()> {
    // parse args
    // temporarily set log level to info so we can see the args
    "info".enable_logging();
    let args: Args = Args::parse(std::env::args());

    args.log_level.enable_logging();

    // Create shutdown flag and set it when ctrl-c signal caught
    static SHUTDOWN: AtomicBool = AtomicBool::new(false);
    if let Err(e) = ctrlc::set_handler(|| {
        SHUTDOWN.swap(true, Ordering::Relaxed);
    }) {
        error!("Error setting Ctrl-C handler: {}", e);
    }

    // Open device

    let mut sdr = if let Some(serial) = args.serial {
        RtlSdr::open_by_serial(&serial)?
    } else {
        RtlSdr::open_by_index(args.index.unwrap())?
    };

    let gains = sdr.get_tuner_gains()?;
    // info!(
    //     "Supported gain values ({}): {:?}",
    //     gains.len(),
    //     gains
    //         .iter()
    //         .map(|g| { *g as f32 / 10.0 })
    //         .collect::<Vec<_>>()
    // );

    // set frequency
    info!("Set frequency to {} MHz", FREQ as f32 / 1_000_000.0);
    sdr.set_center_freq(args.frequency)?;

    // set ppm error
    info!("Set PPM to {}", args.parts_per_million);
    sdr.set_freq_correction(args.parts_per_million)?;

    // set gain
    let closest_gain = if args.gain < 0.0 {
        TunerGain::Auto
    } else {
        let temp_gain = gains
        .iter()
        .min_by(|a, b| {
            (args.gain - **a as f32).abs().partial_cmp(&(args.gain - **b as f32).abs()).unwrap()
        })
        .unwrap();

        TunerGain::Manual(*temp_gain)
    };

    info!("Set gain to {}", closest_gain);
    sdr.set_tuner_gain(closest_gain)?;

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

            if args.display_number_of_samples {
                info!("read {} samples!", n);
            }

            if args.display_buffer {
                info!("Buffer: {:?}\n", &buf[0..n]);
            }
        }
    }

    info!("Close");
    sdr.close()?;
    Ok(())
}
