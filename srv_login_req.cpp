#include "srv_login_req.h"

using namespace RoseCommon::Packet;



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

void SrvLoginReq::set_channels(const ChannelInfo& channels) {
    this->channels = channels;
}

const ChannelInfo& SrvLoginReq::get_channels() const {
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

