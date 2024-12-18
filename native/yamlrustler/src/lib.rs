use rustler::{Encoder, Env, Term, NifResult, types::atom::ok, types::tuple::make_tuple};
use yaml_rust2::{YamlLoader, Yaml};

#[rustler::nif]
fn parse<'a>(env: Env<'a>, input: String) -> NifResult<Term<'a>> {
    match YamlLoader::load_from_str(&input) {
        Ok(docs) => {
            let resolved = resolve_aliases(&docs[0], &docs);
            let result = convert_yaml_to_term(env, &resolved);
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
                    if let Yaml::Hash(merge_hash) = new_v {
                        for (merge_k, merge_v) in merge_hash {
                            if !new_hash.contains_key(&merge_k) {
                                new_hash.insert(merge_k.clone(), merge_v.clone());
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
        Yaml::Real(s) => s.parse::<f64>().unwrap().encode(env),
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