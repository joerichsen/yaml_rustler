use rustler::{Encoder, Env, Term, NifResult, types::atom::ok, types::tuple::make_tuple};
use std::collections::HashSet;
use yaml_rust2::{yaml::Hash, YamlLoader, Yaml};

mod atoms {
    rustler::atoms! {
        infinity,
        neg_infinity,
        nan,
    }
}

#[rustler::nif]
fn parse<'a>(env: Env<'a>, input: &str) -> NifResult<Term<'a>> {
    match YamlLoader::load_from_str(input) {
        Ok(docs) => {
            // An empty stream (e.g. "" or only comments) has no documents
            let result = match docs.first() {
                Some(doc) => convert_yaml_to_term(env, doc),
                None => rustler::types::atom::nil().encode(env),
            };
            Ok(make_tuple(env, &[ok().encode(env), result]))
        },
        Err(e) => Err(rustler::Error::Term(Box::new(format!("YAML parsing error: {}", e)))),
    }
}

fn is_merge_key(key: &Yaml) -> bool {
    matches!(key, Yaml::String(s) if s == "<<")
}

// The entries of a mapping with merge keys (`<<`) resolved. The loader has
// already replaced aliases with copies of their anchors. Explicit keys take
// precedence over merged keys, and earlier merged mappings over later ones.
//
// Entries are ordered from lowest to highest precedence, so that when distinct
// YAML keys become the same term (e.g. 1.0 and 1.00) the last one is right.
fn mapping_entries(hash: &Hash) -> Vec<(&Yaml, &Yaml)> {
    let explicit: Vec<(&Yaml, &Yaml)> = hash.iter().filter(|(k, _)| !is_merge_key(k)).collect();
    if explicit.len() == hash.len() {
        return explicit;
    }

    let mut seen: HashSet<&Yaml> = explicit.iter().map(|(k, _)| *k).collect();
    // Collected from highest to lowest precedence
    let mut merged: Vec<(&Yaml, &Yaml)> = Vec::new();
    for (_, merge_value) in hash.iter().filter(|(k, _)| is_merge_key(k)) {
        // A merge key takes a mapping or a list of mappings
        let merge_hashes: Vec<&Hash> = match merge_value {
            Yaml::Hash(merge_hash) => vec![merge_hash],
            Yaml::Array(merge_list) => merge_list.iter().filter_map(Yaml::as_hash).collect(),
            _ => vec![],
        };
        for merge_hash in merge_hashes {
            for (k, v) in mapping_entries(merge_hash).into_iter().rev() {
                if seen.insert(k) {
                    merged.push((k, v));
                }
            }
        }
    }
    merged.reverse();
    merged.extend(explicit);
    merged
}

fn convert_yaml_to_term<'a>(env: Env<'a>, yaml: &Yaml) -> Term<'a> {
    match yaml {
        // The BEAM has no infinity or NaN floats, so those become atoms
        Yaml::Real(_) => match yaml.as_f64() {
            Some(f) if f.is_finite() => f.encode(env),
            Some(f) if f.is_nan() => atoms::nan().encode(env),
            Some(f) if f > 0.0 => atoms::infinity().encode(env),
            Some(_) => atoms::neg_infinity().encode(env),
            None => rustler::types::atom::nil().encode(env),
        },
        Yaml::Integer(i) => i.encode(env),
        Yaml::String(s) => s.encode(env),
        Yaml::Boolean(b) => b.encode(env),
        Yaml::Array(a) => a.iter().map(|v| convert_yaml_to_term(env, v)).collect::<Vec<Term>>().encode(env),
        Yaml::Hash(h) => {
            let (keys, values): (Vec<Term>, Vec<Term>) = mapping_entries(h)
                .into_iter()
                .map(|(k, v)| (convert_yaml_to_term(env, k), convert_yaml_to_term(env, v)))
                .unzip();
            // Building the map in one go fails on duplicate keys, which happens when
            // different YAML keys become the same term (e.g. 1.0 and 1.00). Then the
            // last one wins.
            Term::map_from_term_arrays(env, &keys, &values).unwrap_or_else(|_| {
                keys.iter().zip(values.iter()).fold(Term::map_new(env), |map, (k, v)| {
                    map.map_put(*k, *v).unwrap()
                })
            })
        },
        Yaml::Null => rustler::types::atom::nil().encode(env),
        Yaml::BadValue => "<<bad_value>>".encode(env),
        // The loader resolves aliases, so these do not occur
        Yaml::Alias(_) => rustler::types::atom::nil().encode(env),
    }
}

rustler::init!("Elixir.YamlRustler");