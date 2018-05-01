extern crate pcm;

use pcm::Frame;
use pcm::Sample;
use pcm::PCM;
use std::fs::File;
use std::io::{BufReader, BufWriter};

fn main() {
    println!("Opening file...");
    let input_wave_reader = &mut BufReader::new(File::open("test_files/input.wav").unwrap());
    println!("Importing...");
    let input_pcm = PCM::import_wave_file(input_wave_reader).unwrap();
    println!("Deriving...");
    let mut pcm_out_channels = Vec::new();
    for channel in 0..input_pcm.parameters.nb_channels {
        let mut pcm_out = Vec::new();
        let mut previous_sample = None;
        let mut current_sample = Some(input_pcm.frames[0usize].samples[channel as usize].clone());
        let mut next_sample;
        for current_frame_id in 0..input_pcm.frames.len() {
            next_sample = match input_pcm.frames.get(current_frame_id + 1) {
                None => None,
                Some(f) => Some(f.samples[channel as usize].clone()),
            };
            match previous_sample.clone() {
                None => {}
                Some(ps) => match next_sample.clone() {
                    None => {}
                    Some(ns) => match ps {
                        Sample::Unsigned8bits(psv) => match ns {
                            Sample::Unsigned8bits(nsv) => {
                                pcm_out.push(Sample::Unsigned8bits(nsv - psv));
                            }
                            _ => panic!(),
                        },
                        Sample::Signed16bits(psv) => match ns {
                            Sample::Signed16bits(nsv) => {
                                pcm_out.push(Sample::Signed16bits(nsv - psv));
                            }
                            _ => panic!(),
                        },
                        _ => unimplemented!(),
                    },
                },
            }
            previous_sample = current_sample.clone();
            current_sample = next_sample.clone();
        }
        pcm_out_channels.push(pcm_out);
    }
    println!("Reconverting...");
    let mut frames = Vec::new();
    for current_frame_id in 0..pcm_out_channels[0usize].len() {
        let mut temp = Vec::new();
        for channel_id in 0..input_pcm.parameters.nb_channels {
            temp.push(pcm_out_channels[channel_id as usize][current_frame_id as usize].clone());
        }
        frames.push(Frame { samples: temp });
    }
    let out_pcm = PCM {
        parameters: input_pcm.parameters,
        loop_info: input_pcm.loop_info,
        frames,
    };
    println!("Writing File...");
    let output_wave_writer =
        &mut BufWriter::new(File::create("test_files/output_derived.wav").unwrap());
    out_pcm.export_wave_file(output_wave_writer).unwrap();
}
