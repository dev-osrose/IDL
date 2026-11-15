//! Checks of `<choice selector="...">`, shared by the backends: the selector
//! is an earlier plain element of the same packet or sequence, of an
//! enumeration of the XML or of an integer type; each branch is for one value
//! of it, at most one for every other value; and the branches can be read
//! back.

use std::collections::BTreeMap;

use ::flat_ast::{ComplexTypeContent, Element, Occurs, Packet, PacketContent, RestrictionContent, Selected, SimpleTypeContent};
use ::heck::ToUpperCamelCase;

type Result<T> = ::std::result::Result<T, ::failure::Error>;

/// The values a selector can take.
enum Values {
    /// An enumeration: its enumerators with their values.
    Enum(BTreeMap<String, i64>),
    /// An integer type: the lowest and highest value.
    Integer(i128, i128),
}

/// Checks every selector choice of `packet`, and records the type of each
/// one's selector.
pub fn check(packet: &mut Packet) -> Result<()> {
    let mut containers: Vec<(String, Vec<Element>)> = vec![(
        packet.type_().clone(),
        packet
            .contents()
            .iter()
            .filter_map(|c| match c {
                PacketContent::Element(e) => Some(e.clone()),
                _ => None,
            })
            .collect(),
    )];
    for content in packet.contents() {
        if let PacketContent::Complex(c) = content {
            if let ComplexTypeContent::Seq(s) = c.content() {
                containers.push((c.name().clone(), s.elements().to_vec()));
            }
        }
    }
    let mut selector_types = BTreeMap::new();
    for (context, elements) in &containers {
        for (index, element) in elements.iter().enumerate() {
            if let Some(selected) = selected_type(packet, element.type_()) {
                let ty = check_one(packet, context, elements, index, selected)?;
                selector_types.insert(element.type_().clone(), ty);
            }
        }
    }
    for content in packet.contents_mut() {
        if let PacketContent::Complex(c) = content {
            let name = c.name().clone();
            if let ComplexTypeContent::Selected(s) = c.content_mut() {
                if let Some(ty) = selector_types.remove(&name) {
                    s.set_selector_type(ty);
                }
            }
        }
    }
    Ok(())
}

/// The selector choice `type_` names, if it names one.
pub fn selected_type<'p>(packet: &'p Packet, type_: &str) -> Option<&'p Selected> {
    packet.contents().iter().find_map(|c| match c {
        PacketContent::Complex(c) if c.name() == type_ => match c.content() {
            ComplexTypeContent::Selected(s) => Some(s),
            _ => None,
        },
        _ => None,
    })
}

fn fail<T>(message: String) -> Result<T> {
    Err(::failure::err_msg(message))
}

/// Checks the choice of `elements[index]`; returns the type of its selector.
fn check_one(packet: &Packet, context: &str, elements: &[Element], index: usize, selected: &Selected) -> Result<String> {
    let choice = elements[index].name();
    let name = selected.selector();
    let selector = match elements[..index].iter().find(|e| e.name() == name) {
        Some(selector) => selector,
        None => return fail(format!("{}: choice {}: selector {:?} is not an earlier element", context, choice, name)),
    };
    let counts = elements.iter().any(|e| {
        e.occurs_ref().as_deref() == Some(name.as_str()) || e.occurs_bits().as_ref().map_or(false, |(by, _)| by == name)
    });
    if selector.occurs().is_some() || selector.bits().is_some() || counts || selected_type(packet, selector.type_()).is_some() {
        return fail(format!("{}: selector {} must be a plain element: no list, bit field or count", context, name));
    }
    let values = match values(packet, selector.type_()) {
        Some(values) => values,
        None => {
            return fail(format!(
                "{}: selector {} has type {}, which is neither an enumeration of the XML nor an integer",
                context,
                name,
                selector.type_()
            ))
        }
    };

    let mut seen: BTreeMap<i128, String> = BTreeMap::new();
    let mut defaults = 0;
    for case in selected.cases() {
        let label = match case.value() {
            Some(label) => label,
            None => {
                defaults += 1;
                if defaults > 1 {
                    return fail(format!("{}: choice {} has more than one default branch", context, choice));
                }
                continue;
            }
        };
        let value = match &values {
            Values::Enum(enumerators) => enumerators.get(label).map(|v| *v as i128),
            Values::Integer(low, high) => match parse_int(label) {
                Some(v) if v < *low || v > *high => {
                    return fail(format!("{}: case {} does not fit in {}", context, label, selector.type_()))
                }
                other => other,
            },
        };
        let value = match value {
            Some(value) => value,
            None => return fail(format!("{}: case {:?} is not a value of {}", context, label, selector.type_())),
        };
        if let Some(other) = seen.insert(value, label.clone()) {
            return fail(format!("{}: cases {} and {} have the same value", context, other, label));
        }
    }

    let mut members: BTreeMap<&str, &Element> = BTreeMap::new();
    for case in selected.cases() {
        let count = case.elements().len();
        for (at, elem) in case.elements().iter().enumerate() {
            let what = format!("{}: element {} of a branch", context, elem.name());
            if elements.iter().any(|e| e.name() == elem.name()) {
                return fail(format!("{} is already an element of {}", what, context));
            }
            if elem.occurs_ref().is_some() || elem.occurs_bits().is_some() {
                return fail(format!("{}: occursRef and occursBits are not supported in a branch", what));
            }
            if let Some(first) = members.insert(elem.name(), elem) {
                if first.type_() != elem.type_() || first.occurs() != elem.occurs() || first.size_occurs() != elem.size_occurs() {
                    return fail(format!("{} is in several branches, so it needs the same type in each", what));
                }
            }
            if to_end(elem) {
                if at + 1 != count {
                    return fail(format!("{}: a list without occursSize must be the last element of its branch", what));
                }
                if index + 1 != elements.len() {
                    return fail(format!(
                        "{}: choice {} ends in a list running to the end of the packet, so it must be the last element",
                        context, choice
                    ));
                }
            }
        }
    }
    Ok(selector.type_().clone())
}

/// A list that runs to the end of the packet.
fn to_end(elem: &Element) -> bool {
    matches!(elem.occurs(), Some(Occurs::Unbounded)) && elem.size_occurs().is_none()
}

/// The values of type `type_`, if it is an enumeration of the XML or an
/// integer type.
fn values(packet: &Packet, type_: &str) -> Option<Values> {
    let type_ = type_.trim();
    let range = |low: i128, high: i128| Some(Values::Integer(low, high));
    match type_ {
        "uint8_t" | "char" => return range(0, u8::MAX as i128),
        "int8_t" => return range(i8::MIN as i128, i8::MAX as i128),
        "uint16_t" => return range(0, u16::MAX as i128),
        "int16_t" => return range(i16::MIN as i128, i16::MAX as i128),
        "uint32_t" | "unsigned int" => return range(0, u32::MAX as i128),
        "int32_t" | "int" => return range(i32::MIN as i128, i32::MAX as i128),
        "uint64_t" => return range(0, u64::MAX as i128),
        "int64_t" => return range(i64::MIN as i128, i64::MAX as i128),
        _ => {}
    }
    let upper = type_.to_upper_camel_case();
    packet.contents().iter().find_map(|c| match c {
        PacketContent::Simple(s) if s.name() == type_ || *s.name() == upper => {
            let mut enumerators = BTreeMap::new();
            for content in s.contents() {
                let SimpleTypeContent::Restriction(r) = content;
                for item in r.contents() {
                    if let RestrictionContent::Enumeration(e) = item {
                        enumerators.insert(e.value().clone(), e.id());
                    }
                }
            }
            if enumerators.is_empty() {
                None
            } else {
                Some(Values::Enum(enumerators))
            }
        }
        _ => None,
    })
}

/// An integer as a case writes it: decimal or `0x` hexadecimal, maybe
/// negative.
pub fn parse_int(text: &str) -> Option<i128> {
    let text = text.trim();
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let value = match digits.strip_prefix("0x").or_else(|| digits.strip_prefix("0X")) {
        Some(hex) => i128::from_str_radix(hex, 16).ok()?,
        None => digits.parse::<i128>().ok()?,
    };
    Some(if negative { -value } else { value })
}
