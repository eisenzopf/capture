use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SizedSample, StreamConfig};
use hound::{WavSpec, WavWriter};
use std::sync::{Arc, Mutex};
use chrono::Local;
use std::path::PathBuf;
use num_traits::cast::ToPrimitive;

pub struct AudioRecorder {
    stream: Arc<Mutex<Option<cpal::Stream>>>,
    writer: Arc<Mutex<Option<WavWriter<std::io::BufWriter<std::fs::File>>>>>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        AudioRecorder {
            stream: Arc::new(Mutex::new(None)),
            writer: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_recording(&self) -> Result<(), Box<dyn std::error::Error>> {
        //println!("Starting recording process...");
        
        // Get host and print info
        let host = cpal::default_host();
        //println!("Audio host: {:?}", host.id());
        
        // Get input device with error handling
        let input_device = host.default_input_device()
            .ok_or_else(|| "Failed to get default input device".to_string())?;
        //println!("Input device name: {:?}", input_device.name()?);
        
        // Get supported configs for debugging
        //println!("Supported configs:");
        //for config in input_device.supported_input_configs()? {
            //println!("  {:?}", config);
        //}
        
        // Get default config
        let config = input_device.default_input_config()
            .map_err(|e| format!("Failed to get default input config: {}", e))?;
        //println!("Selected config: {:?}", config);
        
        // Create output directory with better error handling
        let home_dir = std::env::var("HOME")
            .map_err(|e| format!("Failed to get HOME directory: {}", e))?;
        let mut path = PathBuf::from(home_dir);
        path.push("Documents");
        path.push("AudioRecordings");
        
        //println!("Creating directory at: {:?}", path);
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
        
        // Create timestamp-based filename
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        path.push(format!("recording_{}.wav", timestamp));
        //println!("Recording to file: {:?}", path);
        
        // Set up WAV spec
        let spec = WavSpec {
            channels: config.channels() as u16,
            sample_rate: config.sample_rate().0,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        //println!("WAV spec: {:?}", spec);
        
        // Create writer
        let writer = WavWriter::create(&path, spec)
            .map_err(|e| format!("Failed to create WAV writer: {}", e))?;
        *self.writer.lock().unwrap() = Some(writer);
        
        // Build the stream based on format
        //println!("Building stream with format: {:?}", config.sample_format());
        let stream = match config.sample_format() {
            cpal::SampleFormat::I16 => self.build_stream::<i16>(&input_device, &config.into())?,
            cpal::SampleFormat::F32 => self.build_stream::<f32>(&input_device, &config.into())?,
            unsupported => return Err(format!("Unsupported sample format: {:?}", unsupported).into()),
        };
        
        // Start the stream
        //println!("Starting audio stream...");
        stream.play()
            .map_err(|e| format!("Failed to start stream: {}", e))?;
        
        // Store the stream
        *self.stream.lock().unwrap() = Some(stream);
        
        //println!("Recording started successfully");
        Ok(())
    }

    pub fn stop_recording(&self) {
        if let Some(stream) = self.stream.lock().unwrap().take() {
            drop(stream);
        }
        
        if let Some(writer) = self.writer.lock().unwrap().take() {
            writer.finalize().unwrap();
        }
    }

    fn build_stream<T>(&self, 
        device: &cpal::Device, 
        config: &StreamConfig
    ) -> Result<cpal::Stream, Box<dyn std::error::Error>>
    where
        T: Sample + SizedSample + Send + ToPrimitive + 'static,
    {
        let writer = Arc::clone(&self.writer);
        
        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &_| {
                if let Some(writer) = writer.lock().unwrap().as_mut() {
                    for &sample in data {
                        let sample_i16 = if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>() {
                            (sample.to_f32().unwrap_or(0.0) * i16::MAX as f32) as i16
                        } else {
                            sample.to_i16().unwrap_or(0)
                        };
                        writer.write_sample(sample_i16).unwrap();
                    }
                }
            },
            |err| eprintln!("Error in stream: {}", err),
            None,
        )?;
        
        Ok(stream)
    }
}