use rustler::{Encoder, Env, Term, NifResult, types::atom::ok, types::tuple::make_tuple};
use std::collections::HashSet;
use yaml_rust2::{
    parser::{Event, MarkedEventReceiver, Parser},
    scanner::{Marker, ScanError, TScalarStyle},
    yaml::Hash,
    Yaml, YamlLoader,
};

mod atoms {
    rustler::atoms! {
        infinity,
        neg_infinity,
        nan,
        reschedule,
    }
}

#[rustler::nif]
fn parse_nif<'a>(env: Env<'a>, input: &str, ruby: bool) -> NifResult<Term<'a>> {
    parse(env, input, ruby, false)
}

// Large inputs take more than the ~1 ms a NIF should run on a normal scheduler
#[rustler::nif(schedule = "DirtyCpu")]
fn parse_dirty_nif<'a>(env: Env<'a>, input: &str, ruby: bool) -> NifResult<Term<'a>> {
    parse(env, input, ruby, true)
}

fn parse<'a>(env: Env<'a>, input: &str, ruby: bool, dirty: bool) -> NifResult<Term<'a>> {
    let loaded = if dirty { load(input, ruby) } else { load_documents(input, ruby) };
    match loaded {
        Ok(docs) => {
            // An empty stream (e.g. "" or only comments) has no documents
            let result = match docs.first() {
                Some(doc) => convert_yaml_to_term(env, doc),
                None => rustler::types::atom::nil().encode(env),
            };
            Ok(make_tuple(env, &[ok().encode(env), result]))
        },
        // Reindenting quoted scalars parses the document again for each of
        // them, so it is left to the dirty scheduler
        Err(e) if !dirty && needs_reindent(&e) => Ok(atoms::reschedule().encode(env)),
        Err(e) => Err(rustler::Error::Term(Box::new(format!("YAML parsing error: {}", e)))),
    }
}

// Ruby's Psych (as used by Rails on Ruby 2.4) writes multi-line quoted strings
// whose continuation lines are not indented, e.g.
//
//   iv: 'abc==
//
//   '
//
// libyaml accepts this, but yaml-rust2 rejects it. Leading whitespace on the
// continuation lines of a quoted scalar is not part of its value, so indent
// them and parse again, once for each such scalar.
const MAX_REINDENTS: usize = 100;

fn load(input: &str, ruby: bool) -> Result<Documents, ScanError> {
    let first = match load_documents(input, ruby) {
        Err(e) if needs_reindent(&e) => e,
        result => return result,
    };

    let mut text = input.to_string();
    let mut marker = *first.marker();
    for _ in 0..MAX_REINDENTS {
        match reindent_quoted_scalar(&text, &marker) {
            Some(reindented) if reindented != text => text = reindented,
            _ => return Err(first),
        }
        match load_documents(&text, ruby) {
            Err(e) if needs_reindent(&e) => marker = *e.marker(),
            Err(_) => return Err(first),
            ok => return ok,
        }
    }
    Err(first)
}

// Parsed documents, either from YamlLoader::load_from_str or still inside the
// loader that RubyScalars fed (which only lends them out).
enum Documents {
    Owned(Vec<Yaml>),
    Loader(YamlLoader),
}

impl Documents {
    fn first(&self) -> Option<&Yaml> {
        match self {
            Documents::Owned(docs) => docs.first(),
            Documents::Loader(loader) => loader.documents().first(),
        }
    }
}

fn load_documents(text: &str, ruby: bool) -> Result<Documents, ScanError> {
    if !ruby {
        return YamlLoader::load_from_str(text).map(Documents::Owned);
    }

    let mut receiver = RubyScalars { loader: YamlLoader::default(), document_ends: 0, last_mark: None };
    Parser::new_from_str(text).load(&mut receiver, true)?;
    // The loader's own errors (duplicated keys) are private, but it stops
    // storing documents after one. Load again the standard way to get it.
    if receiver.loader.documents().len() < receiver.document_ends {
        YamlLoader::load_from_str(text)?;
        let mark = receiver.last_mark.expect("a document was loaded");
        return Err(ScanError::new(mark, "duplicated key in mapping"));
    }
    Ok(Documents::Loader(receiver.loader))
}

// Feeds a YamlLoader, turning plain scalars that YAML 1.2 reads as numbers but
// Ruby's Psych (YAML 1.1) reads as strings into strings. Psych writes such
// strings without quotes: 1e5 and 1.5e3 (a Psych float needs a "." and a
// signed exponent), and 0o17 or 0x+1 (not integers in YAML 1.1). Tagged
// scalars, like !!float 1e5, are left to the loader.
struct RubyScalars {
    loader: YamlLoader,
    document_ends: usize,
    last_mark: Option<Marker>,
}

impl MarkedEventReceiver for RubyScalars {
    fn on_event(&mut self, event: Event, mark: Marker) {
        self.last_mark = Some(mark);
        let event = match event {
            Event::Scalar(value, TScalarStyle::Plain, anchor, None) if is_psych_string(&value) => {
                Event::Scalar(value, TScalarStyle::SingleQuoted, anchor, None)
            }
            Event::DocumentEnd => {
                self.document_ends += 1;
                event
            }
            event => event,
        };
        self.loader.on_event(event, mark);
    }
}

fn is_psych_string(value: &str) -> bool {
    match Yaml::from_str(value) {
        // Integers beyond i64 also end up as reals
        Yaml::Real(_) => !is_psych_float(value) && !is_psych_integer(value),
        Yaml::Integer(_) => !is_psych_integer(value),
        _ => false,
    }
}

fn needs_reindent(error: &ScanError) -> bool {
    error.info() == "invalid indentation in quoted scalar"
}

// Indents the continuation lines of the quoted scalar starting at `marker` one
// column deeper than the key (or sequence entry) on the line it starts on.
fn reindent_quoted_scalar(text: &str, marker: &Marker) -> Option<String> {
    let line_start = line_offset(text, marker.line())?;
    let line = &text[line_start..];
    let start = line_start + line.char_indices().nth(marker.col())?.0;
    let quote = text[start..].chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }

    // Skip the indentation and any "- " sequence indicators before the key
    let mut key_col = 0;
    let mut prefix = line.chars().peekable();
    while let Some(c) = prefix.next() {
        match c {
            ' ' => key_col += 1,
            '-' if prefix.peek() == Some(&' ') => key_col += 1,
            _ => break,
        }
    }
    let indent = (key_col + 1).min(marker.col());
    // Give up rather than let a pathological input grow without bound
    let limit = 2 * text.len() + 1024;

    let mut result = String::with_capacity(text.len() + 64);
    result.push_str(&text[..=start]);
    let body = &text[start + 1..];
    let mut chars = body.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        result.push(c);
        match c {
            // '' is an escaped quote in a single-quoted scalar
            '\'' if quote == '\'' && chars.peek().map(|(_, n)| *n) == Some('\'') => {
                result.push(chars.next()?.1);
            }
            '\\' if quote == '"' => {
                let (j, escaped) = chars.next()?;
                result.push(escaped);
                // An escaped line break continues on the next line
                if escaped == '\r' || escaped == '\n' {
                    let end = push_crlf(&mut result, &mut chars, escaped).unwrap_or(j);
                    push_indentation(&mut result, &body[end + 1..], indent, limit)?;
                }
            }
            '\r' | '\n' => {
                let end = push_crlf(&mut result, &mut chars, c).unwrap_or(i);
                push_indentation(&mut result, &body[end + 1..], indent, limit)?;
            }
            c if c == quote => {
                result.push_str(&body[i + 1..]);
                return Some(result);
            }
            _ => {}
        }
    }
    // No closing quote
    None
}

// The byte offset of the start of a 1-based line, counting line breaks like
// yaml-rust2 does: "\r\n", "\r" and "\n".
fn line_offset(text: &str, line: usize) -> Option<usize> {
    let mut current = 1;
    let mut chars = text.char_indices().peekable();
    let mut offset = 0;
    while current < line {
        let (i, c) = chars.next()?;
        if c == '\r' || c == '\n' {
            if c == '\r' && chars.peek().map(|(_, n)| *n) == Some('\n') {
                chars.next();
                offset = i + 2;
            } else {
                offset = i + 1;
            }
            current += 1;
        }
    }
    Some(offset)
}

// After a '\r', consumes and pushes a following '\n' so that "\r\n" counts as
// one line break. Returns the byte offset of that '\n'.
fn push_crlf(
    result: &mut String,
    chars: &mut std::iter::Peekable<std::str::CharIndices>,
    c: char,
) -> Option<usize> {
    if c == '\r' && chars.peek().map(|(_, n)| *n) == Some('\n') {
        let (j, n) = chars.next()?;
        result.push(n);
        return Some(j);
    }
    None
}

// Adds the spaces the line starting `rest` needs to be indented to `indent`.
// Returns None for a document marker, which ends a quoted scalar with an
// error, or when the result would grow beyond `limit`.
fn push_indentation(result: &mut String, rest: &str, indent: usize, limit: usize) -> Option<()> {
    if (rest.starts_with("---") || rest.starts_with("..."))
        && rest[3..].chars().next().map_or(true, |c| c.is_whitespace())
    {
        return None;
    }
    let spaces = rest.chars().take_while(|c| *c == ' ').count();
    // Lines with only whitespace are empty lines and need no indentation
    let empty = matches!(rest[spaces..].chars().next(), Some('\n') | Some('\r') | None);
    if spaces < indent && !empty {
        if result.len() + indent - spaces > limit {
            return None;
        }
        result.extend(std::iter::repeat(' ').take(indent - spaces));
    }
    Some(())
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

// Psych's ScalarScanner::INTEGER:
//   [-+]?0b[0-1_,]+  |  [-+]?0[0-7_,]+  |  [-+]?(0|[1-9][0-9_,]*)  |  [-+]?0x[0-9a-fA-F_,]+
fn is_psych_integer(s: &str) -> bool {
    let unsigned = s.strip_prefix(['+', '-']).unwrap_or(s);
    let all = |digits: &str, valid: fn(char) -> bool| {
        !digits.is_empty() && digits.chars().all(|c| valid(c) || c == '_' || c == ',')
    };
    if let Some(digits) = unsigned.strip_prefix("0b") {
        all(digits, |c| c == '0' || c == '1')
    } else if let Some(digits) = unsigned.strip_prefix("0x") {
        all(digits, |c| c.is_ascii_hexdigit())
    } else if let Some(digits) = unsigned.strip_prefix('0') {
        digits.is_empty() || all(digits, |c| ('0'..='7').contains(&c))
    } else {
        unsigned.starts_with(|c: char| c.is_ascii_digit()) && all(unsigned, |c| c.is_ascii_digit())
    }
}

// Psych's ScalarScanner::FLOAT:
//   [-+]?([0-9][0-9_,]*)?\.[0-9]*([eE][-+][0-9]+)?  |  [-+]?\.(inf|Inf|INF)  |  \.(nan|NaN|NAN)
// It also reads .inf, -.inf and .nan in any case. "." alone is a string.
fn is_psych_float(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    if matches!(lower.as_str(), ".inf" | "+.inf" | "-.inf" | ".nan") {
        return true;
    }

    let unsigned = s.strip_prefix(['+', '-']).unwrap_or(s);
    let Some((whole, fraction)) = unsigned.split_once('.') else {
        return false;
    };
    let whole_ok = whole.is_empty()
        || (whole.starts_with(|c: char| c.is_ascii_digit())
            && whole.chars().all(|c| c.is_ascii_digit() || c == '_' || c == ','));
    let (digits, exponent) = match fraction.find(['e', 'E']) {
        Some(i) => (&fraction[..i], Some(&fraction[i + 1..])),
        None => (fraction, None),
    };
    let exponent_ok = exponent.map_or(true, |e| {
        e.len() > 1 && e.starts_with(['+', '-']) && e[1..].chars().all(|c| c.is_ascii_digit())
    });
    whole_ok && digits.chars().all(|c| c.is_ascii_digit()) && exponent_ok && !(whole.is_empty() && fraction.is_empty())
}

rustler::init!("Elixir.YamlRustler");