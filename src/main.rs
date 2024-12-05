use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the audio host
    let host = cpal::default_host();
    
    // Get the default input device
    let input_device = host
        .default_input_device()
        .expect("Failed to get default input device");
    println!("Using input device: {}", input_device.name()?);

    // Configure the input stream
    let config = input_device.default_input_config()?;
    println!("Default input config: {:?}", config);

    // Set up WAV file specifications
    let spec = WavSpec {
        channels: config.channels() as u16,
        sample_rate: config.sample_rate().0,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    // Create WAV writer
    let writer = Arc::new(Mutex::new(WavWriter::create(
        "recorded_audio.wav",
        spec,
    )?));

    // Set up the audio stream
    let writer_clone = Arc::clone(&writer);
    let err_fn = move |err| {
        eprintln!("An error occurred on stream: {}", err);
    };

    let stream = match config.sample_format() {
        cpal::SampleFormat::I16 => input_device.build_input_stream(
            &config.into(),
            move |data: &[i16], _: &_| {
                if let Ok(mut writer) = writer_clone.lock() {
                    for &sample in data {
                        writer.write_sample(sample).unwrap();
                    }
                }
            },
            err_fn,
            None,  // Added buffer size parameter
        )?,
        cpal::SampleFormat::F32 => input_device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &_| {
                if let Ok(mut writer) = writer_clone.lock() {
                    for &sample in data {
                        // Convert f32 to i16
                        let sample = (sample * i16::MAX as f32) as i16;
                        writer.write_sample(sample).unwrap();
                    }
                }
            },
            err_fn,
            None,  // Added buffer size parameter
        )?,
        _ => return Err("Unsupported sample format".into()),
    };

    // Start the stream
    stream.play()?;
    println!("Recording... Press Ctrl+C to stop");

    // Record for a specified duration (or implement your own stopping mechanism)
    std::thread::sleep(Duration::from_secs(10));

    // Stop the stream and finalize the WAV file
    drop(stream);
    if let Ok(writer) = Arc::try_unwrap(writer) {
        writer.into_inner().unwrap().finalize()?;
    }

    println!("Recording saved to recorded_audio.wav");
    Ok(())
}