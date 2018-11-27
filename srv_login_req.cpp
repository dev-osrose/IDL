#include "srv_login_req.h"

using namespace RoseCommon::Packet;

SrvLoginReq::Password::Password() : is_valid(false) {}

SrvLoginReq::Password::Password(std::string data) : password(data), is_valid(false) {
    bool valid = false;
    if (password.size() > 32) {
        password.resize(32);
        valid &= true;
    } else {
        valid &= true;
    }
    is_valid = valid;
}

bool SrvLoginReq::Password::read(CRoseReader& reader) {
    bool valid = false;
    if (!reader.get_string(password, 32)) {
        return false;
    } else {
        valid &= true;
    }
    is_valid = valid;
    return true;
}

bool SrvLoginReq::Password::write(CRoseBasePolicy& writer) const {
    if (!writer.set_string(password, 32)) {
        return false;
    }
    return true;
}


SrvLoginReq::SrvLoginReq() : CRosePacket(ePacketType::PAKCS_LOGIN_REQ) {}

SrvLoginReq::SrvLoginReq(CRoseReader reader) : CRosePacket(reader) {
    if (!reader.get_iserialize(password)) {
        return;
    }
    if (!reader.get_string(username)) {
        return;
    }
    if (!reader.get_uint8_t(value)) {
        return;
    }
}

SrvLoginReq SrvLoginReq::create(Password, std::string, Test) {
}

void SrvLoginReq::pack(CRoseBasePolicy& writer) const {
    if (!writer.set_iserialize(password)) {
        return;
    }
    if (!writer.set_string(username)) {
        return;
    }
    if (!writer.set_iserialize(value)) {
        return;
    }
}

