// use midir::MidiInput;
use rodio::source::Source;
use rodio::OutputStream;
use std::sync::{Arc, Mutex};

mod keyboard;
mod oscillator;
mod synth;

use keyboard::KeyBoard;
use oscillator::Oscillator;
use synth::Synth;

use rui::*;

fn main() {
    // Create a new synth
    let (_stream, stream_handle) = OutputStream::try_default().unwrap(); // Oddly, this cant be done in the new function, otherwise the program will panic
                                                                         // The synth will manage multiple audio sinks and their envelopes
    let synth = Arc::new(Mutex::new(Synth::new(stream_handle)));

    //   let hz = 440.0 * 2.0_f32.powf((message[1] as f32 - 69.0) / 12.0);
    //             let pressure = message[2] as f32 / 127.0;

    //             let mut synth = synth_clone.lock().unwrap();

    //             if message[0] == 144 {
    //                 // 144 is the event for note on
    //                 // Create a new audio source from the oscillator
    //                 let audio_source = Oscillator::square_wave(hz).amplify(pressure);
    //                 // Play the audio source
    //                 synth.play_source(Box::new(audio_source), message[1]);
    //             }
    //             if message[0] == 128 {
    //                 // 128 is the event for note off
    //                 // Signals the envelope to start releasing
    //                 // The sink is automatically deleted when the envelope is done releasing
    //                 synth.release_source(message[1]);
    //             }

    let synth_clone = synth.clone();

    KeyBoard::new()
        .num_keys(25)
        .show()
        .size([400.0, 200.0])
        .key(move |cx, k| match k {
            Key::Character(c) => match c {
                'a' => {
                    let mut synth = synth_clone.lock().unwrap();
                    let audio_source = Oscillator::sine_wave(440.0).amplify(1.0);
                    synth.play_source(Box::new(audio_source), 0);
                }
                's' => {
                    let mut synth = synth_clone.lock().unwrap();
                    let audio_source = Oscillator::square_wave(440.0).amplify(1.0);
                    synth.play_source(Box::new(audio_source), 1);
                }
                'd' => {
                    let mut synth = synth_clone.lock().unwrap();
                    let audio_source = Oscillator::sawtooth_wave(440.0).amplify(1.0);
                    synth.play_source(Box::new(audio_source), 2);
                }
                'f' => {
                    let mut synth = synth_clone.lock().unwrap();
                    let audio_source = Oscillator::triangle_wave(440.0).amplify(1.0);
                    synth.play_source(Box::new(audio_source), 3);
                }
                _ => (),
            },
            _ => (),
        })
        .anim(move |_, _| {
            let mut synth = synth.lock().unwrap();
            synth.update();
        })
        .run();
}
