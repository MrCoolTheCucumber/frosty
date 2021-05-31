use std::{sync::{mpsc::Receiver}, time::Duration};

use sdl2::{AudioSubsystem, audio::{AudioCallback, AudioDevice, AudioSpecDesired}};

pub type Sample = u8;
pub type SampleBuffer = [Sample; SAMPLES_PER_BUFFER];
pub const SAMPLES_PER_BUFFER: usize = 512;
pub const SAMPLE_RATE: u32 = 48000;

struct Sound {
    receiver: Receiver<SampleBuffer>
}

impl AudioCallback for Sound {
    type Channel = u8;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        // self.receiver.recv_timeout(Duration::from_nanos(1000000000 / (SAMPLE_RATE) as u64))
        match self.receiver.recv() {
            Ok(buf) => {
                let mut i: usize = 0;
                for sample in out.iter_mut() {
                    *sample = (buf[i] * (u8::MAX / 15)) / 32;
                    i += 1;
                }
            }

            Err(_e) => {
                // println!("Tried to recieve buffer in sample callback but failed?\n {:?}", e);
                println!("Sending blank samples.");

                for sample in out.iter_mut() {
                    *sample = 0;
                }
            }
        }
    }
}

// TODO: callback must not have a blocking function in it, think of a better way
//       probably with threads and a shared resource

pub struct Audio {
    device: AudioDevice<Sound>
}

impl Audio {
    pub fn new(audio: &AudioSubsystem, receiver: Receiver<SampleBuffer>) -> Self {
        let spec = AudioSpecDesired {
            freq: Some(SAMPLE_RATE as i32),
            channels: Some(1),
            samples: Some(SAMPLES_PER_BUFFER as u16)
        };

        let sound = Sound {
            receiver
        };

        let device = match AudioDevice::open_playback(audio, None, &spec, |_| sound) {
            Ok(d) => d,
            Err(e) => panic!("{:?}", e)
        };

        Self {
            device
        }
    }

    pub fn resume(&self) {
        self.device.resume();
    }

    pub fn pause(&self) {
        self.device.pause();
    }
}