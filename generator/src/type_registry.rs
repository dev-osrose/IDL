use std::collections::{BTreeMap, HashMap};
use crate::flat_ast::{Packet, PacketContent, SimpleType, ComplexType, ComplexTypeContent, Element, RestrictionContent, SimpleTypeContent, Sequence};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeFingerprint {
    Simple {
        name: String,
        base: String,
        restrictions: Vec<String>,
    },
    Enum {
        name: String,
        variants: Vec<(String, i64)>,
    },
    Struct {
        name: String,
        fields: Vec<(String, String, Option<u32>)>, // name, type, bits
    },
}

pub struct TypeRegistry {
    pub types: BTreeMap<String, PacketContent>,
    pub fingerprints: HashMap<String, TypeFingerprint>,
    pub is_copy: HashMap<String, bool>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        let mut is_copy = HashMap::new();
        // Built-in types
        is_copy.insert("int8_t".to_string(), true);
        is_copy.insert("uint8_t".to_string(), true);
        is_copy.insert("int16_t".to_string(), true);
        is_copy.insert("uint16_t".to_string(), true);
        is_copy.insert("int32_t".to_string(), true);
        is_copy.insert("uint32_t".to_string(), true);
        is_copy.insert("int64_t".to_string(), true);
        is_copy.insert("uint64_t".to_string(), true);
        is_copy.insert("char".to_string(), true);
        is_copy.insert("int".to_string(), true);
        is_copy.insert("unsigned int".to_string(), true);
        is_copy.insert("float".to_string(), true);
        is_copy.insert("double".to_string(), true);
        is_copy.insert("bool".to_string(), true);
        is_copy.insert("std::string".to_string(), false);
        is_copy.insert("NullTerminatedString".to_string(), false);

        Self {
            types: BTreeMap::new(),
            fingerprints: HashMap::new(),
            is_copy,
        }
    }

    pub fn collect_from_packet(&mut self, packet: &Packet) -> Result<(), failure::Error> {
        let filename = packet.filename();
        for content in packet.contents() {
            match content {
                PacketContent::Simple(ref s) => self.register_simple(s, filename)?,
                PacketContent::Complex(ref c) => self.register_complex(c, filename)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn register_simple(&mut self, s: &SimpleType, filename: &str) -> Result<(), failure::Error> {
        let fingerprint = self.fingerprint_simple(s);
        if let Some(existing) = self.fingerprints.get(s.name()) {
            if existing != &fingerprint {
                return Err(failure::format_err!("Conflicting definitions for type {} in {}:\nExisting: {:?}\nNew:      {:?}", s.name(), filename, existing, fingerprint));
            }
        } else {
            self.fingerprints.insert(s.name().clone(), fingerprint);
            self.types.insert(s.name().clone(), PacketContent::Simple(s.clone()));
        }
        Ok(())
    }

    fn register_complex(&mut self, c: &ComplexType, filename: &str) -> Result<(), failure::Error> {
        if c.anonymous() || c.inline() { return Ok(()); }
        
        let fingerprints = self.fingerprint_complex(c);
        for (name, fingerprint) in fingerprints {
            if let Some(existing) = self.fingerprints.get(&name) {
                if existing != &fingerprint {
                    return Err(failure::format_err!("Conflicting definitions for type {} in {}:\nExisting: {:?}\nNew:      {:?}", name, filename, existing, fingerprint));
                }
            } else {
                self.fingerprints.insert(name.clone(), fingerprint);
                if &name == c.name() {
                     self.types.insert(name.clone(), PacketContent::Complex(c.clone()));
                }
            }
        }
        Ok(())
    }

    fn fingerprint_simple(&self, s: &SimpleType) -> TypeFingerprint {
        let mut is_enum = false;
        let mut variants = Vec::new();
        let mut base = String::new();
        for content in s.contents() {
            if let SimpleTypeContent::Restriction(ref r) = content {
                base = r.base().clone();
                for r_content in r.contents() {
                    if let RestrictionContent::Enumeration(ref e) = r_content {
                        is_enum = true;
                        variants.push((e.value().clone(), e.id()));
                    }
                }
            }
        }

        if is_enum {
            variants.sort();
            TypeFingerprint::Enum { name: s.name().clone(), variants }
        } else {
            let mut restrictions = Vec::new();
            for content in s.contents() {
                if let SimpleTypeContent::Restriction(ref r) = content {
                    for r_content in r.contents() {
                        match r_content {
                            RestrictionContent::Length(l) => restrictions.push(format!("Length({})", l)),
                            RestrictionContent::MinValue(ref v) => restrictions.push(format!("Min({})", v)),
                            RestrictionContent::MaxValue(ref v) => restrictions.push(format!("Max({})", v)),
                            _ => {}
                        }
                    }
                }
            }
            restrictions.sort();
            TypeFingerprint::Simple { name: s.name().clone(), base, restrictions }
        }
    }

    fn fingerprint_complex(&self, c: &ComplexType) -> Vec<(String, TypeFingerprint)> {
        let mut res = Vec::new();
        match c.content() {
            ComplexTypeContent::Seq(ref s) => {
                res.push((c.name().clone(), self.fingerprint_sequence(c.name(), s)));
            }
            ComplexTypeContent::Choice(ref ch) => {
                let mut fields = Vec::new();
                for elem in ch.elements() {
                    fields.push((elem.name().clone(), elem.type_().clone(), elem.bits()));
                    if let Some(seq) = ch.inline_seqs().get(elem.name()) {
                        res.push((elem.name().clone(), self.fingerprint_sequence(elem.name(), seq)));
                    }
                }
                res.push((c.name().clone(), TypeFingerprint::Struct { name: c.name().clone(), fields }));
            }
            ComplexTypeContent::Empty => {
                res.push((c.name().clone(), TypeFingerprint::Struct { name: c.name().clone(), fields: Vec::new() }));
            }
        }
        res
    }

    fn fingerprint_sequence(&self, name: &str, s: &Sequence) -> TypeFingerprint {
        let mut fields = Vec::new();
        for elem in s.elements() {
            fields.push((elem.name().clone(), elem.type_().clone(), elem.bits()));
        }
        TypeFingerprint::Struct { name: name.to_string(), fields }
    }

    pub fn compute_is_copy(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;
            let fingerprints = self.fingerprints.clone();
            for (name, fp) in &fingerprints {
                if self.is_copy.contains_key(name) { continue; }

                let current_is_copy = match fp {
                    TypeFingerprint::Simple { base, .. } => {
                        *self.is_copy.get(base).unwrap_or(&true)
                    },
                    TypeFingerprint::Enum { .. } => true,
                    TypeFingerprint::Struct { fields, .. } => {
                        fields.iter().all(|(_, ty, _)| {
                            *self.is_copy.get(ty).unwrap_or(&false)
                        })
                    }
                };

                if current_is_copy {
                    self.is_copy.insert(name.clone(), true);
                    changed = true;
                }
            }
        }
    }
}
