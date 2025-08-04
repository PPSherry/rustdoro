use anyhow::Result;
use rodio::{source::Source, OutputStream, OutputStreamHandle, Sink};

/// Audio notification manager
pub struct NotificationManager {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    enabled: bool,
}

impl NotificationManager {
    /// Create a new notification manager
    pub fn new(enabled: bool) -> Result<Self> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        
        Ok(Self {
            _stream,
            stream_handle,
            enabled,
        })
    }

    /// Enable or disable sound notifications
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if sound notifications are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Play session start sound
    pub fn play_start_sound(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Generate a simple beep sound for session start
        let sound_data = generate_beep_sound(800.0, 0.3); // 800Hz for 0.3 seconds
        self.play_sound_data(sound_data)
    }

    /// Play session end sound
    pub fn play_end_sound(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Generate a different beep sound for session end
        let sound_data = generate_notification_sound(); // More complex sound
        self.play_sound_data(sound_data)
    }

    /// Play work session start sound
    pub fn play_work_start_sound(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let sound_data = generate_beep_sound(600.0, 0.2); // Lower frequency for work
        self.play_sound_data(sound_data)
    }

    /// Play break start sound
    pub fn play_break_start_sound(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let sound_data = generate_beep_sound(900.0, 0.2); // Higher frequency for break
        self.play_sound_data(sound_data)
    }

    /// Play generic notification sound
    pub fn play_notification(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let sound_data = generate_beep_sound(700.0, 0.1); // Short beep
        self.play_sound_data(sound_data)
    }

    /// Play sound data through the audio system
    fn play_sound_data(&self, sound_data: Vec<i16>) -> Result<()> {
        let sink = Sink::try_new(&self.stream_handle)?;
        
        // Convert the sound data to a source
        let source = SineWaveSource::new(sound_data);
        sink.append(source);
        
        // Wait for the sound to finish playing
        sink.sleep_until_end();
        
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

    fn total_duration(&self) -> Option<std::time::Duration> {
        Some(std::time::Duration::from_secs_f32(
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

/// Generate a triple beep sound for special events
pub fn generate_triple_beep() -> Vec<i16> {
    let sample_rate = 44100.0;
    let duration = 1.0; // Total duration
    let samples = (sample_rate * duration) as usize;
    let mut sound_data = Vec::with_capacity(samples);

    for i in 0..samples {
        let t = i as f32 / sample_rate;
        
        // Three beeps with pauses
        let sample = if t < 0.15 {
            // First beep
            (t * 800.0 * 2.0 * std::f32::consts::PI).sin() * 0.6
        } else if t < 0.25 {
            // Pause
            0.0
        } else if t < 0.4 {
            // Second beep
            ((t - 0.25) * 800.0 * 2.0 * std::f32::consts::PI).sin() * 0.6
        } else if t < 0.5 {
            // Pause
            0.0
        } else if t < 0.65 {
            // Third beep
            ((t - 0.5) * 800.0 * 2.0 * std::f32::consts::PI).sin() * 0.6
        } else {
            // Final silence
            0.0
        };
        
        sound_data.push((sample * i16::MAX as f32) as i16);
    }

    sound_data
} 