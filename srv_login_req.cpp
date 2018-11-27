#include "srv_login_req.h"

using namespace RoseCommon::Packet;

SrvLoginReq::Password::Password() : is_valid(false) {}

SrvLoginReq::Password::Password(std::string data) : password(data), is_valid(false) {
    bool valid = true;
    if (password.size() > 32) {
        password.resize(32);
        valid &= true;
    } else {
        valid &= true;
    }
    is_valid = valid;
}

bool SrvLoginReq::Password::read(CRoseReader& reader) {
    bool valid = true;
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
}

void SrvLoginReq::set_password(const SrvLoginReq::Password& password) {
    this->password = password;
}

const SrvLoginReq::Password& SrvLoginReq::get_password() const {
    return password;
}

void SrvLoginReq::set_username(const std::string& username) {
    this->username = username;
}

const std::string& SrvLoginReq::get_username() const {
    return username;
}

SrvLoginReq SrvLoginReq::create(const SrvLoginReq::Password& password, const std::string& username) {
    SrvLoginReq packet;
    packet.set_password(password);
    packet.set_username(username);
    return packet;
}

void SrvLoginReq::pack(CRoseBasePolicy& writer) const {
    if (!writer.set_iserialize(password)) {
        return;
    }
    if (!writer.set_string(username)) {
        return;
    }
}

