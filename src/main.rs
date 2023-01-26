
use rodio::Sink;
use rodio::source::SineWave;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, KeyboardState};
use sdl2::video::WindowContext;
use sdl2::rect::Rect;
use sdl2::render::{WindowCanvas, TextureCreator};
use sdl2::ttf;
use sdl2::keyboard::Scancode;


use rodio::{OutputStream, source::Source};

use std::time::Duration;
use std::path::Path;

use enum_iterator::{all, cardinality, first, last, next, previous, reverse_all, Sequence};
use enum_map::{enum_map, Enum, EnumMap};

//////////////////////////////////////////
const SampleRate: usize = 44100;
const DesiredFreq: usize = 30;
const waveformSampleCount: usize = SampleRate/ DesiredFreq;

//for now just populated with sin vals
/* fn populate_tables(wavetable: &mut wavetable) -> Result<(), String> {
    for i in 0..waveformSampleCount {
        wavetable.samples.push((i as f32 * 2.0 * std::f32::consts::PI as f32/ waveformSampleCount as f32).sin());

    }

    Ok(())
}

pub struct Wavetable_settings {
    pub waveform_length: usize,
    pub dimension_count: usize,
    pub waveforms_per_dim: usize,
    pub base_freq: f32,
}

pub struct wavetable{
    pub settings: Wavetable_settings,
    pub samples: Vec<f32>,
}

pub struct Wavetable_handle {
    pub table: &'static mut wavetable,

    pub sample_ix: f32,

    pub mixes: Vec<f32>,

    pub mixes_for_sample : Vec<f32>,

    pub sample_buffer: Vec<f32>,

    pub frequency_buffer: Vec<f32>,
} */

struct WavetableOscillator {
    sample_rate: u32,
    wave_table: Vec<f32>,
    index: f32,
    index_increment: f32,
}

impl WavetableOscillator {
    fn new(sample_rate: u32, wave_table: Vec<f32>) -> WavetableOscillator {
        return WavetableOscillator {
            sample_rate: sample_rate,
            wave_table: wave_table,
            index: 0.0,
            index_increment: 0.0,
        };
    }
    
    fn set_frequency(&mut self, frequency: f32) {
        self.index_increment = frequency * self.wave_table.len() as f32 
                               / self.sample_rate as f32;
    }

    fn get_sample(&mut self) -> f32 {
        let sample = self.lerp();
        self.index += self.index_increment;
        self.index %= self.wave_table.len() as f32;
        return sample;
    }

    fn lerp(&self) -> f32 {
        let truncated_index = self.index as usize;
        let next_index = (truncated_index + 1) % self.wave_table.len();
        
        let next_index_weight = self.index - truncated_index as f32;
        let truncated_index_weight = 1.0 - next_index_weight;

        return truncated_index_weight * self.wave_table[truncated_index] 
               + next_index_weight * self.wave_table[next_index];
    }
} 

impl Source for WavetableOscillator {
    fn channels(&self) -> u16 {
        return 1;
    }

    fn sample_rate(&self) -> u32 {
        return self.sample_rate;
    }   

    fn current_frame_len(&self) -> Option<usize> {
        return None;
    }

    fn total_duration(&self) -> Option<Duration> {
        return None;
    }
}

impl Iterator for WavetableOscillator {
    type Item = f32;
    
    fn next(&mut self) -> Option<Self::Item> {
        return Some(self.get_sample());
    }
}



fn render(canvas: &mut WindowCanvas, color: Color, font: &sdl2::ttf::Font, texture_creator: &TextureCreator<WindowContext>, txt_inj: &mut String) -> Result<(), String> {
    canvas.set_draw_color(color);
    canvas.clear();
    

    //text
    
    if (txt_inj.len() >0){
    let surface = font.render(&txt_inj)
    .blended(Color::RGBA(255, 0, 0, 128))
    .map_err(|e| e.to_string())?;

    let texture = texture_creator
    .create_texture_from_surface(&surface)
    .map_err(|e| e.to_string())?;

    let target = Rect::new(10 as i32, 0 as i32, 200 as u32, 100 as u32);
    canvas.copy(&texture, None, Some(target))?;
    }

    if txt_inj.len() > 30 {
        txt_inj.clear();
    }
    
    canvas.present();
    Ok(())
}



fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    
    
    //prep fonts
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?; 
    let font_path: &Path = Path::new(&"fonts/OpenSans-BoldItalic.ttf");
    let mut font = ttf_context.load_font(font_path, 128)?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);
    
    let window = video_subsystem.window("game window", 600, 800)
    .position_centered()
    .build()
    .expect("could not create video subsystem");

    let mut canvas = window.into_canvas().build()
    .expect("could not create canvas");
    
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump()?;
    
    let mut text: String = "".to_string();

    let wave_table_size = 64;
    let mut wave_table: Vec<f32> = Vec::with_capacity(wave_table_size);
    
    for n in 0..wave_table_size {
        wave_table.push((2.0 * std::f32::consts::PI * n as f32 / wave_table_size as f32).sin());
    }
    
    let mut oscillator = WavetableOscillator::new(44100, wave_table);
    
    oscillator.set_frequency(440.0);
    text = "".to_string();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let sink = Sink::try_new(&stream_handle).unwrap();

    
    
    
    sink.append(oscillator.convert_samples());

    


    

  

    let mut i =0;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running;
                },
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                //left
                Event::KeyDown { timestamp, window_id, keycode, scancode, keymod, repeat } =>{
                    println!("{}",timestamp);
                    println!("{}",keycode.unwrap().to_string());
                    text.push_str(&keycode.unwrap().to_string());
                    
                    
                    
                },

                Event::KeyUp { timestamp, window_id, keycode, scancode, keymod, repeat } => {
                   sink.pause(); 
                },
                _ => {} 
            }
        }
    

        i = (i+1) %255;

        render(&mut canvas, Color::RGB(i,64, 255-i),&font,&texture_creator, &mut text)?;

        std::thread::sleep(Duration::new(0,1_000_000_000u32 / 60));
        
    }
    
    Ok(())
}