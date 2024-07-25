use rustler::{Encoder, Env, Term, NifResult, types::atom::ok, types::tuple::make_tuple};
use yaml_rust2::{YamlLoader, Yaml};
use std::collections::HashMap;

#[rustler::nif]
fn parse<'a>(env: Env<'a>, input: String) -> NifResult<Term<'a>> {
    match YamlLoader::load_from_str(&input) {
        Ok(docs) => {
            let result = convert_yaml_to_term(env, &docs[0]);
            Ok(make_tuple(env, &[ok().encode(env), result]))
        },
        Err(e) => Err(rustler::Error::Term(Box::new(format!("YAML parsing error: {}", e)))),
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
            let map: HashMap<_, _> = h.iter()
                .map(|(k, v)| (convert_yaml_to_term(env, k), convert_yaml_to_term(env, v)))
                .collect();
            map.encode(env)
        },
        Yaml::Alias(_) => "<<alias>>".encode(env),
        Yaml::Null => ().encode(env),
        Yaml::BadValue => "<<bad_value>>".encode(env),
    }
}

rustler::init!("Elixir.YamlRustler");