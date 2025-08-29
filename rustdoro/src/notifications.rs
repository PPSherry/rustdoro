use anyhow::Result;
use rodio::{source::Source, Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use std::sync::Arc;
use crate::config::Config;

/// Audio notification manager
pub struct NotificationManager {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    config: Config,
    current_sink: Option<Arc<Sink>>,
}

impl NotificationManager {
    /// Create a new notification manager
    pub fn new(config: Config) -> Result<Self> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        
        Ok(Self {
            _stream,
            stream_handle,
            config,
            current_sink: None,
        })
    }

    /// Check if sound notifications are enabled
    pub fn is_enabled(&self) -> bool {
        !self.config.general.no_sound
    }

    /// Stop any currently playing audio
    pub fn stop_audio(&mut self) {
        if let Some(sink) = &self.current_sink {
            sink.stop();
        }
        self.current_sink = None;
    }



    /// Play session end sound with continuous looping until stopped
    pub fn play_end_sound(&mut self) -> Result<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        // Stop any currently playing audio first
        self.stop_audio();

        // Clone the audio file path to avoid borrowing issues
        let audio_file = self.config.audio.audio_file.clone();
        
        if let Some(file_path) = audio_file {
            self.play_custom_audio_file_continuous(&file_path)?;
        } else {
            self.play_default_end_sound_continuous()?;
        }
        
        Ok(())
    }

    /// Play work session start sound
    pub fn play_work_start_sound(&mut self) -> Result<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        // Stop any currently playing audio first
        self.stop_audio();

        // Clone the audio file path to avoid borrowing issues
        let audio_file = self.config.audio.audio_file.clone();
        
        if let Some(file_path) = audio_file {
            self.play_custom_audio_file_once(&file_path)?;
        } else {
            let sound_data = generate_beep_sound(600.0, 0.2); // Lower frequency for work
            self.play_sound_data_non_blocking(sound_data)?;
        }
        
        Ok(())
    }

    /// Play break start sound
    pub fn play_break_start_sound(&mut self) -> Result<()> {
        if !self.is_enabled() {
            return Ok(());
        }

        // Stop any currently playing audio first
        self.stop_audio();

        // Clone the audio file path to avoid borrowing issues
        let audio_file = self.config.audio.audio_file.clone();
        
        if let Some(file_path) = audio_file {
            self.play_custom_audio_file_once(&file_path)?;
        } else {
            let sound_data = generate_beep_sound(900.0, 0.2); // Higher frequency for break
            self.play_sound_data_non_blocking(sound_data)?;
        }
        
        Ok(())
    }





    /// Play custom audio file once (for session start sounds)
    fn play_custom_audio_file_once(&mut self, file_path: &str) -> Result<()> {
        let file = File::open(file_path)
            .map_err(|e| anyhow::anyhow!("Failed to open audio file {}: {}", file_path, e))?;
        let buf_reader = BufReader::new(file);
        
        let source = Decoder::new(buf_reader)
            .map_err(|e| anyhow::anyhow!("Failed to decode audio file {}: {}", file_path, e))?;

        let sink = Sink::try_new(&self.stream_handle)?;
        sink.set_volume(self.config.audio.volume);
        sink.append(source);

        // Store the sink reference but don't wait for completion
        self.current_sink = Some(Arc::new(sink));
        
        Ok(())
    }

    /// Play custom audio file with continuous looping until stopped
    fn play_custom_audio_file_continuous(&mut self, file_path: &str) -> Result<()> {
        let file = File::open(file_path)
            .map_err(|e| anyhow::anyhow!("Failed to open audio file {}: {}", file_path, e))?;
        let buf_reader = BufReader::new(file);
        
        let source = Decoder::new(buf_reader)
            .map_err(|e| anyhow::anyhow!("Failed to decode audio file {}: {}", file_path, e))?;

        let sink = Sink::try_new(&self.stream_handle)?;
        sink.set_volume(self.config.audio.volume);

        // Loop the audio continuously until stopped
        let looped_source = source.repeat_infinite();
        sink.append(looped_source);

        // Store the sink reference for later control
        self.current_sink = Some(Arc::new(sink));
        
        Ok(())
    }

    /// Play default end sound with continuous looping until stopped
    fn play_default_end_sound_continuous(&mut self) -> Result<()> {
        let sound_data = generate_notification_sound();
        
        let sink = Sink::try_new(&self.stream_handle)?;
        sink.set_volume(self.config.audio.volume);
        
        // Create a repeating source from the sound data
        let source = SineWaveSource::new(sound_data).repeat_infinite();
        sink.append(source);

        // Store the sink reference for later control
        self.current_sink = Some(Arc::new(sink));
        
        Ok(())
    }



    /// Play sound data through the audio system (non-blocking)
    fn play_sound_data_non_blocking(&mut self, sound_data: Vec<i16>) -> Result<()> {
        let sink = Sink::try_new(&self.stream_handle)?;
        sink.set_volume(self.config.audio.volume);
        
        // Convert the sound data to a source
        let source = SineWaveSource::new(sound_data);
        sink.append(source);
        
        // Store the sink reference but don't wait for completion
        self.current_sink = Some(Arc::new(sink));
        
        Ok(())
    }
}

/// Simple sine wave source for generating beep sounds
struct SineWaveSource {
    data: Vec<i16>,
    position: usize,
}

impl SineWaveSource {
    fn new(data: Vec<i16>) -> Self {
        Self { data, position: 0 }
    }
}

impl Iterator for SineWaveSource {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.data.len() {
            let sample = self.data[self.position];
            self.position += 1;
            Some(sample)
        } else {
            None
        }
    }
}

impl Source for SineWaveSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.data.len() - self.position)
    }

    fn channels(&self) -> u16 {
        1 // Mono
    }

    fn sample_rate(&self) -> u32 {
        44100 // Standard sample rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(
            self.data.len() as f32 / 44100.0,
        ))
    }
}

/// Generate a simple beep sound at the specified frequency and duration
fn generate_beep_sound(frequency: f32, duration: f32) -> Vec<i16> {
    let sample_rate = 44100.0;
    let samples = (sample_rate * duration) as usize;
    let mut sound_data = Vec::with_capacity(samples);

    for i in 0..samples {
        let t = i as f32 / sample_rate;
        let sample = (t * frequency * 2.0 * std::f32::consts::PI).sin();
        
        // Apply envelope to avoid clicks
        let envelope = if t < 0.1 {
            t / 0.1 // Fade in
        } else if t > duration - 0.1 {
            (duration - t) / 0.1 // Fade out
        } else {
            1.0
        };
        
        sound_data.push((sample * envelope * 0.3 * i16::MAX as f32) as i16);
    }

    sound_data
}

/// Generate a more complex notification sound (two-tone beep)
fn generate_notification_sound() -> Vec<i16> {
    let sample_rate = 44100.0;
    let duration = 0.6; // Total duration
    let samples = (sample_rate * duration) as usize;
    let mut sound_data = Vec::with_capacity(samples);

    for i in 0..samples {
        let t = i as f32 / sample_rate;
        
        // Two-tone beep: first tone for 0.2s, silence for 0.1s, second tone for 0.2s, silence
        let sample = if t < 0.2 {
            // First beep at 800Hz
            (t * 800.0 * 2.0 * std::f32::consts::PI).sin() * 0.7
        } else if t < 0.3 {
            // Short silence
            0.0
        } else if t < 0.5 {
            // Second beep at 1000Hz
            ((t - 0.3) * 1000.0 * 2.0 * std::f32::consts::PI).sin() * 0.7
        } else {
            // Final silence
            0.0
        };
        
        // Apply envelope to the entire sound
        let envelope = if t < 0.05 {
            t / 0.05 // Fade in
        } else if t > duration - 0.05 {
            (duration - t) / 0.05 // Fade out
        } else {
            1.0
        };
        
        sound_data.push((sample * envelope * i16::MAX as f32) as i16);
    }

    sound_data
}

