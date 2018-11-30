#include "srv_login_req.h"

using namespace RoseCommon::Packet;


void SrvLoginReq::ChannelInfo::set_id(const uint8_t id) {
    this->id = id;
}

const uint8_t SrvLoginReq::ChannelInfo::get_id() const {
    return id;
}

void SrvLoginReq::ChannelInfo::set_lowAge(const uint8_t lowAge) {
    this->lowAge = lowAge;
}

const uint8_t SrvLoginReq::ChannelInfo::get_lowAge() const {
    return lowAge;
}

void SrvLoginReq::ChannelInfo::set_highAge(const uint8_t highAge) {
    this->highAge = highAge;
}

const uint8_t SrvLoginReq::ChannelInfo::get_highAge() const {
    return highAge;
}

void SrvLoginReq::ChannelInfo::set_capacity(const uint16_t capacity) {
    this->capacity = capacity;
}

const uint16_t SrvLoginReq::ChannelInfo::get_capacity() const {
    return capacity;
}

void SrvLoginReq::ChannelInfo::set_name(const std::string name) {
    this->name = name;
}

const std::string SrvLoginReq::ChannelInfo::get_name() const {
    return name;
}

bool SrvLoginReq::ChannelInfo::write(CRoseBasePolicy& writer) const {
    if (!writer.set_uint8_t(id)) {
        return false;
    }
    if (!writer.set_uint8_t(lowAge)) {
        return false;
    }
    if (!writer.set_uint8_t(highAge)) {
        return false;
    }
    if (!writer.set_uint16_t(capacity)) {
        return false;
    }
    if (!writer.set_string(name)) {
        return false;
    }
    return true;
}

bool SrvLoginReq::ChannelInfo::read(CRoseReader& reader) {
    if (!reader.get_uint8_t(id)) {
        return false;
    }
    if (!reader.get_uint8_t(lowAge)) {
        return false;
    }
    if (!reader.get_uint8_t(highAge)) {
        return false;
    }
    if (!reader.get_uint16_t(capacity)) {
        return false;
    }
    if (!reader.get_string(name)) {
        return false;
    }
    return true;
}

void SrvLoginReq::Test::set_a(const uint8_t a) {
    this->data.a = a;
}

const uint8_t SrvLoginReq::Test::get_a() const {
    return data.a;
}

void SrvLoginReq::Test::set_b(const uint8_t b) {
    this->data.b = b;
}

const uint8_t SrvLoginReq::Test::get_b() const {
    return data.b;
}

bool SrvLoginReq::Test::write(CRoseBasePolicy& writer) const {
    if (!writer.set_union(data)) {
        return false;
    }
    return true;
}

bool SrvLoginReq::Test::read(CRoseReader& reader) {
    if (!reader.get_union(data)) {
        return false;
    }
    return true;
}


SrvLoginReq::SrvLoginReq() : CRosePacket(ePacketType::PAKCS_LOGIN_REQ) {}

SrvLoginReq::SrvLoginReq(CRoseReader reader) : CRosePacket(reader) {
    if (!reader.get_uint32_t(id)) {
        return;
    }
    if (!reader.get_iserialize(channels)) {
        return;
    }
}

void SrvLoginReq::set_id(const uint32_t id) {
    this->id = id;
}

const uint32_t SrvLoginReq::get_id() const {
    return id;
}

void SrvLoginReq::set_channels(const SrvLoginReq::ChannelInfo& channels) {
    this->channels = channels;
}

const SrvLoginReq::ChannelInfo& SrvLoginReq::get_channels() const {
    return channels;
}

SrvLoginReq SrvLoginReq::create(const uint32_t& id) {
    SrvLoginReq packet;
    packet.set_id(id);
    return packet;
}

void SrvLoginReq::pack(CRoseBasePolicy& writer) const {
    if (!writer.set_uint32_t(id)) {
        return;
    }
    if (!writer.set_iserialize(channels)) {
        return;
    }
}

