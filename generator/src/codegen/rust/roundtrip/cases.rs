//! Runs the generated code of the selector choice packets.

use crate::packets::srv_clan_command::*;
use crate::packets::srv_event::*;
use crate::packets::shared_types::{EventBody, ShapeBody};
use crate::packets::srv_shape::*;
use crate::packets::srv_wish_list::*;
use crate::testgen::{Gen, Sample};
use crate::wire::{Reader, Wire, WireError, Writer};

fn bytes<T: Wire>(value: &T) -> Vec<u8> {
    let mut w = Writer::default();
    value.write(&mut w);
    w.0
}

fn parse<T: Wire>(bytes: &[u8]) -> Result<T, WireError> {
    let mut r = Reader::new(bytes);
    let value = T::read(&mut r)?;
    assert_eq!(r.remaining(), 0, "bytes left over");
    Ok(value)
}

fn round_trips<T: Wire + Sample + PartialEq + std::fmt::Debug>() {
    let mut g = Gen::new(7);
    for _ in 0..500 {
        let value = T::sample(&mut g);
        assert_eq!(parse::<T>(&bytes(&value)), Ok(value));
    }
}

#[test]
fn every_variant_survives_a_write_and_a_read() {
    round_trips::<SrvClanCommand>();
    round_trips::<SrvEvent>();
    round_trips::<SrvShape>();
    round_trips::<SrvWishList>();
}

#[test]
fn the_selector_written_is_the_one_of_the_variant() {
    let created = SrvClanCommand {
        body: SrvClanCommandBody::Created { clan: MyClan { id: 9, level: 2 }, name: b"ab".to_vec() },
    };
    assert_eq!(bytes(&created), [0x30, 9, 0, 0, 0, 2, b'a', b'b', 0]);
    let taken = SrvClanCommand { body: SrvClanCommandBody::NameTaken };
    assert_eq!(bytes(&taken), [0x31]);
    assert_eq!(taken.body.selector(), ClanResult::NAME_TAKEN);
    let event = SrvEvent { time: 1, event: SrvEventEvent::Case16 };
    assert_eq!(bytes(&event), [16, 0, 1, 0, 0, 0]);
}

#[test]
fn a_list_ending_a_branch_takes_the_rest_of_the_packet() {
    let roster = parse::<SrvClanCommand>(&[0x32, b'a', 0, 1, 0, b'b', 0, 2, 0]).unwrap();
    let members = vec![
        ClanMember { name: b"a".to_vec(), level: 1 },
        ClanMember { name: b"b".to_vec(), level: 2 },
    ];
    assert_eq!(roster.body, SrvClanCommandBody::Roster { members });
}

#[test]
fn an_unknown_value_without_a_default_does_not_read() {
    for value in [0x33u8, 0x99] {
        assert_eq!(
            parse::<SrvClanCommand>(&[value]),
            Err(WireError::UnknownCase { what: "SrvClanCommandBody", value: value as i64 })
        );
    }
}

#[test]
fn any_other_value_reads_as_the_default_branch() {
    let event = parse::<SrvEvent>(&[5, 0, 1, 0, 0, 0, 7, 8]).unwrap();
    assert_eq!(event.event, SrvEventEvent::Other { kind: 5, raw: vec![7, 8] });
    assert_eq!(event.event.selector(), 5);
    let event = parse::<SrvEvent>(&[1, 0, 1, 0, 0, 0, 0, 0, 0x80, 0x3f, 2, 7, 8]).unwrap();
    assert_eq!(event.event, SrvEventEvent::Case1 { x: 1.0, tags: vec![7, 8] });
}

#[test]
fn a_type_reads_what_follows_its_choice() {
    let shape = parse::<SrvShape>(&[2, 3, 0, 4, 0, 9, 1, 0, 0, 0]).unwrap();
    assert_eq!(shape.shape, Shape { body: ShapeBody::Case2 { w: 3, h: 4 }, after: 9 });
    assert_eq!(bytes(&shape), [2, 3, 0, 4, 0, 9, 1, 0, 0, 0]);
    assert_eq!(bytes(&Shape { body: ShapeBody::CaseMinus1, after: 1 }), [0xff, 1]);
}

#[test]
fn an_enumeration_selector_reads_other_values_as_the_default() {
    let wish = parse::<SrvWishList>(&[1, 2, 2, 9]).unwrap();
    assert_eq!(wish.event, Event { body: EventBody::One { x: Kind::TWO } });
    assert_eq!(wish.body, SrvWishListBody::Two { y: 9 });
    let wish = parse::<SrvWishList>(&[2, 5, 0, 7]).unwrap();
    assert_eq!(wish.event, Event { body: EventBody::Other { kind: Kind::TWO, raw: 5 } });
    assert_eq!(wish.body, SrvWishListBody::Other { result: Kind(7) });
    assert_eq!(bytes(&wish), [2, 5, 0, 7]);
}

#[test]
fn the_default_given_a_named_value_writes_it_and_reads_back_as_that_branch() {
    // The one way the selector and the bytes after it can disagree: what
    // follows is the default branch's (nothing), not the named one's.
    let event = Event { body: EventBody::One { x: Kind::NONE } };
    let wish = SrvWishList { event, body: SrvWishListBody::Other { result: Kind::TWO } };
    let written = bytes(&wish);
    assert_eq!(written, [1, 0, 2]);
    assert_eq!(parse::<SrvWishList>(&written), Err(WireError::UnexpectedEnd));
}
