use fm_voice::fm;
use nih_plug::prelude::*;
use std::sync::Arc;

mod fm_voice;


struct FmTwo {
    params: Arc<FmTwoParams>,
    voice1: fm::Voice,
    voice2: fm::Voice
}

#[derive(Params)]
struct FmTwoParams {
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "frequency"]
    pub frequency: FloatParam,
    #[id = "depth"]
    pub depth: FloatParam,
    #[id = "attack"]
    pub attack: FloatParam,
    #[id = "decay"]
    pub decay: FloatParam,
    #[id = "sustain"]
    pub sustain: FloatParam,
    #[id = "release"]
    pub release: FloatParam,
}

impl Default for FmTwo {
    fn default() -> Self {
        Self {
            params: Arc::new(FmTwoParams::default()),
            voice1: fm::Voice::new(),
            voice2: fm::Voice::new()
        }
    }
}

impl Default for FmTwoParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(-20.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            frequency: FloatParam::new(
                "Frequency",
                1.0,
                FloatRange::Skewed {
                    min: 0.3,
                    max: 1.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_smoother(SmoothingStyle::Linear(10.0))
            ,
            depth: FloatParam::new(
                "depth",
                0.3,
                FloatRange::Linear{
                    min: 0.0,
                    max: 12.0,
                
                },
            )
            .with_smoother(SmoothingStyle::Linear(10.0)),
            attack: FloatParam::new(
                "attack",
                0.3,
                FloatRange::Linear{
                    min: 0.0,
                    max: 1.0,
                
                },
            )
            .with_smoother(SmoothingStyle::Linear(10.0)),
            decay: FloatParam::new(
                "decay",
                0.3,
                FloatRange::Linear{
                    min: 0.0,
                    max: 1.0,
                
                },
            )
            .with_smoother(SmoothingStyle::Linear(10.0)),
            sustain: FloatParam::new(
                "sustain",
                0.5,
                FloatRange::Linear{
                    min: 0.0,
                    max: 1.0,
                
                },
            )
            .with_smoother(SmoothingStyle::Linear(10.0)),
            release: FloatParam::new(
                "release",
                0.5,
                FloatRange::Linear{
                    min: 0.0,
                    max: 1.0,
                
                },
            )
            .with_smoother(SmoothingStyle::Linear(10.0)),
        }
    }
}



impl Plugin for FmTwo {
    const NAME: &'static str = "Fm Two";
    const VENDOR: &'static str = "volvoDon";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "segalcsam@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const DEFAULT_INPUT_CHANNELS: u32 = 0;
    const DEFAULT_OUTPUT_CHANNELS: u32 = 2;

    const DEFAULT_AUX_INPUTS: Option<AuxiliaryIOConfig> = None;
    const DEFAULT_AUX_OUTPUTS: Option<AuxiliaryIOConfig> = None;

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn accepts_bus_config(&self, config: &BusConfig) -> bool {
        // This works with any symmetrical IO layout
        config.num_input_channels == config.num_output_channels && config.num_input_channels > 0
    }

    fn initialize(
        &mut self,
        _bus_config: &BusConfig,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.voice1.sample_rate = buffer_config.sample_rate;
        self.voice2.sample_rate = buffer_config.sample_rate;
        true
    }

    fn reset(&mut self) {
        self.voice1.reset()
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut next_event = context.next_event();
        for (sample_id, channel_samples) in buffer.iter_samples().enumerate() {
            let gain = self.params.gain.smoothed.next();
            let attack = self.params.attack.smoothed.next();
            let decay = self.params.decay.smoothed.next();
            let sustain = self.params.sustain.smoothed.next();
            let release = self.params.release.smoothed.next();
            while let Some(event) = next_event {
                if event.timing() > sample_id as u32 {
                    break;
                }

                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        if self.voice1.envelope_index == 4 || self.voice1.midi_note_id == note {
                            self.voice1.midi_note_id = note;
                            self.voice1.midi_note_freq = util::midi_note_to_freq(note);
                            self.voice1.envelope_index = 0;
                        }
                        else if self.voice2.envelope_index == 4 || self.voice2.midi_note_id == note {
                            self.voice2.midi_note_id = note;
                            self.voice2.midi_note_freq = util::midi_note_to_freq(note);
                            self.voice2.envelope_index = 0;    
                        } else {
                            self.voice1.midi_note_id = note;
                            self.voice1.midi_note_freq = util::midi_note_to_freq(note);
                            self.voice1.envelope_index = 0;  
                        }
                        
                    }
                    NoteEvent::NoteOff { note, .. } => {
                        if self.voice1.midi_note_id == note {self.voice1.envelope_index = 3}
                        if self.voice2.midi_note_id == note {self.voice2.envelope_index = 3};   
                    }
                    
                    _ => (),
                }

                next_event = context.next_event();
            }
            self.voice1.calculate_envelope(attack, decay, sustain, release);
            self.voice2.calculate_envelope(attack, decay, sustain, release);
            let freq1 = self.voice1.calculate_frequency(self.voice1.midi_note_freq, self.params.frequency.smoothed.next(),self.params.depth.smoothed.next());
            let freq2 = self.voice1.calculate_frequency(self.voice2.midi_note_freq, self.params.frequency.smoothed.next(),self.params.depth.smoothed.next());
            let sine1 = self.voice1.calculate_sine(freq1);
            let sine2 = self.voice1.calculate_sine(freq2);
            for sample in channel_samples {
                *sample = ((sine1 * self.voice1.envelope) + (sine2*self.voice2.envelope)) * gain
            }

        }

        

        ProcessStatus::KeepAlive
    }
}

impl ClapPlugin for FmTwo {
    const CLAP_ID: &'static str = "com.dogvomit.FM-two";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("simple fm synth mostly for percusive sounds");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for FmTwo {
    const VST3_CLASS_ID: [u8; 16] = *b"dgvomtFMtwoSynth";

    // And don't forget to change these categories, see the docstring on `VST3_CATEGORIES` for more
    // information
    const VST3_CATEGORIES: &'static str = "Instrument|Synth";
}

nih_export_clap!(FmTwo);
nih_export_vst3!(FmTwo);
