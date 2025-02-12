use std::collections::HashMap;
use std::time::Instant;

use rodio::source::Source;
use rodio::Sink;

mod oscillator;
pub use oscillator::Oscillator;

struct EnvelopeState {
    envelope: Envelope,
    start_time: Instant,
    is_releasing: bool,
    release_start_time: Option<Instant>,
    release_start_volume: Option<f32>, // Track starting volume for release
}

struct Envelope {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}

impl Envelope {
    fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Envelope {
        Envelope {
            attack,
            decay,
            sustain,
            release,
        }
    }
}

pub struct Synth {
    audio_sinks: HashMap<u8, Sink>,
    envelope_states: HashMap<u8, EnvelopeState>,
    stream_handle: rodio::OutputStreamHandle,
}

impl Synth {
    pub fn new(stream_handle: rodio::OutputStreamHandle) -> Synth {
        Synth {
            audio_sinks: HashMap::new(),
            envelope_states: HashMap::new(),
            stream_handle,
        }
    }

    pub fn play_source(&mut self, audio_source: Box<dyn Source<Item = f32> + Send>, source_id: u8) {
        let sink = Sink::try_new(&self.stream_handle).expect("Failed to create sink");
        sink.append(audio_source);

        let envelope = Envelope::new(0.8, 0.2, 0.7, 1.3);
        let envelope_state = EnvelopeState {
            envelope,
            start_time: Instant::now(),
            is_releasing: false,
            release_start_time: None,
            release_start_volume: None,
        };

        self.audio_sinks.insert(source_id, sink);
        self.envelope_states.insert(source_id, envelope_state);
    }

    pub fn release_source(&mut self, source_id: u8) {
        if let Some(envelope_state) = self.envelope_states.get_mut(&source_id) {
            let now = Instant::now();
            let elapsed = now.duration_since(envelope_state.start_time).as_secs_f32();
            let envelope = &envelope_state.envelope;

            // Calculate current volume at release time
            let current_volume = if elapsed < envelope.attack {
                // Attack phase
                elapsed / envelope.attack
            } else if elapsed < envelope.attack + envelope.decay {
                // Decay phase
                1.0 - (elapsed - envelope.attack) / envelope.decay * (1.0 - envelope.sustain)
            } else {
                // Sustain phase
                envelope.sustain
            };

            envelope_state.is_releasing = true;
            envelope_state.release_start_time = Some(now);
            envelope_state.release_start_volume = Some(current_volume);
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let mut to_remove = Vec::new();

        for (source_id, envelope_state) in self.envelope_states.iter_mut() {
            let sink = self.audio_sinks.get_mut(source_id).unwrap();
            let envelope = &envelope_state.envelope;

            let volume = if envelope_state.is_releasing {
                // Release phase - use captured start volume and release time
                let elapsed_release = now
                    .duration_since(envelope_state.release_start_time.unwrap())
                    .as_secs_f32();

                let start_volume = envelope_state.release_start_volume.unwrap();
                let t = (elapsed_release / envelope.release).min(1.0);
                start_volume * (1.0 - t)
            } else {
                // Calculate based on ADSR phases
                let elapsed = now.duration_since(envelope_state.start_time).as_secs_f32();

                if elapsed < envelope.attack {
                    // Attack phase
                    elapsed / envelope.attack
                } else if elapsed < envelope.attack + envelope.decay {
                    // Decay phase
                    1.0 - (elapsed - envelope.attack) / envelope.decay * (1.0 - envelope.sustain)
                } else {
                    // Sustain phase
                    envelope.sustain
                }
            };

            sink.set_volume(volume);

            // Check if release is complete
            if envelope_state.is_releasing {
                let elapsed_release = now
                    .duration_since(envelope_state.release_start_time.unwrap())
                    .as_secs_f32();

                if elapsed_release >= envelope.release {
                    to_remove.push(*source_id);
                }
            }
        }

        // Cleanup completed sounds
        for source_id in to_remove {
            self.envelope_states.remove(&source_id);
            self.audio_sinks.remove(&source_id);
        }
    }
}
