use crate::event::*;
use crate::generator::{GenModFun, Generator};
use crate::generator_processor::GeneratorProcessor;
use crate::parameter::*;
use dashmap::DashMap;
use std::collections::{BTreeSet, HashMap};

use ruffbox_synth::building_blocks::SynthParameterLabel;

#[derive(Clone)]
pub enum Part {
    Combined(Vec<Generator>, Vec<PartProxy>),
}

pub type PartsStore = HashMap<String, Part>;

// might be unified with event parameters at some point but
// i'm not sure how yet ...
#[derive(Clone)]
pub enum ConfigParameter {
    Numeric(f32),
    Dynamic(DynVal),
    Symbolic(String),
}

// only one so far
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum BuiltinGlobalParameters {
    LifemodelGlobalResources,
    GlobalTimeModifier,
    GlobalLatency, // latency between language and dsp
    DefaultDuration,
}

pub type GlobalParameters = DashMap<BuiltinGlobalParameters, ConfigParameter>;

#[derive(Clone)]
pub enum Command {
    Clear,                                                               // clear the entire session
    Tmod(DynVal),         // set global time mod parameter
    Latency(DynVal),      // set global latency parameter
    Bpm(f32),             // set default tempo in bpm
    DefaultDuration(f32), // set default duration in milliseconds
    GlobRes(f32),         // global resources for lifemodel algorithm
    GlobalRuffboxParams(HashMap<SynthParameterLabel, ParameterValue>), // global ruffbox params
    LoadSampleAsWavematrix(String, String, String, (usize, usize), f32), // key, path, method, matrix size, start
    LoadSample(String, Vec<String>, String, bool), // set (events), keyword, path, downmix_stereo
    LoadSampleSet(String, bool),                   // set path, downmix stereo
    LoadSampleSets(String, bool),                  // top level sets set path
    LoadPart((String, Part)),                      // set (events), keyword, path
    StepPart(String),                              // step through specified path
    FreezeBuffer(usize, usize),                    // freeze live buffer
    ExportDotStatic((String, Generator)),          // filename, generator
    ExportDotRunning((String, BTreeSet<String>)),  // filename, generator id
    ExportDotPart((String, String)),               // filename, part name
    Once((Vec<StaticEvent>, Vec<ControlEvent>)),   // execute event(s) once
    ConnectVisualizer,                             // connect visualizer
    StartRecording(Option<String>, bool),          // start recording, prefix, input
    StopRecording,                                 // stop recording ...
}

#[derive(Clone)]
pub enum PartProxy {
    // part, mods
    Proxy(String, Vec<GeneratorProcessorOrModifier>),
}

#[derive(Clone)]
pub enum GeneratorProcessorOrModifier {
    GeneratorProcessor(Box<dyn GeneratorProcessor + Send>),
    GeneratorModifierFunction(
        (
            GenModFun,
            Vec<ConfigParameter>,
            HashMap<String, ConfigParameter>,
        ),
    ),
}
