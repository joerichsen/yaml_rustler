use rustler::{Encoder, Env, Term};
use yaml_rust2::{YamlLoader, Yaml};

#[rustler::nif]
fn parse(input: String) -> Result<Term, String> {
    match YamlLoader::load_from_str(&input) {
        Ok(docs) => Ok(convert_yaml_to_term(&docs[0])),
        Err(e) => Err(format!("YAML parsing error: {}", e)),
    }
}

fn convert_yaml_to_term(yaml: &Yaml) -> Term {
    match yaml {
        Yaml::Real(s) => s.parse::<f64>().unwrap().encode(),
        Yaml::Integer(i) => i.encode(),
        Yaml::String(s) => s.encode(),
        Yaml::Boolean(b) => b.encode(),
        Yaml::Array(a) => a.iter().map(convert_yaml_to_term).collect::<Vec<Term>>().encode(),
        Yaml::Hash(h) => h.iter().map(|(k, v)| (convert_yaml_to_term(k), convert_yaml_to_term(v))).collect::<Vec<(Term, Term)>>().encode(),
        Yaml::Alias(_) => "<<alias>>".encode(),
        Yaml::Null => ().encode(),
        Yaml::BadValue => "<<bad_value>>".encode(),
    }
}

rustler::init!("Elixir.YamlRustler.Native", [parse]);