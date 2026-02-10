use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, StreamConfig};
use screencapturekit::prelude::*;
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub enum CaptureMode {
    Visio,     // ScreenCaptureKit (system audio) + mic
    InPerson,  // mic only
}

pub struct AudioCapturer {
    mode: CaptureMode,
    stream: Option<cpal::Stream>,
    sc_stream: Option<SCStream>,
    capturing: Arc<AtomicBool>,
    /// The actual sample rate of the device stream. May differ from 16kHz if the
    /// device does not natively support it. Resampling can be added later.
    pub actual_sample_rate: u32,
}

impl AudioCapturer {
    pub fn new(mode: CaptureMode) -> Self {
        Self {
            mode,
            stream: None,
            sc_stream: None,
            capturing: Arc::new(AtomicBool::new(false)),
            actual_sample_rate: 16000,
        }
    }

    /// Start capturing audio. Returns a receiver for audio chunks.
    /// Each chunk is a Vec<i16> of PCM samples at 16kHz mono (or the closest
    /// supported sample rate if 16kHz is not available).
    pub fn start(
        &mut self,
    ) -> Result<mpsc::Receiver<Vec<i16>>, Box<dyn std::error::Error>> {
        match self.mode {
            CaptureMode::Visio => self.start_visio_capture(),
            CaptureMode::InPerson => self.start_mic_capture(),
        }
    }

    /// Stop capturing audio.
    pub fn stop(&mut self) {
        self.capturing.store(false, Ordering::SeqCst);
        // Stop ScreenCaptureKit stream if present (Visio mode).
        if let Some(sc_stream) = self.sc_stream.take() {
            let _ = sc_stream.stop_capture();
            drop(sc_stream);
        }
        // Dropping the cpal stream stops it. We take it out of the Option so it gets dropped.
        if let Some(stream) = self.stream.take() {
            // Pause before dropping to ensure clean shutdown.
            let _ = stream.pause();
            drop(stream);
        }
    }

    /// Check if currently capturing.
    pub fn is_capturing(&self) -> bool {
        self.capturing.load(Ordering::SeqCst)
    }

    /// Internal: start Visio mode capture (system audio via ScreenCaptureKit + mic via cpal).
    ///
    /// Both audio sources send `Vec<i16>` chunks into the same channel. The system
    /// audio is captured via ScreenCaptureKit at 16kHz mono, and the microphone via
    /// cpal (reusing the InPerson logic). Both sources push interleaved chunks into
    /// the single sender; no sample-level mixing is done here.
    fn start_visio_capture(
        &mut self,
    ) -> Result<mpsc::Receiver<Vec<i16>>, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel::<Vec<i16>>();

        // --- 1. Set up ScreenCaptureKit for system audio capture ---

        let content = SCShareableContent::get()
            .map_err(|e| format!("Failed to get shareable content: {}", e))?;

        let displays = content.displays();
        let display = displays
            .into_iter()
            .next()
            .ok_or("No display available for ScreenCaptureKit capture")?;

        let filter = SCContentFilter::create()
            .with_display(&display)
            .with_excluding_windows(&[])
            .build();

        // Configure: audio-only at 16kHz mono. We still need minimal video config
        // but we will simply ignore video frames in the handler.
        let config = SCStreamConfiguration::new()
            .with_width(2)
            .with_height(2)
            .with_captures_audio(true)
            .with_excludes_current_process_audio(true)
            .with_sample_rate(16000)
            .with_channel_count(1);

        let capturing_for_sc = Arc::clone(&self.capturing);
        let tx_sc = tx.clone();

        let mut sc_stream = SCStream::new(&filter, &config);

        sc_stream.add_output_handler(
            move |sample: CMSampleBuffer, of_type: SCStreamOutputType| {
                if of_type != SCStreamOutputType::Audio {
                    return;
                }
                if !capturing_for_sc.load(Ordering::SeqCst) {
                    return;
                }
                // Extract PCM audio data from the sample buffer.
                if let Some(audio_buffers) = sample.audio_buffer_list() {
                    for buf in audio_buffers.iter() {
                        let raw_bytes = buf.data();
                        if raw_bytes.is_empty() {
                            continue;
                        }
                        // ScreenCaptureKit delivers audio as 32-bit float PCM.
                        // Convert f32 samples to i16.
                        let float_samples: &[f32] = unsafe {
                            std::slice::from_raw_parts(
                                raw_bytes.as_ptr().cast::<f32>(),
                                raw_bytes.len() / std::mem::size_of::<f32>(),
                            )
                        };
                        let i16_samples: Vec<i16> = float_samples
                            .iter()
                            .map(|&s| f32_to_i16(s))
                            .collect();
                        let _ = tx_sc.send(i16_samples);
                    }
                }
            },
            SCStreamOutputType::Audio,
        );

        sc_stream
            .start_capture()
            .map_err(|e| format!("Failed to start ScreenCaptureKit capture: {}", e))?;

        eprintln!("[capture] ScreenCaptureKit system audio capture started (16kHz mono)");

        self.sc_stream = Some(sc_stream);

        // --- 2. Set up cpal microphone capture (same logic as InPerson) ---

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No default input device available")?;

        let device_name = device.name().unwrap_or_else(|_| "unknown".to_string());
        eprintln!("[capture] Visio mic using input device: {}", device_name);

        let (stream_config, sample_format) = self.select_input_config(&device)?;

        eprintln!(
            "[capture] Mic stream config: {} Hz, {} ch, format: {:?}",
            stream_config.sample_rate.0, stream_config.channels, sample_format
        );

        self.actual_sample_rate = stream_config.sample_rate.0;

        let capturing_for_mic = Arc::clone(&self.capturing);
        let tx_mic = tx;

        let err_callback = |err: cpal::StreamError| {
            eprintln!("[capture] Mic stream error: {}", err);
        };

        let channels = stream_config.channels as usize;

        let cpal_stream = match sample_format {
            SampleFormat::I16 => {
                let capturing = capturing_for_mic.clone();
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if !capturing.load(Ordering::SeqCst) {
                            return;
                        }
                        let mono = downmix_to_mono_i16(data, channels);
                        let _ = tx_mic.send(mono);
                    },
                    err_callback,
                    None,
                )?
            }
            SampleFormat::F32 => {
                let capturing = capturing_for_mic.clone();
                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if !capturing.load(Ordering::SeqCst) {
                            return;
                        }
                        let i16_data: Vec<i16> = data.iter().map(|&s| f32_to_i16(s)).collect();
                        let mono = downmix_to_mono_i16(&i16_data, channels);
                        let _ = tx_mic.send(mono);
                    },
                    err_callback,
                    None,
                )?
            }
            SampleFormat::U16 => {
                let capturing = capturing_for_mic.clone();
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        if !capturing.load(Ordering::SeqCst) {
                            return;
                        }
                        let i16_data: Vec<i16> = data
                            .iter()
                            .map(|&s| (s as i32 - 32768) as i16)
                            .collect();
                        let mono = downmix_to_mono_i16(&i16_data, channels);
                        let _ = tx_mic.send(mono);
                    },
                    err_callback,
                    None,
                )?
            }
            _ => {
                return Err(format!(
                    "Unsupported sample format: {:?}",
                    sample_format
                )
                .into());
            }
        };

        cpal_stream.play()?;
        self.capturing.store(true, Ordering::SeqCst);
        self.stream = Some(cpal_stream);

        eprintln!("[capture] Visio mode fully started (system audio + mic)");

        Ok(rx)
    }

    /// Internal: start microphone-only capture (InPerson mode).
    fn start_mic_capture(
        &mut self,
    ) -> Result<mpsc::Receiver<Vec<i16>>, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No default input device available")?;

        let device_name = device.name().unwrap_or_else(|_| "unknown".to_string());
        eprintln!("[capture] Using input device: {}", device_name);

        // Try to find a config that supports 16kHz mono i16.
        // If not available, fall back to the default input config.
        let (stream_config, sample_format) =
            self.select_input_config(&device)?;

        eprintln!(
            "[capture] Stream config: {} Hz, {} ch, format: {:?}",
            stream_config.sample_rate.0, stream_config.channels, sample_format
        );

        self.actual_sample_rate = stream_config.sample_rate.0;

        let (tx, rx) = mpsc::channel::<Vec<i16>>();

        let capturing = Arc::clone(&self.capturing);

        let err_callback = |err: cpal::StreamError| {
            eprintln!("[capture] Stream error: {}", err);
        };

        let channels = stream_config.channels as usize;

        let stream = match sample_format {
            SampleFormat::I16 => {
                let capturing = capturing.clone();
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if !capturing.load(Ordering::SeqCst) {
                            return;
                        }
                        // Downmix to mono if needed, then send the chunk.
                        let mono = downmix_to_mono_i16(data, channels);
                        let _ = tx.send(mono);
                    },
                    err_callback,
                    None,
                )?
            }
            SampleFormat::F32 => {
                let capturing = capturing.clone();
                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if !capturing.load(Ordering::SeqCst) {
                            return;
                        }
                        // Convert f32 -> i16, then downmix to mono.
                        let i16_data: Vec<i16> = data
                            .iter()
                            .map(|&s| f32_to_i16(s))
                            .collect();
                        let mono = downmix_to_mono_i16(&i16_data, channels);
                        let _ = tx.send(mono);
                    },
                    err_callback,
                    None,
                )?
            }
            SampleFormat::U16 => {
                let capturing = capturing.clone();
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        if !capturing.load(Ordering::SeqCst) {
                            return;
                        }
                        // Convert u16 -> i16 (center at 0), then downmix.
                        let i16_data: Vec<i16> = data
                            .iter()
                            .map(|&s| (s as i32 - 32768) as i16)
                            .collect();
                        let mono = downmix_to_mono_i16(&i16_data, channels);
                        let _ = tx.send(mono);
                    },
                    err_callback,
                    None,
                )?
            }
            _ => {
                return Err(format!(
                    "Unsupported sample format: {:?}",
                    sample_format
                )
                .into());
            }
        };

        stream.play()?;
        self.capturing.store(true, Ordering::SeqCst);
        self.stream = Some(stream);

        Ok(rx)
    }

    /// Select the best input config for the given device.
    ///
    /// Priority:
    /// 1. 16kHz, mono, i16
    /// 2. 16kHz, any channels, i16
    /// 3. 16kHz, any channels, any format
    /// 4. Default input config (we will note the actual sample rate)
    fn select_input_config(
        &self,
        device: &cpal::Device,
    ) -> Result<(StreamConfig, SampleFormat), Box<dyn std::error::Error>> {
        let target_rate = SampleRate(16000);

        // Gather all supported input configs.
        let supported_configs: Vec<_> = device
            .supported_input_configs()?
            .collect();

        // Try to find a config range that includes 16kHz.
        // Prefer: i16 format, mono, and 16kHz support.
        let mut best: Option<(cpal::SupportedStreamConfig, i32)> = None;

        for config_range in &supported_configs {
            let min = config_range.min_sample_rate();
            let max = config_range.max_sample_rate();

            if min.0 <= 16000 && max.0 >= 16000 {
                let config = config_range.clone().with_sample_rate(target_rate);
                let mut score: i32 = 100; // base score for supporting 16kHz

                // Prefer i16
                if config.sample_format() == SampleFormat::I16 {
                    score += 20;
                } else if config.sample_format() == SampleFormat::F32 {
                    score += 10;
                }

                // Prefer mono
                if config.channels() == 1 {
                    score += 10;
                }

                if best.is_none() || score > best.as_ref().unwrap().1 {
                    best = Some((config, score));
                }
            }
        }

        if let Some((config, _score)) = best {
            let fmt = config.sample_format();
            let stream_config: StreamConfig = config.into();
            return Ok((stream_config, fmt));
        }

        // Fallback: use the default input config.
        eprintln!(
            "[capture] 16kHz not directly supported; using default input config. \
             Resampling to 16kHz may be needed."
        );
        let default_config = device.default_input_config()?;
        let fmt = default_config.sample_format();
        let stream_config: StreamConfig = default_config.into();
        Ok((stream_config, fmt))
    }
}

/// Downmix interleaved multi-channel i16 samples to mono by averaging channels.
fn downmix_to_mono_i16(data: &[i16], channels: usize) -> Vec<i16> {
    if channels == 1 {
        return data.to_vec();
    }
    data.chunks_exact(channels)
        .map(|frame| {
            let sum: i32 = frame.iter().map(|&s| s as i32).sum();
            (sum / channels as i32) as i16
        })
        .collect()
}

/// Convert an f32 sample (range -1.0 to 1.0) to i16.
fn f32_to_i16(sample: f32) -> i16 {
    let clamped = sample.clamp(-1.0, 1.0);
    (clamped * i16::MAX as f32) as i16
}
