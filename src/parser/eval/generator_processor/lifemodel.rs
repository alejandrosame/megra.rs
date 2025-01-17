use crate::event_helpers::*;

use crate::generator_processor::*;
use crate::parameter::DynVal;

use crate::parser::{BuiltIn, EvaluatedExpr};

pub fn collect_lifemodel(tail: &mut Vec<EvaluatedExpr>) -> Box<dyn GeneratorProcessor + Send> {
    let mut tail_drain = tail.drain(..);
    tail_drain.next(); // skip function name

    let mut proc = LifemodelProcessor::new();

    // positional args: growth cycle, lifespan, variance
    if let Some(EvaluatedExpr::Float(growth_cycle)) = tail_drain.next() {
        proc.growth_cycle = growth_cycle as usize;
    }

    if let Some(EvaluatedExpr::Float(lifespan)) = tail_drain.next() {
        proc.node_lifespan = lifespan as usize;
    }

    if let Some(EvaluatedExpr::Float(variance)) = tail_drain.next() {
        proc.variance = variance;
    }

    let mut collect_durations = false;
    let mut collect_keeps = false;

    while let Some(c) = tail_drain.next() {
        if collect_durations {
            match c {
                EvaluatedExpr::Float(f) => proc.durations.push(DynVal::with_value(f)),
                EvaluatedExpr::BuiltIn(BuiltIn::Parameter(ref p)) => proc.durations.push(p.clone()),
                _ => {
                    collect_durations = false;
                }
            }
        }

        if collect_keeps {
            match c {
                EvaluatedExpr::Symbol(ref s) => {
                    proc.keep_param.insert(map_parameter(s));
                }
                _ => {
                    collect_keeps = false;
                }
            }
        }

        if let EvaluatedExpr::Keyword(k) = c {
            match k.as_str() {
                "durs" => {
                    collect_durations = true;
                }
                "keep" => {
                    collect_keeps = true;
                }
                "apoptosis" => {
                    if let Some(EvaluatedExpr::Boolean(b)) = tail_drain.next() {
                        proc.apoptosis = b;
                    }
                }
                "method" => {
                    if let Some(EvaluatedExpr::Symbol(s)) = tail_drain.next() {
                        proc.growth_method = s;
                    }
                }
                "autophagia" => {
                    if let Some(EvaluatedExpr::Boolean(b)) = tail_drain.next() {
                        proc.autophagia = b;
                    }
                }
                "lifespan-variance" => {
                    if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
                        proc.node_lifespan_variance = f;
                    }
                }
                "apoptosis-regain" => {
                    if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
                        proc.apoptosis_regain = f;
                    }
                }
                "autophagia-regain" => {
                    if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
                        proc.autophagia_regain = f;
                    }
                }
                "local-resources" => {
                    if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
                        proc.local_resources = f;
                    }
                }
                "cost" => {
                    if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
                        proc.growth_cost = f;
                    }
                }
                "global-contrib" => {
                    if let Some(EvaluatedExpr::Boolean(b)) = tail_drain.next() {
                        proc.global_contrib = b;
                    }
                }
                "rnd" => {
                    if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
                        proc.rnd_chance = f;
                    }
                }
                "solidify" => {
                    if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
                        proc.solidify_chance = f;
                    }
                }
                "solidify-len" => {
                    if let Some(EvaluatedExpr::Float(f)) = tail_drain.next() {
                        proc.solidify_len = f as usize;
                    }
                }
                _ => {}
            }
        }
    }

    Box::new(proc)
}
