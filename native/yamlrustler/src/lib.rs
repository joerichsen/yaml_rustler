use rustler::{Encoder, Env, Term, NifResult, types::atom::ok, types::tuple::make_tuple};
use yaml_rust2::{YamlLoader, Yaml};

mod atoms {
    rustler::atoms! {
        infinity,
        neg_infinity,
        nan,
    }
}

#[rustler::nif]
fn parse<'a>(env: Env<'a>, input: String) -> NifResult<Term<'a>> {
    match YamlLoader::load_from_str(&input) {
        Ok(docs) => {
            // An empty stream (e.g. "" or only comments) has no documents
            let result = match docs.first() {
                Some(doc) => convert_yaml_to_term(env, &resolve_aliases(doc, &docs)),
                None => rustler::types::atom::nil().encode(env),
            };
            Ok(make_tuple(env, &[ok().encode(env), result]))
        },
        Err(e) => Err(rustler::Error::Term(Box::new(format!("YAML parsing error: {}", e)))),
    }
}

fn resolve_aliases<'a>(yaml: &'a Yaml, docs: &'a [Yaml]) -> Yaml {
    match yaml {
        Yaml::Hash(h) => {
            let mut new_hash = yaml_rust2::yaml::Hash::new();
            for (k, v) in h {
                let new_k = resolve_aliases(k, docs);
                let new_v = resolve_aliases(v, docs);
                if new_k == Yaml::String("<<".to_string()) {
                    // A merge key takes a mapping or a list of mappings, where
                    // earlier mappings take precedence over later ones
                    let merge_hashes = match new_v {
                        Yaml::Hash(merge_hash) => vec![merge_hash],
                        Yaml::Array(merge_list) => merge_list
                            .into_iter()
                            .filter_map(|m| match m {
                                Yaml::Hash(merge_hash) => Some(merge_hash),
                                _ => None,
                            })
                            .collect(),
                        _ => vec![],
                    };
                    for merge_hash in merge_hashes {
                        for (merge_k, merge_v) in merge_hash {
                            if !new_hash.contains_key(&merge_k) {
                                new_hash.insert(merge_k, merge_v);
                            }
                        }
                    }
                } else {
                    new_hash.insert(new_k, new_v);
                }
            }
            Yaml::Hash(new_hash)
        },
        Yaml::Array(a) => Yaml::Array(a.iter().map(|v| resolve_aliases(v, docs)).collect()),
        Yaml::Alias(i) => {
            if let Some(alias_value) = docs.get(*i) {
                resolve_aliases(alias_value, docs)
            } else {
                Yaml::Null
            }
        },
        _ => yaml.clone(),
    }
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
            let map: std::collections::HashMap<_, _> = h.iter()
                .map(|(k, v)| (convert_yaml_to_term(env, k), convert_yaml_to_term(env, v)))
                .collect();
            map.encode(env)
        },
        Yaml::Null => rustler::types::atom::nil().encode(env),
        Yaml::BadValue => "<<bad_value>>".encode(env),
        Yaml::Alias(_) => rustler::types::atom::nil().encode(env), // This should not happen after resolution
    }
}

rustler::init!("Elixir.YamlRustler");