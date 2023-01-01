pub mod fm
{
use std::f32::consts;
pub struct Voice {
    pub sample_rate:f32,
    pub op1_phase:f32,
    pub op2_phase:f32,
    pub midi_note_id:u8,
    pub midi_note_freq:f32,
    pub envelope: f32,
    pub envelope_index:u8,
}
impl Voice {
    pub fn new () -> Voice {
        Voice{
            sample_rate: 1.0,
            op1_phase:0.0,
            op2_phase:0.0,
            midi_note_id:0,
            midi_note_freq:440.0,
            envelope: 0.0,
            envelope_index:4,
        }
    }
    pub fn reset (&mut self) {
        self.op1_phase = 0.0;
        self.op2_phase = 0.0;
        self.midi_note_freq = 1.0;
        self.envelope_index = 3;
    }
    pub fn calculate_sine(&mut self, frequency: f32) -> f32 {
        let phase_delta = frequency / self.sample_rate;
        let sine = (self.op1_phase * consts::TAU).sin();

        self.op1_phase += phase_delta;
        if self.op1_phase >= 1.0 {
            self.op1_phase -= 1.0;
        }

        sine
    }
    pub fn calculate_frequency(&mut self,input_frequency:f32,frequency:f32,depth:f32) -> f32 {
        let phase_delta = (input_frequency*frequency) / self.sample_rate;
        let frequency = (self.op2_phase * consts::TAU).sin() * (depth * input_frequency);

        self.op2_phase += phase_delta;
        if self.op2_phase >= 1.0 {
            self.op2_phase -= 1.0;
        }

        frequency
    }
    pub fn calculate_envelope(&mut self, attack:f32,decay:f32,sustain:f32,release:f32) {
        let attack_delta = 1.0/(attack*self.sample_rate);
        let decay_delta = 1.0/(decay*self.sample_rate);
        let release_delta = 1.0/(release*self.sample_rate);

        if self.envelope_index == 0 {
            self.envelope += attack_delta;
            if self.envelope >= 1.0 {self.envelope_index += 1}
        }
        if self.envelope_index == 1 {
            self.envelope -= decay_delta;
            if self.envelope <= sustain {self.envelope_index += 1; self.envelope = sustain}
        }
        if self.envelope_index == 2 {self.envelope = sustain}
        if self.envelope_index == 3 {
            self.envelope -= release_delta;
            if self.envelope <= 0.0 {self.envelope = 0.0;self.envelope_index += 1}
        }



    }
}
}