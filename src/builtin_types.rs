use crate::event::*;
use crate::generator::{GenModFun, Generator};
use crate::generator_processor::GeneratorProcessor;
use crate::markov_sequence_generator::{MarkovSequenceGenerator, Rule};
use crate::parameter::*;
use crate::session::SyncContext;
use dashmap::DashMap;
use std::collections::HashMap;

use ruffbox_synth::ruffbox::synth::SynthParameter;

pub enum Part {
    Combined(Vec<Generator>, Vec<PartProxy>),
}

pub type PartsStore = HashMap<String, Part>;

// might be unified with event parameters at some point but
// i'm not sure how yet ...
#[derive(Clone)]
pub enum ConfigParameter {
    Numeric(f32),
    Dynamic(Parameter),
    Symbolic(String),
}

// only one so far
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum BuiltinGlobalParameters {
    LifemodelGlobalResources,
    GlobalTimeModifier,
}

pub type GlobalParameters = DashMap<BuiltinGlobalParameters, ConfigParameter>;

// reflect event hierarchy here, like, Tuned, Param, Sample, Noise ?
pub enum BuiltInParameterEvent {
    PitchFrequency(EventOperation),
    Attack(EventOperation),
    Release(EventOperation),
    Sustain(EventOperation),
    ChannelPosition(EventOperation),
    Level(EventOperation),
    Duration(EventOperation),
    Reverb(EventOperation),
    Delay(EventOperation),
    LpFreq(EventOperation),
    LpQ(EventOperation),
    LpDist(EventOperation),
    PeakFreq(EventOperation),
    PeakQ(EventOperation),
    PeakGain(EventOperation),
    Pulsewidth(EventOperation),
    PlaybackStart(EventOperation),
    PlaybackRate(EventOperation),
}

pub enum BuiltInSoundEvent {
    Sine(EventOperation),
    Saw(EventOperation),
    Square(EventOperation),
}

pub enum BuiltInDynamicParameter {
    Bounce,
    Brownian,
    Envelope,
    Fade,
    RandRange, //Oscil,
}

pub enum BuiltInGenProc {
    Pear,
    Apple,
    Every,
    Lifemodel,
}

pub enum BuiltInGenModFun {
    Haste,
    Relax,
    Grow,
    Shrink,
    Blur,
    Sharpen,
    Shake,
    Skip,
    Rewind,
}

#[derive(Clone, Copy)]
pub enum BuiltInMultiplexer {
    XDup,
    XSpread,
    //XBounce,
    //XRot
}

/// constructor for generators ...
pub enum BuiltInConstructor {
    Learn,
    Infer,
    Rule,
    Nucleus,
    Cycle,
    Fully,
    Flower,
    Friendship,
    // Chop,
    // Pseq, ?
}

pub enum BuiltInCommand {
    Clear,
    Tmod,
    GlobRes,
    Delay,
    Reverb,
    ExportDot,
    LoadSample,
    LoadSampleSets,
    LoadSampleSet,
    LoadPart,
    Once,
}

/// As this doesn't strive to be a turing-complete lisp, we'll start with the basic
/// megra operations, learning and inferring, plus the built-in events
pub enum BuiltIn {
    Constructor(BuiltInConstructor),
    Silence,
    Command(BuiltInCommand),
    SyncContext,
    Parameter(BuiltInDynamicParameter),
    SoundEvent(BuiltInSoundEvent),
    ControlEvent,
    ParameterEvent(BuiltInParameterEvent),
    GenProc(BuiltInGenProc),
    GenModFun(BuiltInGenModFun),
    Multiplexer(BuiltInMultiplexer),
}

pub enum Command {
    Clear,                                             // clear the entire session
    Tmod(Parameter),                                   // set global time mod parameter
    GlobRes(f32),                                      // global resources for lifemodel algorithm
    GlobalRuffboxParams(HashMap<SynthParameter, f32>), // global ruffbox params
    LoadSample((String, Vec<String>, String)),         // set (events), keyword, path
    LoadSampleSet(String),                             // set path
    LoadSampleSets(String),                            // top level sets set path
    LoadPart((String, Part)),                          // set (events), keyword, path
    ExportDot((String, Generator)),                    // filename, generator
    Once((Vec<Event>, Vec<ControlEvent>)),
}

#[derive(Clone)]
pub enum PartProxy {
    // part, mods
    Proxy(String, Vec<Box<dyn GeneratorProcessor + Send>>),
}

pub enum Atom {
    // atom might not be the right word any longer
    Float(f32),
    Description(String), // pfa descriptions
    Keyword(String),
    Symbol(String),
    Boolean(bool),
    BuiltIn(BuiltIn),
    MarkovSequenceGenerator(MarkovSequenceGenerator),
    SoundEvent(Event),
    ControlEvent(ControlEvent),
    Rule(Rule),
    Command(Command),
    SyncContext(SyncContext),
    PartProxy(PartProxy),
    ProxyList(Vec<PartProxy>),
    Generator(Generator),
    GeneratorProcessor(Box<dyn GeneratorProcessor + Send>),
    GeneratorProcessorList(Vec<Box<dyn GeneratorProcessor + Send>>),
    GeneratorList(Vec<Generator>),
    Parameter(Parameter),
    GeneratorModifierFunction(
        (
            GenModFun,
            Vec<ConfigParameter>,
            HashMap<String, ConfigParameter>,
        ),
    ),
    Nothing,
}

pub enum Expr {
    Comment,
    Constant(Atom),
    Custom(String),
    Application(Box<Expr>, Vec<Expr>),
}
