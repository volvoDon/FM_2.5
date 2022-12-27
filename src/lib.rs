use nih_plug::prelude::*;
use std::sync::Arc;
use std::f32::consts;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct FmTwo {
    params: Arc<FmTwoParams>,
    sample_rate:f32,
    op1_phase:f32,
    op2_phase:f32,
    midi_note_id:u8,
    midi_note_freq:f32,
    envelope: f32,
    envelope_index:bool,


}

#[derive(Params)]
struct FmTwoParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
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
}

impl Default for FmTwo {
    fn default() -> Self {
        Self {
            params: Arc::new(FmTwoParams::default()),
            op1_phase:0.0,
            op2_phase:0.0,
            sample_rate: 1.0,
            midi_note_id: 0,
            midi_note_freq: 1.0,
            envelope:0.0,
            envelope_index:false,
        }
    }
}

impl Default for FmTwoParams {
    fn default() -> Self {
        Self {
            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(-20.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // decibels, we need logarithmic smoothing
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            // There are many predefined formatters we can use here. If the gain was stored as
            // decibels instead of as a linear gain value, we could have also used the
            // `.with_step_size(0.1)` function to get internal rounding.
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
            // We purposely don't specify a step size here, but the parameter should still be
            // displayed as if it were rounded. This formatter also includes the unit.
            ,
            depth: FloatParam::new(
                "depth",
                0.3,
                FloatRange::Linear{
                    min: 0.0,
                    max: 8.0,
                
                },
            )
            .with_smoother(SmoothingStyle::Linear(10.0)),
            attack: FloatParam::new(
                "attack",
                0.3,
                FloatRange::Linear{
                    min: 0.005,
                    max: 0.5,
                
                },
            )
            .with_smoother(SmoothingStyle::Linear(10.0)),
            decay: FloatParam::new(
                "decay",
                0.3,
                FloatRange::Linear{
                    min: 0.0,
                    max: 0.5,
                
                },
            )
            .with_smoother(SmoothingStyle::Linear(10.0))
        }
    }
}

impl FmTwo {
    fn calculate_sine(&mut self, frequency: f32) -> f32 {
        let phase_delta = frequency / self.sample_rate;
        let sine = (self.op1_phase * consts::TAU).sin();

        self.op1_phase += phase_delta;
        if self.op1_phase >= 1.0 {
            self.op1_phase -= 1.0;
        }

        sine
    }
    fn calculate_frequency(&mut self,input_frequency:f32,frequency:f32,depth:f32) -> f32 {
        let phase_delta = (input_frequency*frequency) / self.sample_rate;
        let frequency = (self.op2_phase * consts::TAU).sin() * (depth * input_frequency);

        self.op2_phase += phase_delta;
        if self.op2_phase >= 1.0 {
            self.op2_phase -= 1.0;
        }

        frequency
    }
    fn calculate_envelope(&mut self, attack:f32,decay:f32) {
        let attack_delta = 1.0/(attack*self.sample_rate);
        let decay_delta = 1.0/(decay*self.sample_rate);

        if self.envelope < 1.0 && self.envelope_index == false {
            self.envelope += attack_delta
        }
        if self.envelope >= 1.0 {self.envelope_index = true}
        if self.envelope_index ==true && self.envelope > 0.0 {
            self.envelope -= decay_delta
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

    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
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
        self.sample_rate = buffer_config.sample_rate;
        true
    }

    fn reset(&mut self) {
        self.op1_phase = 0.0;
        self.op2_phase = 0.0;
        self.midi_note_freq = 1.0;
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
            while let Some(event) = next_event {
                if event.timing() > sample_id as u32 {
                    break;
                }

                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        self.midi_note_id = note;
                        self.midi_note_freq = util::midi_note_to_freq(note);
                        self.calculate_envelope(attack, decay)
                        
                    }
                    NoteEvent::NoteOff { note, .. } if note == self.midi_note_id => {
                        self.envelope = 0.0;
                        self.envelope_index = false;   
                    }
                    
                    _ => (),
                }

                next_event = context.next_event();
            }
            let freq = self.calculate_frequency(self.midi_note_freq, self.params.frequency.smoothed.next(),self.params.depth.smoothed.next());
            let sine = self.calculate_sine(freq);
            for sample in channel_samples {
                *sample = sine * gain * self.envelope
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
    const VST3_CATEGORIES: &'static str = "Fx|Dynamics";
}

nih_export_clap!(FmTwo);
nih_export_vst3!(FmTwo);
