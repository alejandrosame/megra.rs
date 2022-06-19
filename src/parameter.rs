pub mod modifier;
use modifier::*;

use rand::Rng;
use std::boxed::Box;
use std::fmt::*;

use ruffbox_synth::building_blocks::mod_env::{SegmentInfo, SegmentType};
use ruffbox_synth::building_blocks::{SynthParameterLabel, SynthParameterValue, ValOp};

#[derive(Clone)]
#[rustfmt::skip]
pub enum ParameterValue {
    Scalar(Parameter),
    Vector(Vec<Parameter>),
    Matrix(Vec<Vec<Parameter>>),
    Lfo(Parameter, Parameter, Parameter, Parameter, Parameter, ValOp), // init, freq, phase, amp, add, op
    LFSaw(Parameter, Parameter, Parameter, Parameter, Parameter, ValOp), // init, freq, phase, amp, add, op
    LFRSaw(Parameter, Parameter, Parameter, Parameter, Parameter, ValOp), // init, freq, phase, amp, add, op
    LFTri(Parameter, Parameter, Parameter, Parameter, Parameter, ValOp), // init, freq, phase, amp, add, op
    LFSquare(Parameter, Parameter, Parameter, Parameter, Parameter, ValOp), // init, freq, phase, pw, amp, add, op
    LinRamp(Parameter, Parameter, Parameter, ValOp),                        // from, to, time, op
    LogRamp(Parameter, Parameter, Parameter, ValOp),                        // from, to, time, op
    ExpRamp(Parameter, Parameter, Parameter, ValOp),                        // from, to, time, op
    MultiPointEnvelope(Vec<Parameter>, Vec<Parameter>, Vec<SegmentType>, bool, ValOp), // levels, times, loop, op
}

pub fn translate_stereo(val: SynthParameterValue) -> SynthParameterValue {
    match val {
        SynthParameterValue::ScalarF32(p) => SynthParameterValue::ScalarF32((p + 1.0) * 0.5),
        SynthParameterValue::Lfo(init, freq, eff_phase, amp, add, op) => {
            let pos = (init + 1.0) * 0.5;
            let amps = match *amp {
                // unbox
                SynthParameterValue::ScalarF32(a) => {
                    SynthParameterValue::ScalarF32((a + 1.0) * 0.25)
                }
                _ => translate_stereo(*amp),
            };
            let adds = (add + 1.0) * 0.5;
            let phases = (eff_phase + 1.0) * 0.5;
            SynthParameterValue::Lfo(pos, freq, phases, Box::new(amps), adds, op)
        }
        SynthParameterValue::LFSaw(init, freq, eff_phase, amp, add, op) => {
            let pos = (init + 1.0) * 0.5;
            let amps = match *amp {
                // unbox
                SynthParameterValue::ScalarF32(a) => {
                    SynthParameterValue::ScalarF32((a + 1.0) * 0.25)
                }
                _ => translate_stereo(*amp),
            };
            let adds = (add + 1.0) * 0.5;
            let phases = (eff_phase + 1.0) * 0.5;
            SynthParameterValue::LFSaw(pos, freq, phases, Box::new(amps), adds, op)
        }
        SynthParameterValue::LFRSaw(init, freq, eff_phase, amp, add, op) => {
            let pos = (init + 1.0) * 0.5;
            let amps = match *amp {
                // unbox
                SynthParameterValue::ScalarF32(a) => {
                    SynthParameterValue::ScalarF32((a + 1.0) * 0.25)
                }
                _ => translate_stereo(*amp),
            };
            let adds = (add + 1.0) * 0.5;
            let phases = (eff_phase + 1.0) * 0.5;
            SynthParameterValue::LFRSaw(pos, freq, phases, Box::new(amps), adds, op)
        }
        SynthParameterValue::LFTri(init, freq, eff_phase, amp, add, op) => {
            let pos = (init + 1.0) * 0.5;
            let amps = match *amp {
                // unbox
                SynthParameterValue::ScalarF32(a) => {
                    SynthParameterValue::ScalarF32((a + 1.0) * 0.25)
                }
                _ => translate_stereo(*amp),
            };
            let adds = (add + 1.0) * 0.5;
            let phases = (eff_phase + 1.0) * 0.5;
            SynthParameterValue::LFTri(pos, freq, phases, Box::new(amps), adds, op)
        }
        SynthParameterValue::LFSquare(init, freq, pw, amp, add, op) => {
            let pos = (init + 1.0) * 0.5;
            let amps = match *amp {
                // unbox
                SynthParameterValue::ScalarF32(a) => {
                    SynthParameterValue::ScalarF32((a + 1.0) * 0.25)
                }
                _ => translate_stereo(*amp),
            };
            let adds = (add + 1.0) * 0.5;
            SynthParameterValue::LFTri(pos, freq, pw, Box::new(amps), adds, op)
        }
        SynthParameterValue::LinRamp(from, to, time, op) => {
            SynthParameterValue::LinRamp((from + 1.0) * 0.5, (to + 1.0) * 0.5, time, op)
        }
        SynthParameterValue::LogRamp(from, to, time, op) => {
            SynthParameterValue::LinRamp((from + 1.0) * 0.5, (to + 1.0) * 0.5, time, op)
        }
        SynthParameterValue::ExpRamp(from, to, time, op) => {
            SynthParameterValue::LinRamp((from + 1.0) * 0.5, (to + 1.0) * 0.5, time, op)
        }
        SynthParameterValue::MultiPointEnvelope(segments, loop_env, op) => {
            let mut segments_translated = Vec::new();
            for seg in segments.iter() {
                segments_translated.push(SegmentInfo {
                    from: (seg.from + 1.0) * 0.5,
                    to: (seg.to + 1.0) * 0.5,
                    time: seg.time,
                    segment_type: seg.segment_type,
                });
            }
            SynthParameterValue::MultiPointEnvelope(segments_translated, loop_env, op)
        }
        _ => val,
    }
}

pub fn resolve_parameter(k: SynthParameterLabel, v: &mut ParameterValue) -> SynthParameterValue {
    match v {
        ParameterValue::Scalar(val) => {
            if k == SynthParameterLabel::SampleBufferNumber {
                val.evaluate_val_usize()
            } else {
                val.evaluate_val_f32()
            }
        }
        ParameterValue::Vector(vals) => {
            let mut static_vals: Vec<f32> = Vec::new();
            for val in vals.iter_mut() {
                static_vals.push(val.evaluate_numerical());
            }
            SynthParameterValue::VecF32(static_vals)
        }
        ParameterValue::Matrix(mat) => {
            let mut static_vals: Vec<Vec<f32>> = Vec::new();
            let mut rows = 0;
            let mut cols = 0;
            for (r, row) in mat.iter_mut().enumerate() {
                static_vals.push(Vec::new());
                rows += 1;
                if row.len() > cols {
                    cols = row.len();
                }
                for col in row.iter_mut() {
                    static_vals[r].push(col.evaluate_numerical());
                }
            }

            // make sure all rows have the same lenght
            for row in static_vals.iter_mut() {
                if row.len() < cols {
                    row.append(&mut vec![0.0; cols - row.len()])
                }
            }
            SynthParameterValue::MatrixF32((rows, cols), static_vals)
        }
        ParameterValue::Lfo(init, freq, eff_phase, amp, add, op) => SynthParameterValue::Lfo(
            init.evaluate_numerical(),
            Box::new(freq.evaluate_val_f32()),
            eff_phase.evaluate_numerical(),
            Box::new(amp.evaluate_val_f32()),
            add.evaluate_numerical(),
            *op,
        ),
        ParameterValue::LFSaw(init, freq, eff_phase, amp, add, op) => SynthParameterValue::LFSaw(
            init.evaluate_numerical(),
            Box::new(freq.evaluate_val_f32()),
            eff_phase.evaluate_numerical(),
            Box::new(amp.evaluate_val_f32()),
            add.evaluate_numerical(),
            *op,
        ),
        ParameterValue::LFRSaw(init, freq, eff_phase, amp, add, op) => SynthParameterValue::LFRSaw(
            init.evaluate_numerical(),
            Box::new(freq.evaluate_val_f32()),
            eff_phase.evaluate_numerical(),
            Box::new(amp.evaluate_val_f32()),
            add.evaluate_numerical(),
            *op,
        ),
        ParameterValue::LFTri(init, freq, eff_phase, amp, add, op) => SynthParameterValue::LFTri(
            init.evaluate_numerical(),
            Box::new(freq.evaluate_val_f32()),
            eff_phase.evaluate_numerical(),
            Box::new(amp.evaluate_val_f32()),
            add.evaluate_numerical(),
            *op,
        ),
        ParameterValue::LFSquare(init, freq, pw, amp, add, op) => SynthParameterValue::LFSquare(
            init.evaluate_numerical(),
            Box::new(freq.evaluate_val_f32()),
            pw.evaluate_numerical(),
            Box::new(amp.evaluate_val_f32()),
            add.evaluate_numerical(),
            *op,
        ),
        ParameterValue::LinRamp(from, to, time, op) => SynthParameterValue::LinRamp(
            from.evaluate_numerical(),
            to.evaluate_numerical(),
            time.evaluate_numerical(),
            *op,
        ),
        ParameterValue::LogRamp(from, to, time, op) => SynthParameterValue::LogRamp(
            from.evaluate_numerical(),
            to.evaluate_numerical(),
            time.evaluate_numerical(),
            *op,
        ),
        ParameterValue::ExpRamp(from, to, time, op) => SynthParameterValue::ExpRamp(
            from.evaluate_numerical(),
            to.evaluate_numerical(),
            time.evaluate_numerical(),
            *op,
        ),
        ParameterValue::MultiPointEnvelope(levels, times, types, loop_env, op) => {
            if levels.len() == 1 {
                SynthParameterValue::ScalarF32(levels[0].evaluate_numerical())
            } else if !levels.is_empty() {
                let mut segments = Vec::new();

                let mut levels_evaluated = Vec::new();
                let mut times_evaluated = Vec::new();

                for lvl in levels.iter_mut() {
                    levels_evaluated.push(lvl.evaluate_numerical());
                }

                for time in times.iter_mut() {
                    times_evaluated.push(time.evaluate_numerical());
                }

                let mut time = if let Some(t) = times_evaluated.get(0) {
                    *t
                } else {
                    0.2
                };
                let mut segment_type = if let Some(t) = types.get(0) {
                    *t
                } else {
                    SegmentType::Lin
                };

                for i in 0..levels_evaluated.len() {
                    let from = levels_evaluated[i];
                    if let Some(to) = levels_evaluated.get(i + 1) {
                        segments.push(SegmentInfo {
                            from,
                            to: *to,
                            time,
                            segment_type,
                        });

                        time = if let Some(t) = times_evaluated.get(i + 1) {
                            *t
                        } else {
                            time
                        };
                        segment_type = if let Some(t) = types.get(i + 1) {
                            *t
                        } else {
                            segment_type
                        };
                    }
                }

                SynthParameterValue::MultiPointEnvelope(segments, *loop_env, *op)
            } else {
                SynthParameterValue::ScalarF32(0.0)
            }
        }
    }
}

#[derive(Clone)]
pub struct Parameter {
    pub val: f32,
    pub static_val: f32,
    pub modifier: Option<Box<dyn Modifier + Send + Sync>>,
}

impl Debug for Parameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Parameter")
            .field("current", &self.val)
            .field("static", &self.static_val)
            .finish()
    }
}

impl Parameter {
    pub fn with_value(val: f32) -> Self {
        Parameter {
            val,
            static_val: val,
            modifier: None,
        }
    }

    pub fn evaluate_val_f32(&mut self) -> SynthParameterValue {
        SynthParameterValue::ScalarF32(if let Some(m) = &mut self.modifier {
            self.static_val = m.evaluate(self.val);
            self.static_val
        } else {
            self.val
        })
    }

    pub fn evaluate_val_usize(&mut self) -> SynthParameterValue {
        SynthParameterValue::ScalarUsize(if let Some(m) = &mut self.modifier {
            self.static_val = m.evaluate(self.val);
            self.static_val as usize
        } else {
            self.val as usize
        })
    }

    pub fn evaluate_numerical(&mut self) -> f32 {
        if let Some(m) = &mut self.modifier {
            self.static_val = m.evaluate(self.val);
            self.static_val
        } else {
            self.val
        }
    }

    pub fn shake(&mut self, mut factor: f32) {
        factor = factor.clamp(0.0, 1.0);
        let mut rng = rand::thread_rng();
        // heuristic ... from old megra ... not sure what i thought back then, let's see ...
        let rand = (factor * (1000.0 - rng.gen_range(0.0..2000.0))) * (self.val / 1000.0);
        self.val += rand;
        if let Some(m) = self.modifier.as_mut() {
            m.shake(factor);
        }
    }
}

// TEST TEST TEST
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_shake() {
        for _ in 0..20 {
            let mut a = Parameter::with_value(1000.0);
            a.shake(0.5);
            println!("val after shake: {}", a.evaluate_numerical());
            assert!(a.evaluate_numerical() != 1000.0);
            assert!(a.evaluate_numerical() >= 500.0);
            assert!(a.evaluate_numerical() <= 1500.0);
        }
    }
}
