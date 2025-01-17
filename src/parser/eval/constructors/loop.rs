use crate::builtin_types::*;
use crate::event::*;
use crate::generator::Generator;
use crate::markov_sequence_generator::MarkovSequenceGenerator;
use crate::parameter::*;

use ruffbox_synth::building_blocks::SynthParameterLabel;
use std::collections::{BTreeSet, HashMap};
use std::sync;
use vom_rs::pfa::{Pfa, Rule};

use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;

/// construct a simple loop of events ... no tricks here
pub fn a_loop(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    global_parameters: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..);

    // ignore function name in this case
    tail_drain.next();

    // name is the first symbol
    let name = if let Some(EvaluatedExpr::Symbol(n)) = tail_drain.next() {
        n
    } else {
        "".to_string()
    };

    // collect final events and durations in their position in the list
    let mut ev_vecs = Vec::new();
    let mut dur_vec: Vec<DynVal> = Vec::new();

    let mut keep_root = false;

    let dur: DynVal = if let ConfigParameter::Numeric(d) = global_parameters
        .entry(BuiltinGlobalParameters::DefaultDuration)
        .or_insert(ConfigParameter::Numeric(200.0))
        .value()
    {
        DynVal::with_value(*d)
    } else {
        unreachable!()
    };

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::BuiltIn(BuiltIn::SoundEvent(e)) => {
                ev_vecs.push(vec![SourceEvent::Sound(e)]);
                dur_vec.push(dur.clone());
                continue;
            }
            EvaluatedExpr::BuiltIn(BuiltIn::ControlEvent(e)) => {
                ev_vecs.push(vec![SourceEvent::Control(e)]);
                dur_vec.push(dur.clone());
                continue;
            }
            EvaluatedExpr::Float(f) => {
                if !dur_vec.is_empty() {
                    *dur_vec.last_mut().unwrap() = DynVal::with_value(f);
                } else {
                    dur_vec.push(DynVal::with_value(f));
                }
            }
            EvaluatedExpr::Keyword(k) => {
                if k == "keep" {
                    if let Some(EvaluatedExpr::Boolean(b)) = tail_drain.next() {
                        keep_root = b;
                    }
                }
            }
            _ => println! {"ignored"},
        }
    }

    let mut event_mapping = HashMap::<char, Vec<SourceEvent>>::new();
    let mut duration_mapping = HashMap::<(char, char), Event>::new();

    let pfa = if !keep_root {
        // generated ids
        let mut last_char: char = '1';
        let first_char: char = last_char;

        // collect cycle rules
        let mut rules = Vec::new();
        let len = ev_vecs.len() - 1;

        for (count, ev) in ev_vecs.drain(..).enumerate() {
            event_mapping.insert(last_char, ev);

            if count < len {
                let next_char: char = std::char::from_u32(last_char as u32 + 1).unwrap();

                let mut dur_ev = Event::with_name("transition".to_string());
                dur_ev.params.insert(
                    SynthParameterLabel::Duration,
                    ParameterValue::Scalar(dur_vec[count].clone()),
                );

                rules.push(Rule {
                    source: vec![last_char],
                    symbol: next_char,
                    probability: 1.0,
                });

                duration_mapping.insert((last_char, next_char), dur_ev);

                last_char = next_char;
            }
        }

        let mut dur_ev = Event::with_name("transition".to_string());
        dur_ev.params.insert(
            SynthParameterLabel::Duration,
            ParameterValue::Scalar(if let Some(ldur) = dur_vec.last() {
                ldur.clone()
            } else {
                dur.clone()
            }),
        );

        // close the loop
        rules.push(Rule {
            source: vec![last_char],
            symbol: first_char,
            probability: 1.0,
        });

        duration_mapping.insert((last_char, first_char), dur_ev);

        // don't remove orphans here because the first state is technically
        // "orphan"
        Pfa::<char>::infer_from_rules(&mut rules, true)
    } else {
        Pfa::<char>::new()
    };

    let mut id_tags = BTreeSet::new();
    id_tags.insert(name.clone());

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Generator(Generator {
        id_tags,
        root_generator: MarkovSequenceGenerator {
            name,
            generator: pfa,
            event_mapping,
            duration_mapping,
            modified: true,
            symbol_ages: HashMap::new(),
            default_duration: dur.static_val as u64,
            last_transition: None,
            last_symbol: None,
        },
        processors: Vec::new(),
        time_mods: Vec::new(),
        keep_root,
    })))
}
